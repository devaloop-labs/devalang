use std::{ collections::HashMap, fs::File, io::BufReader };
use hound::{ SampleFormat, WavSpec, WavWriter };
use rodio::{ Decoder, Source };

use crate::core::{ store::variable::VariableTable, utils::path::normalize_path };

const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct AudioEngine {
    pub volume: f32,
    pub variables: VariableTable,
    pub buffer: Vec<i16>,
}

impl AudioEngine {
    pub fn new() -> Self {
        AudioEngine {
            volume: 1.0,
            buffer: vec![],
            variables: VariableTable::new(),
        }
    }

    pub fn mix(&mut self, other: &AudioEngine) {
        let max_len = self.buffer.len().max(other.buffer.len());
        self.buffer.resize(max_len, 0);

        for (i, &sample) in other.buffer.iter().enumerate() {
            self.buffer[i] = self.buffer[i].saturating_add(sample);
        }
    }

    pub fn set_duration(&mut self, duration_secs: f32) {
        let mut total_samples = (duration_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;

        if total_samples % (CHANNELS as usize) != 0 {
            total_samples += 1;
        }

        self.buffer.resize(total_samples, 0);
    }

    pub fn set_variables(&mut self, variables: VariableTable) {
        self.variables = variables;
    }

    pub fn generate_wav_file(&mut self, output_dir: &String) -> Result<(), String> {
        if self.buffer.len() % (CHANNELS as usize) != 0 {
            self.buffer.push(0);
            println!("Completed buffer to respect stereo format.");
        }

        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_dir, spec).map_err(|e|
            format!("Error creating WAV file: {}", e)
        )?;

        for sample in &self.buffer {
            writer.write_sample(*sample).map_err(|e| format!("Error writing sample: {:?}", e))?;
        }

        writer.finalize().map_err(|e| format!("Error finalizing WAV: {:?}", e))?;

        Ok(())
    }

    pub fn insert(
        &mut self,
        filepath: &str,
        time_secs: f32,
        dur_sec: f32,
        effects: Option<HashMap<String, f32>>
    ) {
        let normalized_filepath = normalize_path(filepath);

        let file = BufReader::new(
            File::open(normalized_filepath).expect("Failed to open audio file")
        );
        let decoder = Decoder::new(file).expect("Failed to decode audio file");

        // Mono or stereo reading possible here, we will duplicate in L/R
        let max_mono_samples = (dur_sec * (SAMPLE_RATE as f32)) as usize;
        let samples: Vec<i16> = decoder.convert_samples().take(max_mono_samples).collect();

        if samples.is_empty() {
            eprintln!("No samples found in the audio file: {}", filepath);
            return;
        }

        // TODO Apply effects here if needed
        let offset = (time_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;
        let required_len = offset + samples.len() * (CHANNELS as usize);
        let padded_required_len = if required_len % 2 == 1 {
            required_len + 1
        } else {
            required_len
        };

        self.buffer.resize(padded_required_len, 0);
        self.pad_samples(&samples, time_secs);
    }

    fn pad_samples(&mut self, samples: &[i16], time_secs: f32) {
        let offset = (time_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;

        for (i, &sample) in samples.iter().enumerate() {
            let adjusted_sample = ((sample as f32) * self.volume).round() as i16;

            let left_pos = offset + i * 2;
            let right_pos = left_pos + 1;

            if right_pos < self.buffer.len() {
                self.buffer[left_pos] = self.buffer[left_pos].saturating_add(adjusted_sample); // gauche
                self.buffer[right_pos] = self.buffer[right_pos].saturating_add(adjusted_sample); // droite
            }
        }
    }
}
