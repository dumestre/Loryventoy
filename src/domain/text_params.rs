use crate::domain::Color;

/// Parâmetros persistentes do nó Texto.
#[derive(Clone, Debug, PartialEq)]
pub struct TextParams {
    pub cena: String,
    pub conteudo: String,
    pub tamanho: f32,
    pub negrito: bool,
    pub italico: bool,
    pub px: f32,
    pub py: f32,
    pub cor: Color,
    pub trim_inicio: f32,
    pub trim_fim: f32,
}
