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

use eframe::egui::Color32;

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

#[derive(Clone, Debug)]
pub enum NodeParams {
    Transform {
        px: f32, py: f32, pz: f32,
        rx: f32, ry: f32, rz: f32,
        sx: f32, sy: f32, sz: f32,
    },
    Cena {
        nome_cena: String,
        ativa: bool,
        zoom: f32,
        angulo: f32,
        opacidade: f32,
    },
    Layer {
        cena: String,
        layers: Vec<LayerEntry>,
        selected: usize,
    },
    Texto {
        cena: String,
        conteudo: String,
        tamanho: f32,
        negrito: bool,
        italico: bool,
        px: f32,
        py: f32,
        cor: Color32,
        trim_inicio: f32,
        trim_fim: f32,
    },
    Shape {
        cena: String,
        tipo: u8,
        px: f32, py: f32,
        largura: f32, altura: f32,
        rotacao: f32,
        cor: Color32,
        seed: f32,
        noise_scale: f32,
        amp: f32,
        veloc: f32,
        trim_inicio: f32,
        trim_fim: f32,
    },
    Pen {
        cena: String,
        codigo: String,
        erro: Option<String>,
        cor: Color32,
        cor_fill: Color32,
        pos_x: f32,
        pos_y: f32,
        espessura: f32,
        preenchimento: bool,
        seed: f32,
        cantos: f32,
        ordem: f32,
        escala_x: f32,
        escala_y: f32,
        trim_inicio: f32,
        trim_fim: f32,
    },
    Ruido {
        seed: f32,
        freq: f32,
        amp: f32,
        veloc: f32,
        alvo: u8,
    },
    Anim {
        alvo: u8,
        loop_mode: u8,
        segmentos: Vec<crate::procedural::AnimSeg>,
    },
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

// ── Structs auxiliares ──────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct ProjetoConfig {
    pub largura: u32,
    pub altura: u32,
    pub fps: f32,
    pub duracao_seg: f32,
    pub fundo: Color32,
}

impl Default for ProjetoConfig {
    fn default() -> Self {
        Self {
            largura: 1920,
            altura: 1080,
            fps: 30.0,
            duracao_seg: 5.0,
            fundo: Color32::WHITE,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayerEntry {
    pub nome: String,
    pub ordem: f32,
    pub opacidade: f32,
    pub cor: Color32,
    pub visivel: bool,
    pub renomeando: bool,
}

impl LayerEntry {
    const PALETTE: [Color32; 8] = [
        Color32::from_rgb(90, 170, 235),
        Color32::from_rgb(235, 150, 120),
        Color32::from_rgb(150, 200, 120),
        Color32::from_rgb(200, 120, 220),
        Color32::from_rgb(235, 185, 95),
        Color32::from_rgb(120, 200, 220),
        Color32::from_rgb(230, 130, 170),
        Color32::from_rgb(170, 120, 235),
    ];

    pub fn cor_por_idx(idx: usize) -> Color32 {
        Self::PALETTE[idx % Self::PALETTE.len()]
    }
}

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
