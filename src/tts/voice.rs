// src/tts/voice.rs
#[derive(Debug)]
pub struct Voice {
    pub id: &'static str,
    pub model_url: &'static str,
    pub config_url: &'static str,
}

pub fn voices() -> Vec<Voice> {
    vec![
        Voice {
            id: "en_GB-cori-high",
            model_url: "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/cori/high/en_GB-cori-high.onnx",
            config_url: "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/cori/high/en_GB-cori-high.onnx.json",
        },
    ]
}
