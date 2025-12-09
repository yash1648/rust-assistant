use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use rodio::{OutputStream, Sink, buffer::SamplesBuffer};

/// Python-backed Kokoro TTS (PyO3)
pub struct KokoroTts {
    engine: PyObject, // Py<PyAny>
}

impl KokoroTts {
    /// Initialize Python + KokoroEngine(speaker=...)
    pub fn new(speaker: &str) -> Result<Self> {
        Self::setup_pythonpath()?;

        Python::with_gil(|py| {
            // import python module kokoro_engine (your python file)
            let module = PyModule::import_bound(py, "kokoro-engine")?;
            let class = module.getattr("KokoroEngine")?;

            // KokoroEngine(speaker=...)
            let instance = class.call1((speaker,))?;

            Ok(Self {
                engine: instance.unbind(),
            })
        })
    }

    /// Generate speech and play it synchronously
    pub fn speak_and_play_blocking(&self, text: &str) -> Result<()> {
        let (pcm, sample_rate) = self.speak(text)?;
        self.play_pcm_blocking(pcm, sample_rate)
    }

    /// Call Python's engine.speak(text) -> (Vec<i16>, u32)
    fn speak(&self, text: &str) -> Result<(Vec<i16>, u32)> {
        Python::with_gil(|py| {
            let engine = self.engine.bind(py);
            let result = engine.call_method1("speak", (text,))?;
            let (pcm, sample_rate): (Vec<i16>, u32) = result.extract()?;
            Ok((pcm, sample_rate))
        })
    }

    /// Play mono i16 PCM samples synchronously
    fn play_pcm_blocking(&self, pcm: Vec<i16>, sample_rate: u32) -> Result<()> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let source = SamplesBuffer::new(1, sample_rate, pcm);
        sink.append(source);
        sink.sleep_until_end(); // block until done
        Ok(())
    }

    /// Make sure Python can find kokoro_engine.py
    fn setup_pythonpath() -> Result<()> {
        use std::env;
        use std::path::PathBuf;

        let py_dir: PathBuf = env::current_dir()?.join("src/pyengine");
        let py_dir_str = py_dir.to_string_lossy().into_owned();

        let sep = if cfg!(windows) { ';' } else { ':' };
        let current = env::var("PYTHONPATH").unwrap_or_default();

        let mut parts: Vec<String> = if current.is_empty() {
            Vec::new()
        } else {
            current
                .split(sep)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };

        if !parts.iter().any(|p| p == &py_dir_str) {
            parts.insert(0, py_dir_str);
        }

        let cwd_str = env::current_dir()?.to_string_lossy().into_owned();
        if !parts.iter().any(|p| p == &cwd_str) {
            parts.push(cwd_str);
        }

        let new_val = parts.join(&sep.to_string());
        // in your setup this may be unsafe; wrap if needed
        unsafe{
        std::env::set_var("PYTHONPATH", &new_val);
        }
        Ok(())
    }
}
