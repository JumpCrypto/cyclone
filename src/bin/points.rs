use ark_bls12_377::Fr;
use cyclone_msm::{
    bls12_377::G1PTEAffine,
    io::{load, load_slice, store, store_slice},
    testing::harness_points,
    timing::timed,
};

#[derive(argh::FromArgs)]
/// Arguments for tests
pub struct Args {
    /// size of instance
    #[argh(positional)]
    pub size: u8,

    /// prefix of filenames
    #[argh(positional)]
    pub name: String,
}

fn main() {
    let args: Args = argh::from_env();
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
