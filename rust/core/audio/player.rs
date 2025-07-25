use rodio::{ Decoder, OutputStream, OutputStreamHandle, Sink, Source };
use std::{ fs::File, io::BufReader };

pub struct AudioPlayer {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    last_path: Option<String>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();

        Self {
            _stream: stream,
            handle,
            sink,
            last_path: None,
        }
    }

    fn load_source(&self, path: &str) -> Option<impl Source<Item = f32> + Send + 'static> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            match Decoder::new(reader) {
                Ok(decoder) => Some(decoder.convert_samples()),
                Err(e) => {
                    eprintln!("❌ Failed to decode audio file '{}': {}", path, e);
                    None
                }
            }
        } else {
            eprintln!("❌ Could not open audio file: {}", path);
            None
        }
    }

    pub fn play_file_once(&mut self, path: &str) {
        self.sink.stop();
        self.sink = Sink::try_new(&self.handle).unwrap();
        self.sink.set_volume(1.0);

        if let Some(source) = self.load_source(path) {
            self.sink.append(source);
            self.last_path = Some(path.to_string());
        } else {
            eprintln!("⚠️ Skipping playback: failed to load '{}'", path);
        }
    }

    pub fn replay_last(&mut self) {
        if let Some(path) = self.last_path.clone() {
            self.play_file_once(&path);
        } else {
            eprintln!("⚠️ No previous audio to replay.");
        }
    }

    pub fn wait_until_end(&self) {
        self.sink.sleep_until_end();
    }
}
