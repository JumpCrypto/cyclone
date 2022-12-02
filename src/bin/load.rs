use cyclone_msm::{fpga, io::load_points, timing::always_timed, App};

#[path = "../bin-lib/args.rs"]
mod args;

fn main() {
    let args: args::Args = argh::from_env();

    let fpga = fpga().unwrap();
    let mut app = App::new(fpga, args.size);

    let points = load_points(args.size, &args.name);
    always_timed("setting points", || app.set_preprocessed_points(&points));
}
