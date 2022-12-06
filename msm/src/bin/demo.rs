use argh::FromArgs;
use ark_bls12_377::Fr;

use cyclone_msm::{
    bls12_377::{into_weierstrass, G1PTEAffine},
    fpga,
    io::{load, load_beta, load_points, load_slice, store, store_slice},
    testing::{harness_digits, harness_points, harness_scalars},
    timing::{always_timed, timed},
    App, Command, Packet,
};

#[derive(FromArgs)]
/// Arguments for tests
pub struct Args {
    /// size of instance
    #[argh(positional)]
    pub size: u8,

    /// prefix of filenames
    #[argh(positional)]
    pub name: String,

    /// skip loading points
    #[argh(switch)]
    pub preloaded: bool,

    /// verbose output
    #[argh(switch, short = 'v')]
    pub verbose: bool,

    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    /// column
    Column(Column),
    Msm(Msm),
    Load(Load),
    Points(Points),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "column")]
/// Column
struct Column {}

#[derive(FromArgs)]
#[argh(subcommand, name = "msm")]
/// MSM
struct Msm {}

#[derive(FromArgs)]
#[argh(subcommand, name = "load")]
/// Load
struct Load {}

#[derive(FromArgs)]
#[argh(subcommand, name = "points")]
/// Generate points
struct Points {}

fn main() {
    let args: Args = argh::from_env();

    match args.subcommand {
        Subcommand::Column(_) => {
            let fpga = fpga().unwrap();
            let mut app = App::new(fpga, args.size);
            let beta = load_beta(&args.name);

            if !args.preloaded {
                let points = load_points(args.size, &args.name);
                always_timed("setting points", || app.set_preprocessed_points(&points));
            }

            if args.verbose {
                println!("{:?}", app.statistics());
            }

            let (digits, sum) =
                always_timed("generating test case", || harness_digits(&beta, args.size));
            let point = always_timed("column sum", || {
                if args.verbose {
                    println!("{:?}", app.statistics());
                }

                let mut packet = Packet::default();
                let mut stream = app.start_column();
                for chunk in digits.chunks(8) {
                    for (digit, cmd) in chunk.iter().zip(packet.iter_mut()) {
                        *cmd = Command::set_digit(*digit);
                    }
                    stream.write(&packet);
                }

                let point = app.get_point();
                into_weierstrass(&point)
            });

            if args.verbose {
                println!("{:?}", app.statistics());
            }

            if cfg!(feature = "hw") {
                if point != sum {
                    println!("\n==> FAILURE <==");
                    std::process::exit(1);
                } else {
                    println!("\n==> SUCCESS <==");
                }
            } else {
                println!("sum: {:?}", &sum);
                println!("res: {:?}", &point);
            }
        }
        Subcommand::Msm(_) => {
            let fpga = fpga().unwrap();
            let mut app = App::new(fpga, args.size);
            let beta = load_beta(&args.name);

            if !args.preloaded {
                let points = load_points(args.size, &args.name);
                always_timed("setting points", || app.set_preprocessed_points(&points));
            }

            if args.verbose {
                println!("{:?}", app.statistics());
            }

            let (scalars, sum) =
                always_timed("generating test case", || harness_scalars(&beta, args.size));
            let point = always_timed(&format!("MSM/{}", args.size), || app.msm(scalars.iter()));

            // let (scalars, sum) =
            //     always_timed("generating test case", || cyclone_msm::testing::harness_bigints(&beta, args.size));
            // let point = always_timed(&format!("MSM/{}", args.size), || app.msm_bigint(&scalars));

            if args.verbose {
                println!("{:?}", app.statistics());
            }

            if cfg!(feature = "hw") {
                if point != sum {
                    println!("\n==> FAILURE <==");
                    std::process::exit(1);
                } else {
                    println!("\n==> SUCCESS <==");
                }
            } else {
                println!("sum: {:?}", &sum);
                println!("res: {:?}", &point);
            }
        }

        Subcommand::Load(_) => {
            let fpga = fpga().unwrap();
            let mut app = App::new(fpga, args.size);
            let points = load_points(args.size, &args.name);
            always_timed("setting points", || app.set_preprocessed_points(&points));
        }

        Subcommand::Points(_) => {
            let len: usize = 1usize << args.size;

            // let data = harness(size as _);
            // msm_fpga::store(&data, "harness.bin");
            // let (points, digits, sum) = data;
            let (beta, points) = harness_points(args.size);

            let beta_name = format!("{}.beta", args.name);
            let points_name = format!("{}.points", args.name);

            store(&beta, &beta_name);
            let mut beta_load = Fr::default();
            load(&mut beta_load, &beta_name);
            println!("loaded beta {}", beta);
            assert_eq!(beta, beta_load);

            store_slice(&points, &points_name);
            let mut points_load = vec![G1PTEAffine::zero(); len];
            timed("loading", || load_slice(&mut points_load, &points_name));
            let equal = points == points_load;
            assert!(equal);
        }
    }
}
