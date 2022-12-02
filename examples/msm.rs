use cyclone_msm::{
    fpga,
    io::{load_beta, load_points},
    testing::harness_scalars,
    timing::always_timed,
    App,
};

#[path = "../src/examples-args.rs"]
mod args;

fn main() {
    let args: args::Args = argh::from_env();

    let fpga = fpga().unwrap();

    let mut app = App::new(fpga, args.size);

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
    let point = always_timed(&format!("MSM/{}", args.size), || app.msm(&scalars));

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
