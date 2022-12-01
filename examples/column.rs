use cyclone_msm::{
    always_timed, harness_digits, load_beta, load_points, preprocess::into_weierstrass, timed, App,
    Cmd,
};
use fpga::{SendBuffer64, F1};

#[path = "../src/examples-args.rs"]
mod args;

fn main() {
    let args: args::Args = argh::from_env();

    let f1 = F1::new(0, 0x500).unwrap();
    let mut app = App::new(f1, args.size);

    let beta = load_beta(&args.name);
    if !args.preloaded {
        let points = load_points(args.size, &args.name);
        timed("setting points", || app.set_preprocessed_points(&points));
    }
    let (digits, sum) = always_timed("generating test case", || harness_digits(&beta, args.size));

    let point = always_timed("column sum", || {
        if args.verbose {
            println!("{:?}", app.statistics());
        }

        use fpga::Stream;
        let mut stream = app.start();

        let mut packet = SendBuffer64::default();
        for chunk in digits.chunks(8) {
            for (digit, cmd) in chunk.iter().zip(packet.iter_mut()) {
                *cmd = Cmd::set_digit(*digit);
            }
            stream.write(&packet);
        }

        let point = app.get_point();
        into_weierstrass(&point)
    });

    if point != sum {
        println!("\n==> FAILURE <==");
        std::process::exit(1);
    } else {
        println!("\n==> SUCCESS <==");
    }
}
