#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Step,
}

impl Easing {
    pub fn from_u8(v: u8) -> Easing {
        match v {
            1 => Easing::EaseIn,
            2 => Easing::EaseOut,
            3 => Easing::EaseInOut,
            4 => Easing::Step,
            _ => Easing::Linear,
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            Easing::Linear => 0,
            Easing::EaseIn => 1,
            Easing::EaseOut => 2,
            Easing::EaseInOut => 3,
            Easing::Step => 4,
        }
    }
    pub fn aplicar(self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Easing::Linear => x,
            Easing::EaseIn => x * x,
            Easing::EaseOut => 1.0 - (1.0 - x) * (1.0 - x),
            Easing::EaseInOut => {
                if x < 0.5 {
                    2.0 * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
                }
            }
            Easing::Step => {
                if x >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    Nenhum,
    Repetir,
    PingPong,
}

impl LoopMode {
    pub fn from_u8(v: u8) -> LoopMode {
        match v {
            1 => LoopMode::Repetir,
            2 => LoopMode::PingPong,
            _ => LoopMode::Nenhum,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimSeg {
    pub t_ini: f32,
    pub t_fim: f32,
    pub v_ini: [f32; 2],
    pub v_fim: [f32; 2],
    pub easing: Easing,
}
