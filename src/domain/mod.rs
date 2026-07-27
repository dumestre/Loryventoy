//! Tipos centrais do domínio, independentes da camada visual.

mod color;
mod project_config;
mod animation;

pub use color::Color;
pub use project_config::ProjectConfig;
pub use animation::{Easing, LoopMode, AnimSeg};

#[cfg(test)]
mod tests {
    use super::{Color, ProjectConfig};

    #[test]
    fn configuracao_padrao_preserva_contrato_atual() {
        let cfg = ProjectConfig::default();

        assert_eq!(cfg.largura, 1920);
        assert_eq!(cfg.altura, 1080);
        assert_eq!(cfg.fps, 30.0);
        assert_eq!(cfg.duracao_seg, 5.0);
        assert_eq!(cfg.fundo, Color::WHITE);
    }

    #[test]
    fn cor_preserva_rgba() {
        let cor = Color::from_rgba(10, 20, 30, 40);
        assert_eq!(cor.to_rgba(), [10, 20, 30, 40]);
    }
}
