use cyclone_msm::{
    bls12_377::into_weierstrass,
    fpga,
    io::{load_beta, load_points},
    testing::harness_digits,
    timing::{always_timed, timed},
    App, Command, Packet,
};

#[path = "../bin-lib/args.rs"]
mod args;

fn main() {
    let args: args::Args = argh::from_env();

    let fpga = fpga().unwrap();
    let mut app = App::new(fpga, args.size);

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

        let mut packet = Packet::default();
        let mut stream = app.start_column();
        for chunk in digits.chunks(8) {
            for (digit, cmd) in chunk.iter().zip(packet.iter_mut()) {
                *cmd = Command::set_digit(*digit);
            }
            stream.write(&packet);
        }

        let point = app.get_point();
        into_weierstrass(&point)
    });

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
