use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = cc::Build::new();

    let builder = builder
        .file("upstream/fpga_libs/fpga_pci/fpga_pci.c")
        .file("upstream/fpga_libs/fpga_pci/fpga_pci_sysfs.c")
        .file("upstream/utils/io.c")
        .include("upstream/include")
        .opt_level(3)
        .flag("-mavx2")
        // can't fix upstream warnings
        .warnings(false);

    builder.compile("f1-sys");

    let bindings = bindgen::Builder::default()
        .header("upstream/include/fpga_pci.h")
        .use_core()
        .ctypes_prefix("cty")
        .rustfmt_bindings(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    Ok(())
}
