use core::iter;

use ark_bls12_377::{Fq, G1Affine, G1TEProjective};
use ark_std::Zero;
// use our_ff::{FromBytes, ToBytes};

use fpga::{
    f1::Stream as F1Stream, Fpga, SendBuffer64 as SendBuffer, Stream as _, Streamable as _, F1,
};

use crate::{
    digits::single_digit_carry, limb_carries, preprocess::into_weierstrass, timed, G1PTEAffine,
    G1Projective, Scalar,
};

const DDR_READ_LEN: u32 = 64;

const NUM_BUCKETS: u32 = 1 << 15;
const FIRST_BUCKET: u32 = 0;
const LAST_BUCKET: u32 = NUM_BUCKETS - 1;

const BACKOFF_THRESHOLD: u32 = 64;
const SET_POINTS_FLUSH_EVERY: usize = 1024;
const SET_DIGITS_FLUSH_BACKOFF_EVERY: usize = 512;

fn shl_assign(point: &mut G1TEProjective, c: usize) {
    use ark_ec::Group;
    (0..c).for_each(|_| {
        point.double_in_place();
    })
}

#[repr(usize)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Top-level commands of the FPGA App's interface.
/// Of more interest are the subcommands of `Stream::Msm` in [`Command`].
pub enum Stream {
    SetX = 1 << 26,
    SetY = 2 << 26,
    SetKT = 3 << 26,
    // must start with Cmd::Start, then packets of Cmd::SetDigit
    Msm = 4 << 26,
    SetZero = 5 << 26,
}

#[repr(u64)]
pub enum Cmd {
    Start = 1,
    SetDigit = 3,
}

