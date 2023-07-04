#![no_main]

use libfuzzer_sys::fuzz_target;

use wavedrom::wavejson::WaveJson;
use wavedrom::Figure;

fuzz_target!(|data: WaveJson| {
    let figure = Figure::from(data);

    let Figure::Signal(figure) = figure;

    let assembled = figure.assemble();
    let mut writer = Vec::new();
    let _ = assembled.write_svg(&mut writer);
});
