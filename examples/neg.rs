use cyclone_msm::{preprocess::into_weierstrass, random_points, timed, App, Instruction};
use fpga::SendBuffer64;

fn main() {
    const SIZE: u8 = 0;

    let f1 = fpga::F1::new(0, 0x500).unwrap();
    let mut app = App::new(f1, SIZE);
    let points = timed("generating random points", || random_points(SIZE));
    let expected = -points[0];

    app.set_points(&points);
    app.start();

    let mut cmds = SendBuffer64::default();
    cmds[0] = Instruction::new(-1);
    app.update(&cmds);

    let point = into_weierstrass(&app.get_point());
    assert_eq!(expected, point);
}
