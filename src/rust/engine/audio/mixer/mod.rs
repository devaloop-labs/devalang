use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub const MASTER_INSERT: &str = "master";

#[derive(Debug, Clone)]
pub struct SampleBuffer {
    data: Arc<Vec<f32>>,
    frames: usize,
    channels: usize,
    sample_rate: u32,
}

impl SampleBuffer {
    pub fn new(data: Arc<Vec<f32>>, channels: usize, sample_rate: u32) -> Self {
        let frames = if channels == 0 {
            0
        } else {
            data.len() / channels.max(1)
        };
        Self {
            data,
            frames,
            channels: channels.max(1),
            sample_rate,
        }
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn sample_channel(&self, frame: usize, channel: usize, target_channels: usize) -> f32 {
        if self.frames == 0 {
            return 0.0;
        }
        let sample_channels = self.channels.max(1);
        let source_channel = match (sample_channels, target_channels) {
            (1, _) => 0,
            (2, 1) => {
                let left = self.data[frame * sample_channels];
                let right = self.data[frame * sample_channels + 1];
                return (left + right) * 0.5;
            }
            _ => channel % sample_channels,
        };
        self.data[frame * sample_channels + source_channel]
    }

    /// Create a new SampleBuffer with modified data (for effects processing)
    pub fn with_modified_data(&self, new_data: Vec<f32>, new_channels: Option<usize>) -> Self {
        let channels = new_channels.unwrap_or(self.channels);
        Self::new(Arc::new(new_data), channels, self.sample_rate)
    }

    /// Get a mutable copy of the internal data for processing
    pub fn data_clone(&self) -> Vec<f32> {
        self.data.as_ref().clone()
    }
}

#[derive(Debug, Clone)]
struct AudioInsert {
    name: String,
    parent: Option<String>,
    buffer: Vec<f32>,
}

impl AudioInsert {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent: None,
            buffer: Vec::new(),
        }
    }

    fn set_parent(&mut self, parent: Option<String>) {
        if self.name == MASTER_INSERT {
            self.parent = None;
        } else {
            self.parent = parent;
        }
    }

    fn ensure_frames(&mut self, frames: usize, channels: usize) {
        let required = frames.saturating_mul(channels);
        if self.buffer.len() < required {
            self.buffer.resize(required, 0.0);
        }
    }
}

#[derive(Debug)]
pub struct AudioMixer {
    sample_rate: u32,
    channels: usize,
    inserts: HashMap<String, AudioInsert>,
}

impl AudioMixer {
    pub fn new(sample_rate: u32, channels: usize) -> Self {
        let mut inserts = HashMap::new();
        inserts.insert(MASTER_INSERT.to_string(), AudioInsert::new(MASTER_INSERT));
        Self {
            sample_rate,
            channels: channels.max(1),
            inserts,
        }
    }

    pub fn register_insert(&mut self, name: impl Into<String>, parent: Option<&str>) -> String {
        let key = name.into();
        if key != MASTER_INSERT {
            if let Some(parent_name) = parent {
                if !self.inserts.contains_key(parent_name) {
                    self.register_insert(parent_name.to_string(), None);
                }
            }
        }
        use std::collections::hash_map::Entry;
        match self.inserts.entry(key.clone()) {
            Entry::Occupied(mut entry) => {
                if key != MASTER_INSERT {
                    if let Some(parent_name) = parent {
                        if entry.get().parent.as_deref() != Some(parent_name) {
                            entry.get_mut().set_parent(Some(parent_name.to_string()));
                        }
                    }
                }
            }
            Entry::Vacant(slot) => {
                let mut insert = AudioInsert::new(&key);
                if key != MASTER_INSERT {
                    insert.set_parent(parent.map(|p| p.to_string()));
                }
                slot.insert(insert);
            }
        }
        key
    }

    pub fn ensure_master_frames(&mut self, frames: usize) {
        if let Some(master) = self.inserts.get_mut(MASTER_INSERT) {
            master.ensure_frames(frames, self.channels);
        }
    }

