use super::{
    anim, canvas, cena, layer, pen, ruido, saida, shape, texto, transform,
    ProjetoConfig, TipoNo,
};
pub use super::shape_params::ShapeParams;
pub use super::text_params::TextParams;
pub use super::pen_params::PenParams;
pub use super::saida_params::SaidaParams;
pub use super::ruido_params::RuidoParams;
pub use super::transform_params::TransformParams;
pub use super::cena_params::CenaParams;
pub use super::layer_params::LayerParams;
pub use super::anim_params::AnimParams;

/// Parâmetros editáveis de cada tipo de nó.
///
/// As cores de alguns parâmetros ainda usam `egui` por compatibilidade. Elas
/// serão migradas para o domínio em uma etapa posterior.
#[derive(Clone, Debug)]
pub enum NodeParams {
    Transform(TransformParams),
    Cena(CenaParams),
    Layer(LayerParams),
    Texto(TextParams),
    Shape(ShapeParams),
    Pen(PenParams),
    Ruido(RuidoParams),
    Anim(AnimParams),
    Saida(SaidaParams),
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
