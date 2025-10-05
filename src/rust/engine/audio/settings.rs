use std::fmt;

#[cfg(feature = "cli")]
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[cfg_attr(feature = "cli", clap(rename_all = "lower"))]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    Mid, // MIDI format
}

impl Default for AudioFormat {
    fn default() -> Self {
        AudioFormat::Mp3
    }
}

impl AudioFormat {
    pub fn label(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Wav => "wav",
            AudioFormat::Flac => "flac",
            AudioFormat::Mid => "mid",
        }
    }

    pub fn file_extension(self) -> &'static str {
        self.label()
    }

    /// Parse format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::Mp3),
            "wav" => Some(AudioFormat::Wav),
            "flac" => Some(AudioFormat::Flac),
            "mid" | "midi" => Some(AudioFormat::Mid),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[cfg_attr(feature = "cli", clap(rename_all = "lower"))]
pub enum AudioBitDepth {
    #[cfg_attr(feature = "cli", clap(alias = "8"))]
    Bit8,
    #[cfg_attr(feature = "cli", clap(alias = "16"))]
    Bit16,
    #[cfg_attr(feature = "cli", clap(alias = "24"))]
    Bit24,
    #[cfg_attr(feature = "cli", clap(alias = "32"))]
    Bit32,
}

impl Default for AudioBitDepth {
    fn default() -> Self {
        AudioBitDepth::Bit16
    }
}

impl AudioBitDepth {
    pub fn bits(self) -> u16 {
        match self {
            AudioBitDepth::Bit8 => 8,
            AudioBitDepth::Bit16 => 16,
            AudioBitDepth::Bit24 => 24,
            AudioBitDepth::Bit32 => 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[cfg_attr(feature = "cli", clap(rename_all = "lower"))]
pub enum AudioChannels {
    #[cfg_attr(feature = "cli", clap(alias = "1"))]
    Mono,
    #[cfg_attr(feature = "cli", clap(alias = "2"))]
    Stereo,
}

impl Default for AudioChannels {
    fn default() -> Self {
        AudioChannels::Stereo
    }
}

impl AudioChannels {
    pub fn count(self) -> u16 {
        match self {
            AudioChannels::Mono => 1,
            AudioChannels::Stereo => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[cfg_attr(feature = "cli", clap(rename_all = "kebab-case"))]
pub enum ResampleQuality {
    #[cfg_attr(feature = "cli", clap(alias = "linear"))]
    Linear2,
    #[cfg_attr(feature = "cli", clap(alias = "sinc-12"))]
    Sinc12,
    Sinc24,
    Sinc48,
    Sinc96,
    Sinc192,
    Sinc512,
}

impl Default for ResampleQuality {
    fn default() -> Self {
        ResampleQuality::Sinc24
    }
}

impl ResampleQuality {
    pub fn label(self) -> &'static str {
        match self {
            ResampleQuality::Linear2 => "2-point linear",
            ResampleQuality::Sinc12 => "12-point sinc",
            ResampleQuality::Sinc24 => "24-point sinc",
            ResampleQuality::Sinc48 => "48-point sinc",
            ResampleQuality::Sinc96 => "96-point sinc",
            ResampleQuality::Sinc192 => "192-point sinc",
            ResampleQuality::Sinc512 => "512-point sinc",
        }
    }
}

impl fmt::Display for ResampleQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
