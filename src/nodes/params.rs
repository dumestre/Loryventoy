use super::{
    anim, canvas, cena, layer, pen, ruido, saida, shape, texto, transform, LayerEntry,
    ProjetoConfig, TipoNo,
};
pub use super::shape_params::ShapeParams;
pub use super::text_params::TextParams;
pub use super::pen_params::PenParams;

/// Parâmetros editáveis de cada tipo de nó.
///
/// As cores de alguns parâmetros ainda usam `egui` por compatibilidade. Elas
/// serão migradas para o domínio em uma etapa posterior.
#[derive(Clone, Debug)]
pub enum NodeParams {
    Transform { px: f32, py: f32, pz: f32, rx: f32, ry: f32, rz: f32, sx: f32, sy: f32, sz: f32 },
    Cena { nome_cena: String, ativa: bool, zoom: f32, angulo: f32, opacidade: f32 },
    Layer { cena: String, layers: Vec<LayerEntry>, selected: usize },
    Texto(TextParams),
    Shape(ShapeParams),
    Pen(PenParams),
    Ruido { seed: f32, freq: f32, amp: f32, veloc: f32, alvo: u8 },
    Anim { alvo: u8, loop_mode: u8, segmentos: Vec<crate::procedural::AnimSeg> },
    Saida { brilho: f32, contraste: f32, saturacao: f32 },
    Canvas(ProjetoConfig),
}

impl NodeParams {
    pub fn padrao(tipo: TipoNo) -> NodeParams {
        match tipo {
            TipoNo::Saida => saida::padrao(),
            TipoNo::Transform => transform::padrao(),
            TipoNo::Canvas => canvas::padrao(),
            TipoNo::Cena => cena::padrao(),
            TipoNo::Layer => layer::padrao(),
            TipoNo::Shape => shape::padrao(),
            TipoNo::Texto => texto::padrao(),
            TipoNo::Pen => pen::padrao(),
            TipoNo::Ruido => ruido::padrao(),
            TipoNo::Anim => anim::padrao(),
        }
    }
}
