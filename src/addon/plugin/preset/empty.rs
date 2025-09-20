use std::path::Path;

pub async fn create_plugin_src_empty(src_path: &Path) -> Result<(), String> {
    if let Err(e) = std::fs::create_dir_all(src_path) {
        crate::utils::logger::Logger::new().log_message(
            crate::utils::logger::LogLevel::Error,
            &format!("Error creating plugin src directory: {}", e),
        );
        return Err(format!("Failed to create plugin src directory: {}", e));
    }

    if let Err(e) = create_plugin_empty_src_lib(src_path).await {
        crate::utils::logger::Logger::new().log_message(
            crate::utils::logger::LogLevel::Error,
            &format!("Error creating plugin src/lib.rs: {}", e),
        );
        return Err(format!("Failed to create plugin src/lib.rs: {}", e));
    }

    Ok(())
}

async fn create_plugin_empty_src_lib(rs_path: &Path) -> Result<(), String> {
    let lib_path = rs_path.join("lib.rs");
    let src_lib_content: &'static str = r#"// lib.rs — Empty preset (safe API)
// Authors implement the safe function below. Thin FFI glue is provided.

use devalang_bindings::{BufferParams, Note};

// ——— Safe API for authors ———
pub fn process_gain_api(out: &mut [f32], _params: BufferParams, _note: Note, gain: f32) {
    for s in out.iter_mut() {
        *s *= gain;
    }
}

// ——— FFI glue (auto) ———
#[no_mangle]
pub extern "C" fn process_gain(ptr: *mut f32, len: usize, gain: f32) {
    if ptr.is_null() { return; }
    let params = BufferParams { sample_rate: 44100, channels: 1, frames: len as u32 };
    unsafe {
        let out = core::slice::from_raw_parts_mut(ptr, len);
        process_gain_api(out, params, Note::default(), gain);
    }
}
"#;

    if let Err(e) = std::fs::write(&lib_path, src_lib_content) {
        crate::utils::logger::Logger::new().log_message(
            crate::utils::logger::LogLevel::Error,
            &format!("Error creating lib.rs: {}", e),
        );
        return Err(format!("Failed to create lib.rs: {}", e));
    }

    Ok(())
}