    pub fn mix_sample(
        &mut self,
        insert: &str,
        start_frame: usize,
        duration: f32,
        sample: &SampleBuffer,
    ) {
        if sample.frames() == 0 {
            return;
        }
        let route = self.route_chain(insert);
        for route_insert in route {
            if !self.inserts.contains_key(&route_insert) {
                // Unknown insert; skip mixing to avoid panics.
                continue;
            }
            if let Some(target) = self.inserts.get_mut(&route_insert) {
                Self::mix_into_insert(
                    target,
                    self.channels,
                    start_frame,
                    duration,
                    self.sample_rate,
                    sample,
                );
            }
        }
    }

    pub fn into_master_buffer(mut self, total_frames: usize) -> Vec<f32> {
        let samples = total_frames.saturating_mul(self.channels);
        self.ensure_master_frames(total_frames);
        let mut master = self
            .inserts
            .remove(MASTER_INSERT)
            .unwrap_or_else(|| AudioInsert::new(MASTER_INSERT));
        if samples == 0 {
            master.buffer.clear();
            return master.buffer;
        }
        if master.buffer.len() < samples {
            master.buffer.resize(samples, 0.0);
        } else if master.buffer.len() > samples {
            master.buffer.truncate(samples);
        }
        master.buffer
    }

    pub fn sanitize_label(label: &str) -> String {
        label
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn route_chain(&mut self, insert: &str) -> Vec<String> {
        if !self.inserts.contains_key(insert) {
            self.register_insert(insert.to_string(), Some(MASTER_INSERT));
        }
        let mut chain = Vec::new();
        let mut current = insert.to_string();
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(current.clone()) {
                break;
            }
            chain.push(current.clone());
            if current == MASTER_INSERT {
                break;
            }
            let parent = self
                .inserts
                .get(&current)
                .and_then(|insert| insert.parent.as_deref())
                .unwrap_or(MASTER_INSERT);
            current = parent.to_string();
        }
        if !chain.iter().any(|name| name == MASTER_INSERT) {
            chain.push(MASTER_INSERT.to_string());
        }
        chain
    }

    fn mix_into_insert(
        insert: &mut AudioInsert,
        channel_count: usize,
        start_frame: usize,
        duration: f32,
        output_rate: u32,
        sample: &SampleBuffer,
    ) {
        let channel_count = channel_count.max(1);
        let mut max_play_frames = (duration * output_rate as f32).ceil() as usize;
        if max_play_frames == 0 {
            let scaled =
                (sample.frames() as f32 * output_rate as f32) / sample.sample_rate() as f32;
            max_play_frames = scaled.ceil() as usize;
        }
        if max_play_frames == 0 {
            return;
        }
        let required_frames = start_frame.saturating_add(max_play_frames);
        insert.ensure_frames(required_frames, channel_count);
        let ratio = if output_rate == 0 {
            1.0
        } else {
            sample.sample_rate() as f32 / output_rate as f32
        };
        for frame_idx in 0..max_play_frames {
            let buffer_frame = start_frame + frame_idx;
            let sample_pos = frame_idx as f32 * ratio;
            if sample_pos >= sample.frames() as f32 {
                break;
            }
            let base = sample_pos.floor() as usize;
            let next = if base + 1 >= sample.frames() {
                sample.frames().saturating_sub(1)
            } else {
                base + 1
            };
            let frac = sample_pos - base as f32;
            for ch in 0..channel_count {
                let v0 = sample.sample_channel(base, ch, channel_count);
                let v1 = sample.sample_channel(next, ch, channel_count);
                let interpolated = v0 + (v1 - v0) * frac;
                let buffer_index = buffer_frame * channel_count + ch;
                if let Some(slot) = insert.buffer.get_mut(buffer_index) {
                    *slot += interpolated;
                }
            }
        }
    }
}
