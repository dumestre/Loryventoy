use crate::domain::Color;

/// Parâmetros persistentes do nó Shape.
#[derive(Clone, Debug)]
pub struct ShapeParams {
    pub cena: String,
    pub tipo: u8,
    pub px: f32,
    pub py: f32,
    pub largura: f32,
    pub altura: f32,
    pub rotacao: f32,
    pub cor: Color,
    pub seed: f32,
    pub noise_scale: f32,
    pub amp: f32,
    pub veloc: f32,
    pub trim_inicio: f32,
    pub trim_fim: f32,
}
