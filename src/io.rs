//! Load and store points efficiently.

use crate::{bls12_377::G1PTEAffine, timing::always_timed, Fr};

pub fn load_beta(name: &str) -> Fr {
    let beta_name = format!("{}.beta", name);
    let mut beta = Fr::default();
    load(&mut beta, &beta_name);
    beta
}

pub fn load_points(size: u8, name: &str) -> Vec<G1PTEAffine> {
    let points_name = format!("{}.points", name);
    let mut points = always_timed("allocating points", || vec![G1PTEAffine::zero(); 1 << size]);
    always_timed("loading points", || load_slice(&mut points, &points_name));
    points
}

pub fn store_slice<T: Sized>(slice: &[T], name: &str) {
    use std::io::Write as _;
    let slice_data_size = std::mem::size_of::<T>() * slice.len();
    std::fs::File::create(name)
        .unwrap()
        .write_all(unsafe {
            std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice_data_size)
        })
        .unwrap();
    println!("store {}B to {}", slice_data_size, name);
}

pub fn load_slice<T: Sized>(slice: &mut [T], name: &str) {
    use std::io::Read as _;
    let slice_data_size = std::mem::size_of::<T>() * slice.len();
    std::fs::File::open(name)
        .unwrap()
        .read_exact(unsafe {
            std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, slice_data_size)
        })
        .unwrap();
    println!("load {}B from {}", slice_data_size, name);
}
pub fn store<T: Sized>(data: &T, name: &str) {
    use std::io::Write as _;
    let size = std::mem::size_of::<T>();
    std::fs::File::create(name)
        .unwrap()
        .write_all(unsafe { std::slice::from_raw_parts(data as *const T as *const u8, size) })
        .unwrap();
    println!("store {}B to {}", size, name);
}

pub fn load<T: Sized>(data: &mut T, name: &str) {
    use std::io::Read as _;
    let size = std::mem::size_of::<T>();
    std::fs::File::open(name)
        .unwrap()
        .read_exact(unsafe { std::slice::from_raw_parts_mut(data as *mut T as *mut u8, size) })
        .unwrap();
    println!("load {}B from {}", size, name);
}
