pub mod insert;
pub mod padding;

use devalang_types::Value;
use devalang_types::VariableTable;
use std::collections::HashMap;

impl super::driver::AudioEngine {
    pub fn insert_sample(
        &mut self,
        filepath: &str,
        time_secs: f32,
        dur_sec: f32,
        effects: Option<HashMap<String, Value>>,
        variable_table: &VariableTable,
    ) {
        crate::core::audio::engine::sample::insert::insert_sample_impl(
            self,
            filepath,
            time_secs,
            dur_sec,
            effects,
            variable_table,
        );
    }

    pub fn pad_samples(
        &mut self,
        samples: &[i16],
        time_secs: f32,
        effects_map: Option<HashMap<String, Value>>,
    ) {
        crate::core::audio::engine::sample::padding::pad_samples_impl(
            self,
            samples,
            time_secs,
            effects_map,
        );
    }
}
