/// Estado de reprodução (playback) independente do app principal.
/// O `App` apenas coleta o tempo do `egui` e chama o serviço de playback.

pub struct PlaybackState {
    pub playing: bool,
    pub was_playing: bool,
    pub fps: f32,
    pub frame_accum: f32,
    pub last_time: f64,
}

impl PlaybackState {
    pub fn new() -> Self {
        Self {
            playing: false,
            was_playing: false,
            fps: 24.0,
            frame_accum: 0.0,
            last_time: 0.0,
        }
    }

    /// Atualiza o estado de reprodução e avança o frame se necessário.
    /// Retorna o número de frames para avançar (0 se não houver aviso).
    pub fn update(
        &mut self,
        now: f64,
        is_playing: bool,
    ) -> u32 {
        if !is_playing && self.was_playing {
            self.last_time = now;
            self.frame_accum = 0.0;
        }
        self.was_playing = is_playing;
        self.playing = is_playing;

        if is_playing {
            let dt = (now - self.last_time) as f32;
            self.last_time = now;
            self.frame_accum += dt * self.fps;
            let advance = self.frame_accum.floor() as u32;
            self.frame_accum -= advance as f32;
            advance
        } else {
            0
        }
    }
}