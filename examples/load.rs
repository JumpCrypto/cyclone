use cyclone_msm::{always_timed, load_points, App};
use fpga::F1;

#[path = "../src/examples-args.rs"]
mod args;

fn main() {
    let args: args::Args = argh::from_env();

    let f1 = F1::new(0, 0x500).unwrap();
    let mut app = App::new(f1, args.size);

    let points = load_points(args.size, &args.name);
    always_timed("setting points", || app.set_preprocessed_points(&points));
}
