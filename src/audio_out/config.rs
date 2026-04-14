//! Audio output configuration.

use serde::{Deserialize, Serialize};

/// Configuration for the audio output engine.
///
/// Sample rate and buffer size are hints — when the cpal stream is opened,
/// the actual device may negotiate different values. See `AudioOutEngine`
/// for details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Requested sample rate in Hz. Default 48000.
    pub sample_rate: u32,
    /// Requested buffer size in samples per channel. Default 256.
    pub buffer_size: u32,
    /// Number of output channels. Stereo (2) in v1.
    pub channels: u16,
    /// Target device identifier (cpal device name). `None` = system default.
    pub device_id: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            buffer_size: 256,
            channels: 2,
            device_id: None,
        }
    }
}

impl AudioConfig {
    /// Returns a new config with the given device id.
    pub fn with_device(mut self, id: String) -> Self {
        self.device_id = Some(id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AudioConfig::default();
        assert_eq!(cfg.sample_rate, 48_000);
        assert_eq!(cfg.buffer_size, 256);
        assert_eq!(cfg.channels, 2);
        assert_eq!(cfg.device_id, None);
    }

    #[test]
    fn test_config_with_device() {
        let cfg = AudioConfig::default().with_device("MacBook Pro Speakers".to_string());
        assert_eq!(cfg.device_id.as_deref(), Some("MacBook Pro Speakers"));
    }
}
