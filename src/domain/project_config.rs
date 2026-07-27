use super::Color;

/// Configuração global do projeto, sem dependência de interface gráfica.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectConfig {
    pub largura: u32,
    pub altura: u32,
    pub fps: f32,
    pub duracao_seg: f32,
    pub fundo: Color,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            largura: 1920,
            altura: 1080,
            fps: 30.0,
            duracao_seg: 5.0,
            fundo: Color::WHITE,
        }
    }
}
