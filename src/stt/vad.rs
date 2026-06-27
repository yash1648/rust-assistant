//! Energy-based Voice Activity Detection (VAD).
//!
//! Uses RMS (Root Mean Square) thresholding to detect speech vs silence.
//! No external model, no system dependencies — pure signal processing.
//!
//! When audio energy drops below the threshold for a configurable duration,
//! VAD signals that recording should stop.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Configuration for the energy-based VAD
#[derive(Debug, Clone)]
pub struct VadConfig {
    /// RMS threshold (0.0 – 1.0, as fraction of i16::MAX).
    /// Values below this are considered silence.
    /// Default: 0.02 (~2% of max amplitude, suitable for quiet environments).
    pub threshold: f32,

    /// How many consecutive silent frames trigger stop (each frame = 1 VAD update).
    /// Default: 40 frames (= 800ms at 20ms per frame).
    pub max_silent_frames: u64,

    /// Sample rate for frame timing calculation (Hz).
    /// Default: 16000 (Whisper native rate).
    pub sample_rate: u32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            threshold: 0.02,
            max_silent_frames: 40,  // 800ms
            sample_rate: 16000,
        }
    }
}

impl VadConfig {
    /// Duration per frame in milliseconds (based on a fixed 20ms frame size).
    pub fn frame_ms(&self) -> u64 {
        // Each "check" happens every ~20ms of audio
        (self.sample_rate / 50) as u64 // 20ms worth of samples
    }

    /// Total silence duration before stop (ms).
    pub fn silence_timeout_ms(&self) -> u64 {
        self.max_silent_frames * 20 // 20ms per frame
    }
}

/// Shared VAD state accessible from both the audio callback and monitor thread.
pub struct VadState {
    /// Whether VAD has detected enough silence to stop recording.
    pub should_stop: AtomicBool,
    /// Count of consecutive frames below threshold.
    pub silent_frames: AtomicU64,
    /// Configuration (read-only after init).
    pub config: VadConfig,
}

impl VadState {
    pub fn new(config: VadConfig) -> Self {
        Self {
            should_stop: AtomicBool::new(false),
            silent_frames: AtomicU64::new(0),
            config,
        }
    }

    /// Process an audio buffer: compute RMS and update silent frame counter.
    /// Called from the audio callback (must be fast, no blocking).
    pub fn process_audio(&self, samples: &[i16]) {
        if self.should_stop.load(Ordering::Relaxed) {
            return;
        }

        let rms = compute_rms_i16(samples);
        let is_silent = rms < self.config.threshold;

        if is_silent {
            let frames = self.silent_frames.fetch_add(1, Ordering::Relaxed);
            if frames + 1 >= self.config.max_silent_frames {
                self.should_stop.store(true, Ordering::Release);
            }
        } else {
            self.silent_frames.store(0, Ordering::Relaxed);
        }
    }

    /// Process f32 audio samples (convert to i16 range then compute).
    pub fn process_audio_f32(&self, samples: &[f32]) {
        if self.should_stop.load(Ordering::Relaxed) {
            return;
        }

        // Compute RMS on f32 samples directly (range -1.0 to 1.0)
        let rms = compute_rms_f32(samples);
        let is_silent = rms < self.config.threshold;

        if is_silent {
            let frames = self.silent_frames.fetch_add(1, Ordering::Relaxed);
            if frames + 1 >= self.config.max_silent_frames {
                self.should_stop.store(true, Ordering::Release);
            }
        } else {
            self.silent_frames.store(0, Ordering::Relaxed);
        }
    }

    /// Check if VAD has triggered stop (non-blocking).
    pub fn is_stopped(&self) -> bool {
        self.should_stop.load(Ordering::Acquire)
    }

    /// Reset VAD state for a new recording session.
    pub fn reset(&self) {
        self.should_stop.store(false, Ordering::Release);
        self.silent_frames.store(0, Ordering::Relaxed);
    }
}

/// Compute RMS (Root Mean Square) of i16 audio samples as a fraction of i16::MAX.
fn compute_rms_i16(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| {
        let f = s as f32 / i16::MAX as f32;
        f * f
    }).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Compute RMS of f32 audio samples (range -1.0 to 1.0).
fn compute_rms_f32(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rms_silence() {
        // All zeros → RMS = 0
        let samples = vec![0i16; 1000];
        let rms = compute_rms_i16(&samples);
        assert!(rms.abs() < 1e-6, "silence RMS should be ~0, got {}", rms);
    }

    #[test]
    fn test_rms_full_scale() {
        // Full scale square wave → RMS = 1.0
        let samples = vec![i16::MAX; 1000];
        let rms = compute_rms_i16(&samples);
        assert!((rms - 1.0).abs() < 0.01, "full-scale RMS should be ~1.0, got {}", rms);
    }

    #[test]
    fn test_rms_half_scale() {
        let samples = vec![i16::MAX / 2; 1000];
        let rms = compute_rms_i16(&samples);
        assert!((rms - 0.5).abs() < 0.01, "half-scale RMS should be ~0.5, got {}", rms);
    }

    #[test]
    fn test_rms_f32_silence() {
        let samples = vec![0.0f32; 1000];
        let rms = compute_rms_f32(&samples);
        assert!(rms.abs() < 1e-6);
    }

    #[test]
    fn test_vad_state_silence_triggers_stop() {
        let config = VadConfig {
            threshold: 0.1,
            max_silent_frames: 3,
            sample_rate: 16000,
        };
        let state = VadState::new(config);

        // Process silent audio — should eventually trigger stop
        let silent = vec![0i16; 160]; // 10ms at 16kHz
        for _ in 0..3 {
            state.process_audio(&silent);
        }
        assert!(state.is_stopped(), "VAD should trigger after 3 silent frames");
    }

    #[test]
    fn test_vad_state_speech_resets_counter() {
        let config = VadConfig {
            threshold: 0.1,
            max_silent_frames: 5,
            sample_rate: 16000,
        };
        let state = VadState::new(config);

        let silent = vec![0i16; 160];
        let speech = vec![i16::MAX / 2; 160];

        // 3 silent frames
        for _ in 0..3 {
            state.process_audio(&silent);
        }
        assert!(!state.is_stopped(), "3 < 5 silent frames should not stop");

        // Speech resets counter
        state.process_audio(&speech);
        assert_eq!(state.silent_frames.load(Ordering::Relaxed), 0,
            "speech should reset silent frame counter");

        // Now 5 silent frames
        for _ in 0..5 {
            state.process_audio(&silent);
        }
        assert!(state.is_stopped(), "5 silent frames after speech should stop");
    }

    #[test]
    fn test_vad_reset() {
        let config = VadConfig {
            threshold: 0.1,
            max_silent_frames: 3,
            sample_rate: 16000,
        };
        let state = VadState::new(config);

        let silent = vec![0i16; 160];
        for _ in 0..3 {
            state.process_audio(&silent);
        }
        assert!(state.is_stopped());

        state.reset();
        assert!(!state.is_stopped(), "reset should clear stop flag");
        assert_eq!(state.silent_frames.load(Ordering::Relaxed), 0,
            "reset should clear silent frame counter");
    }
}
