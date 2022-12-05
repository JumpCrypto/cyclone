//! Timing utilities.

use std::time::SystemTime;

#[cfg(feature = "timings")]
#[inline]
pub fn timed<R>(name: &str, f: impl FnOnce() -> R) -> R {
    println!("{} ...", name);
    let t = SystemTime::now();
    let r = f();
    println!("... {:?}", t.elapsed());
    r
}

#[cfg(not(feature = "timings"))]
#[inline]
pub fn timed<R>(_: &str, f: impl FnOnce() -> R) -> R {
    f()
}

#[inline]
pub fn always_timed<R>(name: &str, f: impl FnOnce() -> R) -> R {
    println!(":: {}...", name);
    let t = SystemTime::now();
    let r = f();
    println!("   {:?}", t.elapsed().unwrap());
    r
}
