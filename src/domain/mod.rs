//! Tipos centrais do domínio, independentes da camada visual.

mod color;
mod project_config;
mod animation;
mod node_type;
mod layer_entry;

// ── Parâmetros de nós ────────────────────────────────────────────
mod transform_params;
mod cena_params;
mod layer_params;
mod text_params;
mod shape_params;
mod pen_params;
mod ruido_params;
mod saida_params;
mod anim_params;
mod params;

// ── Projeto ──────────────────────────────────────────────────────
mod project;

pub use color::Color;
pub use project_config::ProjectConfig;
pub use animation::{Easing, LoopMode, AnimSeg};
pub use node_type::TipoNo;
pub use layer_entry::LayerEntry;

pub use transform_params::TransformParams;
pub use cena_params::CenaParams;
pub use layer_params::LayerParams;
pub use text_params::TextParams;
pub use shape_params::ShapeParams;
pub use pen_params::PenParams;
pub use ruido_params::RuidoParams;
pub use saida_params::SaidaParams;
pub use anim_params::AnimParams;
pub use params::NodeParams;

pub use project::{Project, ProjectNode, ProjectEdge};

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
