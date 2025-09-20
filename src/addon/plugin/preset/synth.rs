use std::path::Path;

pub async fn create_plugin_src_synth(src_path: &Path) -> Result<(), String> {
    if let Err(e) = std::fs::create_dir_all(src_path) {
        crate::utils::logger::Logger::new().log_message(
            crate::utils::logger::LogLevel::Error,
            &format!("Error creating plugin src directory: {}", e),
        );
        return Err(format!("Failed to create plugin src directory: {}", e));
    }

    if let Err(e) = create_plugin_synth_src_lib(src_path).await {
        crate::utils::logger::Logger::new().log_message(
            crate::utils::logger::LogLevel::Error,
            &format!("Error creating plugin src/lib.rs: {}", e),
        );
        return Err(format!("Failed to create plugin src/lib.rs: {}", e));
    }

    Ok(())
}

async fn create_plugin_synth_src_lib(rs_path: &Path) -> Result<(), String> {
    let lib_path = rs_path.join("lib.rs");
    let src_lib_content: &'static str = r#"// lib.rs — Simple synth preset (safe API)
// Exposes a tiny sine synth that ADDS to the input buffer.
// Authors implement safe functions; thin FFI glue is provided below.

use devalang_bindings::BufferParams;
use std::sync::{Mutex, OnceLock};

static PHASE: OnceLock<Mutex<f32>> = OnceLock::new();

fn with_phase<F, R>(f: F) -> R where F: FnOnce(&mut f32) -> R {
    let m = PHASE.get_or_init(|| Mutex::new(0.0));
    let mut g = m.lock().unwrap();
    f(&mut *g)
}

// ——— Safe API for authors ———
pub fn reset_phase_api() {
    with_phase(|p| *p = 0.0);
}

pub fn process_sine_add_api(out: &mut [f32], params: BufferParams, freq_hz: f32, amplitude: f32) {
    if params.sample_rate == 0 { return; }
    let sr = params.sample_rate as f32;
    let frames = params.frames as usize;
    let channels = params.channels as usize;
    let two_pi = 2.0 * core::f32::consts::PI;
    with_phase(|phase| {
        let step = two_pi * freq_hz / sr;
        for i in 0..frames {
            let s = (*phase).sin() * amplitude;
            let base = i * channels;
            if base + channels <= out.len() {
                for c in 0..channels { out[base + c] += s; }
            }
            *phase += step;
            if *phase >= two_pi { *phase -= two_pi; }
        }
    });
}

// ——— FFI glue (auto) ———
#[no_mangle]
pub extern "C" fn reset_phase() { reset_phase_api(); }

#[no_mangle]
pub extern "C" fn process_sine_add(
    ptr: *mut f32,
    len: usize,
    freq_hz: f32,
    sample_rate: f32,
    amplitude: f32,
){
    if ptr.is_null() || sample_rate <= 0.0 { return; }
    let params = BufferParams { sample_rate: sample_rate as u32, channels: 1, frames: len as u32 };
    unsafe {
        let out = core::slice::from_raw_parts_mut(ptr, len);
        process_sine_add_api(out, params, freq_hz, amplitude);
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
