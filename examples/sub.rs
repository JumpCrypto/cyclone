use cyclone_msm::{preprocess::into_weierstrass, random_points, timed, App, Cmd};
use fpga::{SendBuffer64, Stream};

fn main() {
    const SIZE: u8 = 1;

    let f1 = fpga::F1::new(0, 0x500).unwrap();
    let mut app = App::new(f1, SIZE);
    let points = timed("generating random points", || random_points(SIZE));
    let expected = points[0] - points[1];

    app.set_points(&points);
    let mut stream = app.start();

    let mut cmds = SendBuffer64::default();
    cmds[0] = Cmd::set_digit(1);
    cmds[1] = Cmd::set_digit(-1);
    stream.write(&cmds);

    let point = into_weierstrass(&app.get_point());
    assert_eq!(expected, point);
}
