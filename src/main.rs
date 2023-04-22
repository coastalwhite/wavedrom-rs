use wavedrom_rs::Wave;

fn main() {
    let wave = Wave {
        name: String::from("Test"),
        cycles: "12..11043...110..101001".parse().unwrap(),
    };

    let mut buff = String::new();
    wave.to_svg(&mut buff).unwrap();

    std::fs::write("test.svg", buff).unwrap();
}