impl Cmd {
    #[inline(always)]
    pub fn set_digit(digit: i16) -> u64 {
        Cmd::SetDigit as u64 | (digit as u16 as u64) << 14
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WriteRegister {
    // parametrised read registers Statistic, X, Y, Z need a preceding query with the parameter
    Query = 0x10,
    DdrReadLen = 0x11,
    MsmLength = 0x20,
    LastBucket = 0x21,
    FirstBucket = 0x22,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReadRegister {
    Statistic = 0x20,
    DigitsQueue = 0x21,
    Aggregated = 0x30,
    X = 0x31,
    Y = 0x32,
    Z = 0x33,
    T = 0x34,
}

#[repr(u32)]
// TODO: double check these are named correctly
pub enum Statistic {
    DroppedCommands = 0,
    DdrReadMiss = 1,
    DdrWriteMiss = 2,
    DdrPushCount = 3,
    DdrReadCountChannel1 = 4,
    DdrReadCountChannel2 = 5,
    DdrReadCountChannel3 = 6,
}

#[derive(Copy, Clone, Debug)]
// TODO: double check these are named correctly
pub struct Statistics {
    pub dropped_commands: u32,
    pub ddr_read_miss: u32,
    pub ddr_write_miss: u32,
    pub ddr_push_count: u32,
    pub ddr_read_count_channel_1: u32,
    pub ddr_read_count_channel_2: u32,
    pub ddr_read_count_channel_3: u32,
}

pub struct App {
    pub fpga: F1,
    len: usize,
    pool: Option<rayon::ThreadPool>,
    carried: Option<Vec<Scalar>>,
}

impl App {
    pub fn new(fpga: F1, size: u8) -> Self {
        assert!(size < 32);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        let mut app = App {
            fpga,
            len: 1 << size,
            pool: Some(pool),
            carried: Some(vec![Scalar::default(); 1 << size]),
        };
        app.set_size();
        app.set_first_bucket();
        app.set_last_bucket();
        app.set_ddr_read_len();
        app.set_zero();

        app
    }

    #[inline]
    fn column(&mut self, i: usize, scalars: &[Scalar], total: &mut G1TEProjective) {
        let mut cmds = SendBuffer::default();
        for j in (0..4).rev() {
            timed(&format!("\n:: column {}", j as usize), || {
                let mut stream = self.start();

                for chunk in scalars.chunks(8) {
                    for (cmd, scalar) in cmds.iter_mut().zip(chunk) {
                        let digit = single_digit_carry(scalar, i, j);
                        *cmd = Cmd::set_digit(digit);
                    }
                    stream.write(&cmds);
                }

                *total += timed("fetching point", || self.get_point());
                if (i, j) != (0, 0) {
                    shl_assign(total, 16);
                }
            });
        }
    }

    /// Perform full MSM.
    #[inline]
    pub fn msm(&mut self, scalars: &[Scalar]) -> G1Projective {
        assert_eq!(scalars.len(), self.len);

        let pool = self.pool.take().unwrap_or_else(|| unreachable!());
        let mut carried = self.carried.take().unwrap_or_else(|| unreachable!());

        let mut total = G1TEProjective::zero();
        let mut total0 = G1TEProjective::zero();
        pool.scope(|s| {
            s.spawn(|_| {
                timed("limb carries", || limb_carries(scalars, &mut carried));
            });

            s.spawn(|_| {
                self.column(0, scalars, &mut total0);
            });
        });

        for i in (1..4).rev() {
            self.column(i, &carried, &mut total);
        }

        shl_assign(&mut total, 48);
        total += total0;

        let total = into_weierstrass(&total);
        self.pool = Some(pool);
        self.carried = Some(carried);
        total
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn set_zero(&mut self) {
        let zero = G1TEProjective::zero();
        let mut buffer = SendBuffer::default();

        let mut stream: F1Stream<'_, SetPointsBackoff> = self.fpga.stream(Stream::SetZero as _);

        buffer[..6].copy_from_slice(zero.x.0.as_ref());
        stream.write(&buffer);
        buffer[..6].copy_from_slice(zero.y.0.as_ref());
        stream.write(&buffer);
        buffer[..6].copy_from_slice(zero.z.0.as_ref());
        stream.write(&buffer);
        buffer[..6].copy_from_slice(zero.t.0.as_ref());
        stream.write(&buffer);

        self.fpga.flush();
    }

    fn set_size(&mut self) {
        self.fpga
            .write_register(WriteRegister::MsmLength as _, self.len as _);
    }

    fn set_last_bucket(&mut self) {
        self.fpga
            .write_register(WriteRegister::LastBucket as _, LAST_BUCKET);
    }

    fn set_first_bucket(&mut self) {
        self.fpga
            .write_register(WriteRegister::FirstBucket as _, FIRST_BUCKET);
    }

    fn set_ddr_read_len(&mut self) {
        self.fpga
            .write_register(WriteRegister::DdrReadLen as _, DDR_READ_LEN);
    }

    #[inline]
    fn set_coordinates(&mut self, coordinate: Stream, coordinates: impl Iterator<Item = Fq>) {
        debug_assert!([
            coordinate == Stream::SetX,
            coordinate == Stream::SetY,
            coordinate == Stream::SetKT
        ]
        .iter()
        .any(|&condition| condition));
        let mut buffer = SendBuffer::default();
        let mut stream: F1Stream<'_, SetPointsBackoff> = self.fpga.stream(coordinate as _);
        for coordinate in coordinates {
            buffer[..6].copy_from_slice(coordinate.0.as_ref());
            stream.write(&buffer);
        }
    }
    #[inline]
    pub fn set_preprocessed_points(&mut self, points: &[G1PTEAffine]) {
        assert!(self.len == points.len());

        self.set_coordinates(Stream::SetX, points.iter().map(|point| point.x));
        self.set_coordinates(Stream::SetY, points.iter().map(|point| point.y));
        self.set_coordinates(Stream::SetKT, points.iter().map(|point| point.kt));
    }

    pub fn set_points(&mut self, points: &[G1Affine]) {
        assert!(self.len == points.len());
        let preprocessed_points: Vec<_> = points.iter().map(|point| point.into()).collect();
        self.set_preprocessed_points(&preprocessed_points);
    }

    pub fn set_preprocessed_point_repeatedly(&mut self, point: &G1PTEAffine) {
        self.set_coordinates(Stream::SetX, iter::repeat(point.x).take(self.len));
        self.set_coordinates(Stream::SetY, iter::repeat(point.y).take(self.len));
        self.set_coordinates(Stream::SetKT, iter::repeat(point.kt).take(self.len));
    }

    fn get_coordinate(&mut self, coordinate: ReadRegister) -> Fq {
        debug_assert!([
            coordinate == ReadRegister::X,
            coordinate == ReadRegister::Y,
            coordinate == ReadRegister::Z,
            coordinate == ReadRegister::T,
        ]
        .iter()
        .any(|&condition| condition));
        let mut buffer = [0u64; 6];
        for j in (0..12).step_by(2) {
            self.fpga.write_register(WriteRegister::Query as _, j);
            let lo = self.fpga.read_register(coordinate as _);

            self.fpga.write_register(WriteRegister::Query as _, j + 1);
            let hi = self.fpga.read_register(coordinate as _);

            // | has lower precedence than <<, whereas + has higher
            // and would need parentheses
            buffer[j as usize / 2] = (hi as u64) << 32 | lo as u64;
        }
        ark_ff::BigInt(buffer).into()
    }

    pub fn get_point(&mut self) -> G1TEProjective {
        self.fpga.flush();
        while 0 == self.fpga.read_register(ReadRegister::Aggregated as _) {
            continue;
        }

        let mut point = G1TEProjective::zero();
        point.x = self.get_coordinate(ReadRegister::X);
        point.y = self.get_coordinate(ReadRegister::Y);
        point.z = self.get_coordinate(ReadRegister::Z);
        point.t = self.get_coordinate(ReadRegister::T);

        point
    }

    pub fn statistics(&mut self) -> Statistics {
        use Statistic::*;
        Statistics {
            dropped_commands: self.statistic(DroppedCommands),
            ddr_read_miss: self.statistic(DdrReadMiss),
            ddr_write_miss: self.statistic(DdrWriteMiss),
            ddr_push_count: self.statistic(DdrPushCount),
            ddr_read_count_channel_1: self.statistic(DdrReadCountChannel1),
            ddr_read_count_channel_2: self.statistic(DdrReadCountChannel2),
            ddr_read_count_channel_3: self.statistic(DdrReadCountChannel3),
        }
    }

    pub fn statistic(&mut self, statistic: Statistic) -> u32 {
        self.fpga
            .write_register(WriteRegister::Query as _, statistic as _);
        self.fpga.read_register(ReadRegister::Statistic as _)
    }

    pub fn start(&mut self) -> F1Stream<'_, DigitsBackoff> {
        let mut stream = self.fpga.stream(Stream::Msm as _);

        let mut cmds = SendBuffer::default();
        cmds[0] = Cmd::Start as _;
        stream.write(&cmds);
        stream.flush();
        stream
    }
}

pub struct SetPointsBackoff;
impl fpga::Backoff<F1> for SetPointsBackoff {
    #[inline(always)]
    fn backoff(fpga: &mut F1, offset: usize) {
        if (offset % SET_POINTS_FLUSH_EVERY) == 0 {
            fpga.flush();
        }
    }
}

pub struct DigitsBackoff;
impl fpga::Backoff<F1> for DigitsBackoff {
    #[inline(always)]
    fn backoff(fpga: &mut F1, offset: usize) {
        if (offset % SET_DIGITS_FLUSH_BACKOFF_EVERY) == 0 {
            fpga.flush();
            while fpga.read_register(ReadRegister::DigitsQueue as _) > BACKOFF_THRESHOLD {
                continue;
            }
        }
    }
}
