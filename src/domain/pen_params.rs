use crate::domain::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct PenParams {
    pub cena: String,
    pub codigo: String,
    pub erro: Option<String>,
    pub cor: Color,
    pub cor_fill: Color,
    pub pos_x: f32,
    pub pos_y: f32,
    pub espessura: f32,
    pub preenchimento: bool,
    pub seed: f32,
    pub cantos: f32,
    pub ordem: f32,
    pub escala_x: f32,
    pub escala_y: f32,
    pub trim_inicio: f32,
    pub trim_fim: f32,
}
