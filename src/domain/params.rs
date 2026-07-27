use super::ProjectConfig as ProjetoConfig;
use super::shape_params::ShapeParams;
use super::text_params::TextParams;
use super::pen_params::PenParams;
use super::saida_params::SaidaParams;
use super::ruido_params::RuidoParams;
use super::transform_params::TransformParams;
use super::cena_params::CenaParams;
use super::layer_params::LayerParams;
use super::anim_params::AnimParams;

/// Parâmetros editáveis de cada tipo de nó.
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
