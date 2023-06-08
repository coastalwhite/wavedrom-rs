use wavedrom_rs::wavejson::WaveJson;
use wavedrom_rs::{Figure, SignalOptions};

fn main() {
    let data = r#"
        {
            "signal": [
                [
                    "xyz",
                    { "name": "0", "wave": "0" },
                    { "name": "1", "wave": "1" },
                    { "name": "2", "wave": "2" },
                    { "name": "z", "wave": "z" },
                    { "name": "x", "wave": "x" },
                    { "name": "p", "wave": "p" },
                    { "name": "P", "wave": "P" },
                    { "name": "n", "wave": "n" },
                    { "name": "N", "wave": "N" }
                ],
                [
                    { "name": "0001020z0x0p0P0n0N0", "wave": "0001020z0x0p0P0n0N0" },
                    { "name": "1011121z1x1p0P0n0N0", "wave": "1011121z1x1p1P1n1N1" },
                    { "name": "2021222z2x2p2P2n2N2", "wave": "2021222z2x2p2P2n2N2" },
                    { "name": "z0z1z2zzzxzpzPznzNz", "wave": "z0z1z2zzzxzpzPznzNz" },
                    { "name": "x0x1x2xzxxxpxPxnxNx", "wave": "x0x1x2xzxxxpxPxnxNx" },
                    { "name": "p0p1p2pzpxpppPpnpNp", "wave": "p0p1p2pzpxpppPpnpNp" },
                    { "name": "P0P1P2PzPxPpPPPnPNP", "wave": "P0P1P2PzPxPpPPPnPNP" },
                    { "name": "n0n1n2nznxnpnPnnnNn", "wave": "n0n1n2nznxnpnPnnnNn" },
                    { "name": "N0N1N2NzNxNpNPNnNNN", "wave": "N0N1N2NzNxNpNPNnNNN" }
                ],
                [
                    { "name": "012345zx", "wave": "012345zx" },
                    {
                        "name": "02....3...0",
                        "wave": "02....3...0",
                        "data": [
                            "0xDEAD",
                            "0xBEEF"
                        ]
                    }
                ]
            ]
        }
        "#;
    let wavejson: WaveJson = serde_json::from_str(data).unwrap();
    let figure: Figure = wavejson.try_into().unwrap();

    for _ in 0..100000 {
        let rendered = figure.assemble().unwrap();
        // for line in rendered.lines {
            // let assembled = line.path.render_with_options(&WaveOptions::default());
            // assert!(assembled.segments().len() != 0);
        // }

    }
}
