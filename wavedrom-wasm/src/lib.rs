use std::alloc::Layout;

use wavedrom::wavejson::WaveJson;
use wavedrom::Figure;

mod options;
pub use options::{get_parameter, modify_parameter, OPTIONS};

use self::options::merge_in_skin_internal;

/// # Safety
/// Free afterwards
#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
    unsafe { std::alloc::alloc(Layout::array::<u8>(size).unwrap()) }
}

/// # Safety
/// Only call on malloced chunks
#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut u8, size: usize) {
    unsafe { std::alloc::dealloc(ptr, Layout::array::<u8>(size).unwrap()) }
}

#[repr(u8)]
enum RenderError {
    JsonDeserializeError = 1,
    WriteError = 2,
    InvalidUtf8 = 3,
}

fn render_internal(json: &str) -> Result<Vec<u8>, RenderError> {
    let Ok(wavejson) = WaveJson::from_json5(json) else {
        return Err(RenderError::JsonDeserializeError);
    };

    let figure = Figure::from(wavejson);

    let mut buffer = vec![0; 9];

    {
        let options = &*OPTIONS.lock().unwrap();
        match figure {
            Figure::Signal(figure) => {
                let Ok(_) = figure
                    .assemble_with_options(options)
                    .write_svg_with_options(&mut buffer, options)
                else {
                    return Err(RenderError::WriteError);
                };
            }
            Figure::Register(figure) => {
                let Ok(_) = figure.write_svg_with_options(&mut buffer, options) else {
                    return Err(RenderError::WriteError);
                };
            }
        }
    }

    let size = buffer.len() - 9;
    let capacity = buffer.capacity();

    for (i, b) in capacity.to_be_bytes().into_iter().enumerate() {
        buffer[i + 1] = b;
    }
    for (i, b) in size.to_be_bytes().into_iter().enumerate() {
        buffer[i + 5] = b;
    }

    Ok(buffer)
}

/// # Safety
/// Always give valid ptr
#[no_mangle]
pub unsafe extern "C" fn render(ptr: *mut u8, size: usize, capacity: usize) -> *const u8 {
    let bytes = unsafe { Vec::from_raw_parts(ptr, size, capacity) };
    let Ok(json) = String::from_utf8(bytes) else {
        return Box::leak(Box::new(RenderError::InvalidUtf8 as u8)) as *const u8;
    };

    match render_internal(&json[..]) {
        Ok(svg) => svg.leak().as_ptr(),
        Err(err) => Box::leak(Box::new(err as u8)) as *const u8,
    }
}

/// # Safety
/// Always give valid ptr
#[no_mangle]
pub unsafe extern "C" fn merge_in_skin(ptr: *mut u8, size: usize, capacity: usize) -> u8 {
    let bytes = unsafe { Vec::from_raw_parts(ptr, size, capacity) };
    let Ok(json) = String::from_utf8(bytes) else {
        return 1;
    };

    match merge_in_skin_internal(&json[..]) {
        Ok(_) => 0,
        Err(_) => 2,
    }
}

#[no_mangle]
pub extern "C" fn reset_parameters() {
    options::reset()
}

#[no_mangle]
pub extern "C" fn export_parameters() -> *const u8 {
    match options::export() {
        Ok(v) => {
            let mut out = Vec::with_capacity(v.len() + 5);
            out.push(0u8);
            let bs = v.len().to_be_bytes();

            for b in bs.into_iter().take(4) {
                out.push(b);
            }

            out.extend(v.into_bytes());
            out.leak().as_ptr()
        }
        Err(_) => Box::leak(Box::new(1u8)) as *const u8,
    }
}
