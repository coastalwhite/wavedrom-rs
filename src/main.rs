use wavedrom_rs::{Figure, ToSvg, Wave};

fn main() {
    let wave = Figure::from_lines(vec![
        Wave {
            name: String::from("Broken"),
            cycles: "4013".parse().unwrap(),
        },
        Wave {
            name: String::from("T"),
            cycles: "12..11043...110..101001".parse().unwrap(),
        },
        Wave {
            name: String::from("Ads..........kgjlajflk....."),
            cycles: "0...1..2....3...5.1010.1".parse().unwrap(),
        },
        Wave {
            name: String::from("Ads..........kgjlajflk....."),
            cycles: "0...1..2....3...5.1010.1".parse().unwrap(),
        },
    ]);

    let figure = wave.assemble().unwrap();

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open("test.svg")
        .unwrap();

    figure.write_svg(&mut file).unwrap();
}
