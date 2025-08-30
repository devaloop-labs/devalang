use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use wasmtime::{Engine, Instance, Linker, Module, Store, TypedFunc};

#[cfg(not(target_arch = "wasm32"))]
type RenderFunc = TypedFunc<(i32, i32, f32, f32, i32, i32, i32), ()>;

#[cfg(not(target_arch = "wasm32"))]
pub struct WasmPluginRunner {
    engine: Engine,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for WasmPluginRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl WasmPluginRunner {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self { engine }
    }

    pub fn process_in_place(&self, wasm_bytes: &[u8], buffer: &mut [f32]) -> Result<(), String> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| format!("Failed to compile wasm: {e}"))?;

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate wasm: {e}"))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| "WASM memory export not found".to_string())?;

        let func = instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "process")
            .map_err(|_| "Exported function `process(i32,i32)` not found".to_string())?;

        let byte_len = std::mem::size_of_val(buffer) as i32;
        let ptr = Self::alloc_temp(&mut store, &instance, &memory, byte_len as usize)? as i32;
        let mem_slice = memory
            .data_mut(&mut store)
            .get_mut(ptr as usize..(ptr as usize) + (byte_len as usize))
            .ok_or_else(|| "Failed to get memory slice".to_string())?;

        let src_bytes =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, byte_len as usize) };
        mem_slice.copy_from_slice(src_bytes);

        func.call(&mut store, (ptr, buffer.len() as i32))
            .map_err(|e| format!("Error calling `process`: {e}"))?;

        let mem_slice_after = memory
            .data(&store)
            .get(ptr as usize..(ptr as usize) + (byte_len as usize))
            .ok_or_else(|| "Failed to get memory slice after".to_string())?;
        let dst_bytes = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, byte_len as usize)
        };
        dst_bytes.copy_from_slice(mem_slice_after);

        Ok(())
    }

    pub fn render_note_in_place(
        &self,
        wasm_bytes: &[u8],
        buffer: &mut [f32],
        synth_name: Option<&str>,
        freq: f32,
        amp: f32,
        duration_ms: i32,
        sample_rate: i32,
        channels: i32,
    ) -> Result<(), String> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| format!("Failed to compile wasm: {e}"))?;

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate wasm: {e}"))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| "WASM memory export not found".to_string())?;

        // Try specific function first
        let mut func_opt: Option<RenderFunc> = None;
        if let Some(name) = synth_name {
            let specific = format!("render_note_{}", name);
            if let Ok(f) = instance
                .get_typed_func::<(i32, i32, f32, f32, i32, i32, i32), ()>(&mut store, &specific)
            {
                func_opt = Some(f);
            }
        }
        if func_opt.is_none() {
            // fallback to generic name
            if let Ok(f) = instance.get_typed_func::<(i32, i32, f32, f32, i32, i32, i32), ()>(
                &mut store,
                "render_note",
            ) {
                func_opt = Some(f);
            }
        }

        let func =
            func_opt.ok_or_else(|| "Exported function `render_note` not found".to_string())?;

        // Copy host buffer into wasm memory
        let byte_len = std::mem::size_of_val(buffer) as i32;
        let ptr = Self::alloc_temp(&mut store, &instance, &memory, byte_len as usize)? as i32;
        let mem_slice = memory
            .data_mut(&mut store)
            .get_mut(ptr as usize..(ptr as usize) + (byte_len as usize))
            .ok_or_else(|| "Failed to get memory slice".to_string())?;
        let src_bytes =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, byte_len as usize) };
        mem_slice.copy_from_slice(src_bytes);

        // Call render
        func.call(
            &mut store,
            (
                ptr,
                buffer.len() as i32,
                freq,
                amp,
                duration_ms,
                sample_rate,
                channels,
            ),
        )
        .map_err(|e| format!("Error calling `render_note`: {e}"))?;

        // Copy back
        let mem_slice_after = memory
            .data(&store)
            .get(ptr as usize..(ptr as usize) + (byte_len as usize))
            .ok_or_else(|| "Failed to get memory slice after".to_string())?;
        let dst_bytes = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, byte_len as usize)
        };
        dst_bytes.copy_from_slice(mem_slice_after);

        Ok(())
    }

    /// Same as render_note_in_place, but first tries to call exported setters `set_<param>(f32)`
    /// for each provided param before rendering. Ignored if setter is missing.
    pub fn render_note_with_params_in_place(
        &self,
        wasm_bytes: &[u8],
        buffer: &mut [f32],
        synth_name: Option<&str>,
        freq: f32,
        amp: f32,
        duration_ms: i32,
        sample_rate: i32,
        channels: i32,
        params_num: &HashMap<String, f32>,
        params_str: Option<&HashMap<String, String>>,
    ) -> Result<(), String> {
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| format!("Failed to compile wasm: {e}"))?;

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::new(&self.engine);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("Failed to instantiate wasm: {e}"))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| "WASM memory export not found".to_string())?;

        // Call numeric setters if present: set_<param>(f32)
        for (k, v) in params_num.iter() {
            let fname = format!("set_{}", k);
            if let Ok(setter) = instance.get_typed_func::<f32, ()>(&mut store, &fname) {
                let _ = setter.call(&mut store, *v);
            }
        }

        // Call string setters if present: set_<param>_str(ptr: i32, len: i32)
        if let Some(smap) = params_str {
            for (k, v) in smap.iter() {
                let fname = format!("set_{}_str", k);
                if let Ok(setter) = instance.get_typed_func::<(i32, i32), ()>(&mut store, &fname) {
                    // Allocate and copy UTF-8 bytes into wasm memory
                    let bytes = v.as_bytes();
                    let ptr = Self::alloc_temp(&mut store, &instance, &memory, bytes.len())? as i32;
                    let mem_slice = memory
                        .data_mut(&mut store)
                        .get_mut(ptr as usize..(ptr as usize) + bytes.len())
                        .ok_or_else(|| "Failed to get memory slice for string".to_string())?;
                    mem_slice.copy_from_slice(bytes);
                    let _ = setter.call(&mut store, (ptr, bytes.len() as i32));
                }
            }
        }

        // Try specific or generic render function
        let mut func_opt: Option<RenderFunc> = None;
        if let Some(name) = synth_name {
            let specific = format!("render_note_{}", name);
            if let Ok(f) = instance
                .get_typed_func::<(i32, i32, f32, f32, i32, i32, i32), ()>(&mut store, &specific)
            {
                func_opt = Some(f);
            }
        }
        if func_opt.is_none() {
            if let Ok(f) = instance.get_typed_func::<(i32, i32, f32, f32, i32, i32, i32), ()>(
                &mut store,
                "render_note",
            ) {
                func_opt = Some(f);
            }
        }
        let func =
            func_opt.ok_or_else(|| "Exported function `render_note` not found".to_string())?;

        // Copy host buffer into wasm memory
        let byte_len = std::mem::size_of_val(buffer) as i32;
        let ptr = Self::alloc_temp(&mut store, &instance, &memory, byte_len as usize)? as i32;
        let mem_slice = memory
            .data_mut(&mut store)
            .get_mut(ptr as usize..(ptr as usize) + (byte_len as usize))
            .ok_or_else(|| "Failed to get memory slice".to_string())?;
        let src_bytes =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, byte_len as usize) };
        mem_slice.copy_from_slice(src_bytes);

        // Call render
        func.call(
            &mut store,
            (
                ptr,
                buffer.len() as i32,
                freq,
                amp,
                duration_ms,
                sample_rate,
                channels,
            ),
        )
        .map_err(|e| format!("Error calling `render_note`: {e}"))?;

        // Copy back
        let mem_slice_after = memory
            .data(&store)
            .get(ptr as usize..(ptr as usize) + (byte_len as usize))
            .ok_or_else(|| "Failed to get memory slice after".to_string())?;
        let dst_bytes = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, byte_len as usize)
        };
        dst_bytes.copy_from_slice(mem_slice_after);

        Ok(())
    }

    fn alloc_temp(
        store: &mut Store<()>,
        instance: &Instance,
        memory: &wasmtime::Memory,
        size: usize,
    ) -> Result<usize, String> {
        // Try to use an exported `__wbindgen_malloc` if present; otherwise, grow memory manually.
        if let Ok(malloc) = instance.get_typed_func::<i32, i32>(&mut *store, "__wbindgen_malloc") {
            let ptr = malloc
                .call(&mut *store, size as i32)
                .map_err(|e| format!("malloc failed: {e}"))? as usize;
            return Ok(ptr);
        }

        // Fallback: grow memory and use end of memory as scratch space
        let current_len = memory.data_size(&mut *store);
        let need = size;
        let pages_needed = (current_len + need).div_ceil(0x10000) as u64; // 64KiB pages
        let current_pages = memory.size(&mut *store);
        if pages_needed > current_pages {
            let to_grow = pages_needed - current_pages;
            memory
                .grow(&mut *store, to_grow)
                .map_err(|e| format!("memory.grow failed: {e}"))?;
        }
        Ok(current_len)
    }
}

// Provide a minimal stub for wasm32 target so the crate compiles there.
#[cfg(target_arch = "wasm32")]
pub struct WasmPluginRunner;

#[cfg(target_arch = "wasm32")]
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
    ) -> Result<(), String> {
        Err("Wasm plugin rendering is not available in wasm builds".to_string())
    }
}
