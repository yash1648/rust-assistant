from kokoro import KPipeline
import numpy as np
import torch

def get_best_device():
    if torch.cuda.is_available():
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")

class KokoroEngine:
    def __init__(
        self,
        speaker="bf_emma",
        lang_code="a",
        sample_rate=24000,
        speed=1.0
    ):
        self.device = get_best_device()
        print("▶ Using device:", self.device)

        # Pipeline uses PyTorch under the hood → auto GPU
        self.pipeline = KPipeline(lang_code=lang_code)

        self.speaker = speaker
        self.sample_rate = sample_rate
        self.speed = speed

        # Warm-up generation
        _ = next(self.pipeline("warming up...", voice=self.speaker, speed=self.speed), None)

    def speak(self, text: str):
        audios = []
        for _, _, audio in self.pipeline(text, voice=self.speaker, speed=self.speed):
            audios.append(audio)

        if not audios:
            return [], self.sample_rate

        audio = np.concatenate(audios)
        pcm = (audio * 32767.0).clip(-32768, 32767).astype(np.int16)
        return pcm.tolist(), self.sample_rate


# if __name__=="__main__":
#     eng=KokoroEngine()
#     pcm,sr=eng.speak("Hello ");
#     print(len(pcm),sr)