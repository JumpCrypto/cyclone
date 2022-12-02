use cyclone_msm::{preprocess::into_weierstrass, random_points, timed, App, Command};
use fpga::f1::Packet;

fn main() {
    const SIZE: u8 = 1;

    let f1 = fpga::F1::new(0, 0x500).unwrap();
    let mut app = App::new(f1, SIZE);
    let points = timed("generating random points", || random_points(SIZE));
    let expected = points[0] + points[1];

    app.set_points(&points);

    let mut stream = app.start_column();
    let mut packet = Packet::default();
    packet[0] = Command::set_digit(1);
    packet[1] = Command::set_digit(1);
    stream.write(&packet);

    let point = into_weierstrass(&app.get_point());
    assert_eq!(expected, point);
}
