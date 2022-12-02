use cyclone_msm::{always_timed, fpga, load_points, App};

#[path = "../src/examples-args.rs"]
mod args;

fn main() {
    let args: args::Args = argh::from_env();

    let fpga = fpga().unwrap();
    let mut app = App::new(fpga, args.size);

    let points = load_points(args.size, &args.name);
    always_timed("setting points", || app.set_preprocessed_points(&points));
}
