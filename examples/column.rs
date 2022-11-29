use cyclone_msm::{
    always_timed, harness_digits, load_beta, load_points, preprocess::into_weierstrass, timed, App,
    Instruction,
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
            app.print_stats();
        }
        app.start();

        let mut cmds = SendBuffer64::default();
        for chunk in digits.chunks(8) {
            for (digit, cmd) in chunk.iter().zip(cmds.iter_mut()) {
                *cmd = Instruction::new(*digit as i16);
            }
            app.update(&cmds);
        }
        app.flush();

        let register_point = app.get_point_register();
        let dma_point = app.get_point_dma();
        assert_eq!(register_point, dma_point);
        into_weierstrass(&dma_point)
    });

    if point != sum {
        println!("\n==> FAILURE <==");
        std::process::exit(1);
    } else {
        println!("\n==> SUCCESS <==");
    }
}
