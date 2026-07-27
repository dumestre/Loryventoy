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
mod params;
mod layer_entry;
mod shape_params;

pub use params::NodeParams;
pub use layer_entry::LayerEntry;
pub use shape_params::ShapeParams;

use eframe::egui::Color32;

pub use crate::domain::ProjectConfig as ProjetoConfig;

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

// ── TipoNo ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoNo {
    Saida,
    Transform,
    Canvas,
    Cena,
    Layer,
    Shape,
    Texto,
    Pen,
    Ruido,
    Anim,
}

impl TipoNo {
    pub fn nome(&self) -> &'static str {
        match self {
            TipoNo::Saida => "Master",
            TipoNo::Transform => "Transform",
            TipoNo::Canvas => "Canvas",
            TipoNo::Cena => "Cena",
            TipoNo::Layer => "Layers",
            TipoNo::Shape => "Shape",
            TipoNo::Texto => "Texto",
            TipoNo::Pen => "Pen",
            TipoNo::Ruido => "Ruído",
            TipoNo::Anim => "Animação",
        }
    }

    pub fn cor(&self) -> Color32 {
        match self {
            TipoNo::Saida => Color32::from_rgb(120, 220, 140),
            TipoNo::Transform => Color32::from_rgb(235, 185, 95),
            TipoNo::Canvas => Color32::from_rgb(170, 120, 235),
            TipoNo::Cena => Color32::from_rgb(90, 190, 190),
            TipoNo::Layer => Color32::from_rgb(120, 170, 235),
            TipoNo::Shape => Color32::from_rgb(235, 150, 120),
            TipoNo::Texto => Color32::from_rgb(150, 200, 120),
            TipoNo::Pen => Color32::from_rgb(200, 120, 220),
            TipoNo::Ruido => Color32::from_rgb(120, 200, 220),
            TipoNo::Anim => Color32::from_rgb(230, 130, 170),
        }
    }

    pub fn from_label(label: &str) -> Option<TipoNo> {
        match label {
            "Master" => Some(TipoNo::Saida),
            "Transform" => Some(TipoNo::Transform),
            "Canvas" => Some(TipoNo::Canvas),
            "Cena" => Some(TipoNo::Cena),
            "Layers" => Some(TipoNo::Layer),
            "Shape" => Some(TipoNo::Shape),
            "Texto" => Some(TipoNo::Texto),
            "Pen" => Some(TipoNo::Pen),
            "Ruído" | "Ruido" => Some(TipoNo::Ruido),
            "Animação" | "Animacao" => Some(TipoNo::Anim),
            _ => None,
        }
    }

    pub fn instancia(&self) -> TipoNo {
        *self
    }

    pub fn pode_conectar(origem: TipoNo, destino: TipoNo) -> bool {
        match (origem, destino) {
            (TipoNo::Saida, _) => false,
            (_, TipoNo::Saida) => true,
            (TipoNo::Canvas, TipoNo::Cena) => true,
            (TipoNo::Cena, TipoNo::Cena) => true,
            (TipoNo::Layer, TipoNo::Shape | TipoNo::Texto | TipoNo::Pen) => true,
            (TipoNo::Shape, TipoNo::Cena) => true,
            (TipoNo::Texto, TipoNo::Cena) => true,
            (TipoNo::Pen, TipoNo::Cena) => true,
            (
                TipoNo::Ruido,
                TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen,
            ) => true,
            (
                TipoNo::Anim,
                TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen,
            ) => true,
            (
                o @ (TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen),
                TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen,
            ) if o != TipoNo::Saida => true,
            _ => false,
        }
    }
}

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
