use std::collections::HashMap;

pub struct WasmPluginRunner;

impl WasmPluginRunner {
    pub fn new() -> Self {
        WasmPluginRunner
    }

    pub fn process_in_place(&self, _wasm_bytes: &[u8], _buffer: &mut [f32]) -> Result<(), String> {
        Err("Wasm plugin execution is not available in wasm builds".to_string())
    }

    pub fn render_note_in_place(
        &self,
        _wasm_bytes: &[u8],
        _buffer: &mut [f32],
        _synth_name: Option<&str>,
        _freq: f32,
        _amp: f32,
        _duration_ms: i32,
        _sample_rate: i32,
        _channels: i32,
    ) -> Result<(), String> {
        Err("Wasm plugin rendering is not available in wasm builds".to_string())
    }

    pub fn render_note_with_params_in_place(
        &self,
        _wasm_bytes: &[u8],
        _buffer: &mut [f32],
        _synth_name: Option<&str>,
        _freq: f32,
        _amp: f32,
        _duration_ms: i32,
        _sample_rate: i32,
        _channels: i32,
        _params_num: &HashMap<String, f32>,
        _params_str: Option<&HashMap<String, String>>,
        _exported_names: Option<&[String]>,
    ) -> Result<(), String> {
        Err("Wasm plugin rendering is not available in wasm builds".to_string())
    }
}
