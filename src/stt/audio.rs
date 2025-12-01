use cpal::traits::{DeviceTrait, HostTrait};
use anyhow::{Context, Result};

pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: cpal::SampleFormat,
}

impl AudioConfig {
    pub fn from_device() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no default input device available")?;
        
        let config = device
            .default_input_config()
            .context("no default input config")?;
        
        println!("🎛 Input device: {}", device.name()?);
        println!("📡 Config: {:?}", config);
        
        Ok(Self {
            sample_rate: config.sample_rate().0,
            channels: config.channels() as u16,
            sample_format: config.sample_format(),
        })
    }
}