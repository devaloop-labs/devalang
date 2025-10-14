// Native MIDI I/O helper (optional, enabled under CLI/native builds via 'midir')
// Responsibilities:
// - list available midi in/out ports
// - open a chosen in port and forward incoming messages to EventRegistry
// - provide a small API to send MIDI messages to out ports

use crate::engine::events::EventRegistry;
use crate::language::syntax::ast::Value;
#[cfg(feature = "cli")]
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::collections::HashMap;
#[cfg(feature = "cli")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "cli")]
pub struct MidiManager {
    // Keep connections alive
    in_connections: Vec<MidiInputConnection<()>>,
    out_connections: HashMap<String, MidiOutputConnection>,
    registry: Arc<Mutex<EventRegistry>>,
    start: Arc<std::time::Instant>,
}

#[cfg(feature = "cli")]
impl MidiManager {
    pub fn new(registry: Arc<Mutex<EventRegistry>>) -> Self {
        MidiManager {
            in_connections: Vec::new(),
            out_connections: HashMap::new(),
            registry,
            start: Arc::new(std::time::Instant::now()),
        }
    }

    pub fn list_input_ports() -> Vec<String> {
        if let Ok(midi_in) = MidiInput::new("devalang-in") {
            let ports = midi_in.ports();
            ports
                .iter()
                .map(|p| {
                    midi_in
                        .port_name(p)
                        .unwrap_or_else(|_| "unknown".to_string())
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn open_input_by_index(&mut self, index: usize, name: &str) -> Result<(), String> {
        let midi_in = MidiInput::new("devalang-in").map_err(|e| format!("midi_in: {}", e))?;
        // midi_in.ignore(Ignore::None);
        let ports = midi_in.ports();
        if index >= ports.len() {
            return Err("port index out of range".to_string());
        }
        let port = ports[index].clone();
        let registry = self.registry.clone();
        let device_name = name.to_string();
        let device_name_inner = device_name.clone();
        let start = self.start.clone();
        let conn_in = midi_in
            .connect(
                &port,
                &device_name,
                move |_stamp, message, _| {
                    // Parse simple NoteOn/NoteOff (0x9x / 0x8x) messages
                    if message[0] != 248 {
                        let status = message[0];
                        let cmd = status & 0xF0;
                        let channel = (status & 0x0F) as u8;
                        match cmd {
                            0x90 => {
                                let note = message[1] as u8;
                                let vel = message[2] as u8;
                                let mut data = HashMap::new();
                                data.insert("note".to_string(), Value::Number(note as f32));
                                data.insert("velocity".to_string(), Value::Number(vel as f32));
                                data.insert("channel".to_string(), Value::Number(channel as f32));
                                // Emit event name like mapping.in.<device>.noteOn (keyboard mode timestamp)
                                let event_name = format!("mapping.in.{}.noteOn", device_name_inner);
                                let elapsed = std::time::Instant::now().duration_since(*start);
                                let ts = elapsed.as_secs_f32();
                                if let Ok(mut reg) = registry.lock() {
                                    reg.emit(event_name, data, ts);
                                }
                            }
                            0x80 => {
                                let note = message[1] as u8;
                                let vel = message[2] as u8;
                                let mut data = HashMap::new();
                                data.insert("note".to_string(), Value::Number(note as f32));
                                data.insert("velocity".to_string(), Value::Number(vel as f32));
                                data.insert("channel".to_string(), Value::Number(channel as f32));
                                let event_name =
                                    format!("mapping.in.{}.noteOff", device_name_inner);
                                let elapsed = std::time::Instant::now().duration_since(*start);
                                let ts = elapsed.as_secs_f32();
                                if let Ok(mut reg) = registry.lock() {
                                    reg.emit(event_name, data, ts);
                                }
                            }
                            _ => {}
                        }
                    }
                },
                (),
            )
            .map_err(|e| format!("connect error: {}", e))?;

        self.in_connections.push(conn_in);
        Ok(())
    }

    pub fn open_output_by_name(&mut self, name: &str, index: usize) -> Result<(), String> {
        let midi_out = MidiOutput::new("devalang-out").map_err(|e| format!("midi_out: {}", e))?;
        let ports = midi_out.ports();
        if index >= ports.len() {
            return Err("port index out of range".to_string());
        }
        let port = &ports[index];
        let _port_name = midi_out
            .port_name(port)
            .unwrap_or_else(|_| "out".to_string());
        let conn_out = midi_out
            .connect(port, name)
            .map_err(|e| format!("connect out: {}", e))?;
        self.out_connections.insert(name.to_string(), conn_out);
        Ok(())
    }

    pub fn send_note_on(
        &mut self,
        device_name: &str,
        channel: u8,
        note: u8,
        vel: u8,
    ) -> Result<(), String> {
        if let Some(conn) = self.out_connections.get_mut(device_name) {
            let status = 0x90 | (channel & 0x0F);
            let _ = conn.send(&[status, note, vel]);
            Ok(())
        } else {
            Err("output connection not found".to_string())
        }
    }
}

#[cfg(not(feature = "cli"))]
pub struct MidiManager;

#[cfg(not(feature = "cli"))]
impl MidiManager {
    pub fn new(_registry: Arc<Mutex<EventRegistry>>) -> Self {
        MidiManager
    }
    pub fn list_input_ports() -> Vec<String> {
        Vec::new()
    }
    pub fn open_input_by_index(&mut self, _index: usize, _name: &str) -> Result<(), String> {
        Ok(())
    }
    pub fn open_output_by_name(&mut self, _name: &str, _index: usize) -> Result<(), String> {
        Ok(())
    }
    pub fn send_note_on(
        &mut self,
        _device_name: &str,
        _channel: u8,
        _note: u8,
        _vel: u8,
    ) -> Result<(), String> {
        Ok(())
    }
}
