use wavedrom_rs::Figure;
use wavedrom_rs::wavejson::WaveJson;

#[no_mangle]
pub extern fn malloc(size: usize) -> *const u8 {
    Vec::with_capacity(size).leak().as_ptr() as *const u8
}

#[no_mangle]
pub extern fn free(ptr: *mut u8, size: usize) {
    unsafe { Vec::from_raw_parts(ptr, 0, size) };
}

#[no_mangle]
pub extern fn render(ptr: *mut u8, size: usize) -> *const u8 {
    let bytes = unsafe { Vec::from_raw_parts(ptr, size, size) };
    let s = unsafe { String::from_utf8_unchecked(bytes) };

    use wavedrom_rs::ToSvg;

    let Ok(wavejson) = json5::from_str::<WaveJson>(&s[..]) else {
        return vec![1].leak().as_ptr();
    };

    
    let Ok(figure) = Figure::try_from(wavejson) else {
        return vec![2].leak().as_ptr();
    };
    let Ok(rendered) = figure.assemble() else {
        return vec![3].leak().as_ptr();
    };
    let mut buffer = vec![0; 5];

    let Ok(()) = rendered.write_svg(&mut buffer) else {
        return vec![4].leak().as_ptr();
    };

    let size = buffer.len() - 5;
    let [b0, b1, b2, b3] = size.to_be_bytes();

    buffer[1] = b0;
    buffer[2] = b1;
    buffer[3] = b2;
    buffer[4] = b3;

    buffer.leak().as_ptr()
}
