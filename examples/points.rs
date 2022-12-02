use ark_bls12_377::Fr;
use cyclone_msm::{
    bls12_377::G1PTEAffine,
    io::{load, load_slice, store, store_slice},
    testing::harness_points,
    timing::timed,
};

fn main() {
    let size = std::env::args()
        .nth(1)
        .expect("pass with SIZE argument")
        .parse()
        .expect("SIZE invalid as u8");
    let len: usize = 1usize << size;

    let name = std::env::args().nth(2).expect("pass with NAME argument");

    // let data = harness(size as _);
    // msm_fpga::store(&data, "harness.bin");
    // let (points, digits, sum) = data;
    let (beta, points) = harness_points(size);

    let beta_name = format!("{}.beta", name);
    let points_name = format!("{}.points", name);

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
