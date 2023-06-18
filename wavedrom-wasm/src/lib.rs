use wavedrom::wavejson::WaveJson;
use wavedrom::Figure;

mod render_options;
pub use render_options::{get_parameter, modify_parameter};

use crate::render_options::{get_assemble_options, get_render_options};

use self::render_options::merge_in_skin_internal;

/// # Safety
/// Free afterwards
#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *const u8 {
    Vec::with_capacity(size).leak().as_ptr() as *const u8
}

/// # Safety
/// Only call on malloced chunks
#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut u8, size: usize) {
    unsafe { Vec::from_raw_parts(ptr, 0, size) };
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

    let mut buffer = vec![0; 5];

    {
        let assemble_options = get_assemble_options();
        let render_options = get_render_options();
        let Ok(()) = figure.assemble_with_options(*assemble_options).write_svg_with_options(&mut buffer, render_options) else {
            return Err(RenderError::WriteError);
        };
    }

    let size = buffer.len() - 5;
    let bs = size.to_be_bytes();

    for (i, b) in bs.into_iter().take(4).enumerate() {
        buffer[i + 1] = b;
    }

    Ok(buffer)
}

/// # Safety
/// Always give valid ptr
#[no_mangle]
pub unsafe extern "C" fn render(ptr: *mut u8, size: usize) -> *const u8 {
    let bytes = unsafe { Vec::from_raw_parts(ptr, size, size) };
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
pub unsafe extern "C" fn merge_in_skin(ptr: *mut u8, size: usize) -> u8 {
    let bytes = unsafe { Vec::from_raw_parts(ptr, size, size) };
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
    render_options::reset()
}

#[no_mangle]
pub extern "C" fn export_parameters() -> *const u8 {
    match render_options::export() {
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
