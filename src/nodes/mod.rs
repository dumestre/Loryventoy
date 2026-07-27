#![allow(dead_code)]

mod anim;
mod canvas;
mod cena;
mod layer;
mod pen;
mod ruido;
mod saida;
mod shape;
mod texto;
mod transform;

pub use crate::domain::{
    TipoNo,
    NodeParams,
    ProjectConfig as ProjetoConfig,
    TransformParams, CenaParams, LayerParams,
    TextParams, ShapeParams, PenParams,
    RuidoParams, AnimParams, SaidaParams,
};

pub fn node_params_padrao(tipo: TipoNo) -> NodeParams {
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

// ── Tipos de porto ──────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum TipoPorto {
    Escalar,
    Vetor(&'static [&'static str]),
}

#[derive(Clone, Copy, Debug)]
pub struct ParametroPorto {
    pub nome: &'static str,
    pub tipo: TipoPorto,
}

impl ParametroPorto {
    pub fn n_componentes(&self) -> usize {
        match &self.tipo {
            TipoPorto::Escalar => 1,
            TipoPorto::Vetor(c) => c.len(),
        }
    }

    pub fn is_vetor(&self) -> bool {
        matches!(self.tipo, TipoPorto::Vetor(_))
    }

    pub fn componente(&self, k: usize) -> &'static str {
        match &self.tipo {
            TipoPorto::Escalar => self.nome,
            TipoPorto::Vetor(c) => c.get(k).copied().unwrap_or(self.nome),
        }
    }
}

pub struct PortSpec {
    pub entradas: &'static [ParametroPorto],
    pub saidas: &'static [ParametroPorto],
}

// ── Portos canônicos compartilhados ─────────────────────────────

pub(crate) const COMP_XYZ: &[&str] = &["X", "Y", "Z"];
pub(crate) const COMP_XY: &[&str] = &["X", "Y"];

pub(crate) static P_CANVAS: ParametroPorto = ParametroPorto { nome: "Canvas", tipo: TipoPorto::Escalar };
pub(crate) static P_CENA: ParametroPorto = ParametroPorto { nome: "Cena", tipo: TipoPorto::Escalar };
pub(crate) static P_LAYER: ParametroPorto = ParametroPorto { nome: "Layer", tipo: TipoPorto::Escalar };
pub(crate) static P_PEN: ParametroPorto = ParametroPorto { nome: "Pen", tipo: TipoPorto::Escalar };
pub(crate) static P_POS_XY: ParametroPorto = ParametroPorto { nome: "Posição", tipo: TipoPorto::Vetor(COMP_XY) };
pub(crate) static P_LARGURA: ParametroPorto = ParametroPorto { nome: "Largura", tipo: TipoPorto::Escalar };
pub(crate) static P_ALTURA: ParametroPorto = ParametroPorto { nome: "Altura", tipo: TipoPorto::Escalar };
pub(crate) static P_TAMANHO: ParametroPorto = ParametroPorto { nome: "Tamanho", tipo: TipoPorto::Escalar };
pub(crate) static P_RUIDO_OUT: ParametroPorto = ParametroPorto { nome: "Ruído", tipo: TipoPorto::Vetor(COMP_XY) };
pub(crate) static P_ANIM_OUT: ParametroPorto = ParametroPorto { nome: "Animação", tipo: TipoPorto::Vetor(COMP_XY) };
pub(crate) static P_OPACIDADE: ParametroPorto = ParametroPorto { nome: "Opacidade", tipo: TipoPorto::Escalar };
pub(crate) static P_ESC_XY: ParametroPorto = ParametroPorto { nome: "Escala", tipo: TipoPorto::Vetor(COMP_XY) };

// ── NodeParams ──────────────────────────────────────────────────

// ── Structs auxiliares ──────────────────────────────────────────

// ── Funções de porto ────────────────────────────────────────────

pub fn portos(tipo: TipoNo) -> PortSpec {
    match tipo {
        TipoNo::Saida => saida::portos(),
        TipoNo::Transform => transform::portos(),
        TipoNo::Canvas => canvas::portos(),
        TipoNo::Cena => cena::portos(),
        TipoNo::Layer => layer::portos(),
        TipoNo::Shape => shape::portos(),
        TipoNo::Texto => texto::portos(),
        TipoNo::Pen => pen::portos(),
        TipoNo::Ruido => ruido::portos(),
        TipoNo::Anim => anim::portos(),
    }
}

pub fn porto_saida(tipo: TipoNo, i: usize) -> Option<&'static ParametroPorto> {
    portos(tipo).saidas.get(i)
}

#[allow(dead_code)]
pub fn porto_entrada(tipo: TipoNo, i: usize) -> Option<&'static ParametroPorto> {
    portos(tipo).entradas.get(i)
}
