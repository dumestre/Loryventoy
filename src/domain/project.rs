use super::{NodeParams, ProjectConfig, TipoNo};

/// Nó do projeto — dados puros, sem referência ao grafo visual.
#[derive(Clone, Debug)]
pub struct ProjectNode {
    pub tipo: TipoNo,
    pub pos_x: f32,
    pub pos_y: f32,
    pub params: NodeParams,
}

/// Aresta do projeto — índices dos nós e portos.
#[derive(Clone, Debug)]
pub struct ProjectEdge {
    pub from: usize,
    pub to: usize,
    pub from_port: usize,
    pub from_comp: Option<usize>,
    pub to_port: usize,
    pub to_comp: Option<usize>,
}

/// Projeto completo — única fonte de verdade persistível.
#[derive(Clone, Debug)]
pub struct Project {
    pub script_text: String,
    pub nodes: Vec<ProjectNode>,
    pub edges: Vec<ProjectEdge>,
}

impl Project {
    #[allow(dead_code)]
    pub fn config(&self) -> ProjectConfig {
        self.nodes
            .iter()
            .find(|n| n.tipo == TipoNo::Canvas)
            .and_then(|n| {
                if let NodeParams::Canvas(cfg) = &n.params {
                    Some(cfg.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }
}
