#[cfg(feature = "cli")]
use wasmtime::{Engine, Instance, Linker, Module, Store};
use std::collections::HashMap;
use std::cell::RefCell;

#[cfg(feature = "cli")]
pub struct WasmPluginRunner {
    engine: Engine,
    // Cache instance par hash de WASM pour réutiliser le state
    cache: RefCell<HashMap<u64, (Store<()>, Instance)>>,
}

#[cfg(feature = "cli")]
impl Default for WasmPluginRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli")]
impl WasmPluginRunner {
    pub fn new() -> Self {
        let engine = Engine::default();
        Self { 
            engine,
            cache: RefCell::new(HashMap::new()),
        }
    }
    
    /// Renders a note using a WASM plugin with optional parameter overrides
    /// 
    /// Tries multiple function name patterns in order:
    /// 1. Named export (if synth_name provided): e.g. "synth", "saw"
    /// 2. Generic "render_note"
    pub fn render_note_in_place(
        &self,
        wasm_bytes: &[u8],
        buffer: &mut [f32],
        instance_key: Option<&str>,  // ← NEW: For cache hash ("acidScreamer" / "acidSquare")
        synth_name: Option<&str>,    // ← Function to call ("synth")
        freq: f32,
        amp: f32,
        duration_ms: i32,
        sample_rate: i32,
        channels: i32,
        options: Option<&HashMap<String, f32>>,
    ) -> Result<(), String> {
        // Hash du WASM + instance_key pour avoir une instance par synth!
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        wasm_bytes.hash(&mut hasher);
        instance_key.hash(&mut hasher); // Use instance_key (synth_id) for caching!
        let hash = hasher.finish();
        
        // Borrow mutable du cache
        let mut cache = self.cache.borrow_mut();
        
        // Créer l'instance si elle n'existe pas
        if !cache.contains_key(&hash) {
            // Creating new instance for synth_id
            let module = Module::new(&self.engine, wasm_bytes)
                .map_err(|e| format!("Failed to compile wasm: {e}"))?;
            
            let mut store = Store::new(&self.engine, ());
            let mut linker = Linker::new(&self.engine);
            
            // Add wasm-bindgen placeholder imports (for plugins compiled with wasm-bindgen)
            // These are stub implementations that do nothing but allow the WASM to load
            linker.func_wrap("__wbindgen_placeholder__", "__wbindgen_describe", |_: i32| {})
                .map_err(|e| format!("Failed to define wbindgen import: {e}"))?;
            linker.func_wrap("__wbindgen_placeholder__", "__wbindgen_object_clone_ref", |_: i32| -> i32 { 0 })
                .map_err(|e| format!("Failed to define wbindgen import: {e}"))?;
            linker.func_wrap("__wbindgen_placeholder__", "__wbindgen_object_drop_ref", |_: i32| {})
                .map_err(|e| format!("Failed to define wbindgen import: {e}"))?;
            linker.func_wrap("__wbindgen_placeholder__", "__wbindgen_string_new", |_: i32, _: i32| -> i32 { 0 })
                .map_err(|e| format!("Failed to define wbindgen import: {e}"))?;
            linker.func_wrap("__wbindgen_placeholder__", "__wbindgen_throw", |_: i32, _: i32| {})
                .map_err(|e| format!("Failed to define wbindgen import: {e}"))?;
            
            let instance = linker
                .instantiate(&mut store, &module)
                .map_err(|e| format!("Failed to instantiate wasm: {e}"))?;
            
            cache.insert(hash, (store, instance));
        } else {
            // Reusing cached instance for synth_id
        }
        
        // Récupérer l'instance cachée
        let entry = cache.get_mut(&hash).unwrap();
        
        let memory = entry.1
            .get_memory(&mut entry.0, "memory")
            .ok_or_else(|| "WASM memory export not found".to_string())?;
        
        // Find the right function to call
        let func_name = if let Some(name) = synth_name {
            // Try the named export first
            if entry.1.get_func(&mut entry.0, name).is_some() {
                name
            } else if entry.1.get_func(&mut entry.0, "render_note").is_some() {
                "render_note"
            } else {
                return Err(format!("Plugin export '{}' not found", name));
            }
        } else {
            "render_note"
        };
        
        let func = entry.1
            .get_typed_func::<(i32, i32, f32, f32, i32, i32, i32), ()>(&mut entry.0, func_name)
            .map_err(|e| format!("Function '{}' not found or wrong signature: {}", func_name, e))?;
        
        // Apply plugin options by calling setter functions if available
        if let Some(opts) = options {
            // Applying plugin options
            
            // Map of parameter names to setter functions
            let setter_map: HashMap<&str, &str> = [
                ("waveform", "setWaveform"),
                ("cutoff", "setCutoff"),
                ("resonance", "setResonance"),
                ("env_mod", "setEnvMod"),
                ("decay", "setDecay"),
                ("accent", "setAccent"),
                ("drive", "setDrive"),
                ("tone", "setTone"),
                ("slide", "setSlide"),
                ("glide", "setGlide"),
            ].iter().cloned().collect();
            
            for (key, value) in opts.iter() {
                // Try to find matching setter
                if let Some(setter_name) = setter_map.get(key.as_str()) {
                    // Try to get the setter function
                    if let Ok(setter) = entry.1.get_typed_func::<f32, ()>(&mut entry.0, setter_name) {
                        // Call setter with the parameter value
                        // Setting option
                        let _ = setter.call(&mut entry.0, *value);
                    } else {
                        // Setter not found
                    }
                } else {
                    // Unknown parameter
                }
            }
        } else {
            // No plugin options provided
        }
        
        // Allocate memory in WASM for the buffer
        let byte_len = std::mem::size_of_val(buffer);
        let ptr = Self::alloc_temp(&mut entry.0, &entry.1, &memory, byte_len)? as i32;
        
    // Calling plugin with parameters (log removed)
        
        // Copy buffer into WASM memory
        let mem_slice = memory
            .data_mut(&mut entry.0)
            .get_mut(ptr as usize..(ptr as usize) + byte_len)
            .ok_or_else(|| "Failed to get memory slice".to_string())?;
        let src_bytes = unsafe { 
            std::slice::from_raw_parts(buffer.as_ptr() as *const u8, byte_len) 
        };
        mem_slice.copy_from_slice(src_bytes);
        
        // Call the plugin function
        func.call(
            &mut entry.0,
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
        .map_err(|e| format!("Error calling '{}': {}", func_name, e))?;
        
        // Copy result back from WASM memory
        let mem_slice_after = memory
            .data(&entry.0)
            .get(ptr as usize..(ptr as usize) + byte_len)
            .ok_or_else(|| "Failed to get memory slice after".to_string())?;
        let dst_bytes = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, byte_len)
        };
        dst_bytes.copy_from_slice(mem_slice_after);
        
    // Debug: Check if buffer has any non-zero values
    let _non_zero = buffer.iter().any(|&x| x.abs() > 0.0001);
    let _max_val = buffer.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    // Plugin result diagnostics
        
        Ok(())
    }
    
    /// Helper to allocate temporary memory in WASM module
    fn alloc_temp(
        store: &mut Store<()>,
        _instance: &Instance,
        memory: &wasmtime::Memory,
        size: usize,
    ) -> Result<i32, String> {
        // Simple bump allocator: just use end of current memory
        let pages = memory.size(&*store);
        let current_size = pages * 65536; // WASM page size
        let ptr = current_size as i32;
        
        // Grow memory if needed
        let needed_pages = ((size + 65535) / 65536) as u64;
        if needed_pages > 0 {
            memory
                .grow(store, needed_pages)
                .map_err(|e| format!("Failed to grow memory: {}", e))?;
        }
        
        Ok(ptr)
    }
}
