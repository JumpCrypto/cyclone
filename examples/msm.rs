use cyclone_msm::{always_timed, harness_scalars, load_beta, load_points, App};
use fpga::F1;

#[path = "../src/examples-args.rs"]
mod args;

fn main() {
    let args: args::Args = argh::from_env();

    let f1 = F1::new(0, 0x500).unwrap();
    let mut app = App::new(f1, args.size);

    let beta = load_beta(&args.name);
    if !args.preloaded {
        let points = load_points(args.size, &args.name);
        always_timed("setting points", || app.set_preprocessed_points(&points));
    }
    let (scalars, sum) = always_timed("generating test case", || harness_scalars(&beta, args.size));

    if args.verbose {
        println!("{:?}", app.statistics());
    }

    // the MSM
    let total = always_timed(&format!("MSM/{}", args.size), || app.msm(&scalars));

    if args.verbose {
        println!("{:?}", app.statistics());
    }

    if total != sum {
        println!("\n==> FAILURE <==");
        std::process::exit(1);
    } else {
        println!("\n==> SUCCESS <==");
    }
}
