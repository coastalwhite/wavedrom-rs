use wavedrom_rs::{Figure, Wave};

fn main() {
    let wave = Figure(vec![
        Wave {
            name: String::from("T"),
            cycles: "12..11043...110..101001".parse().unwrap(),
        },
        Wave {
            name: String::from("AdsafkljjBC"),
            cycles: "0...1..2....3...5.1010.1".parse().unwrap(),
        },
    ]);

    let mut buff = String::new();
    wave.to_svg(&mut buff).unwrap();

    std::fs::write("test.svg", buff).unwrap();
}
