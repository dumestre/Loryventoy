use crate::domain::Project;
use crate::nodes::TipoNo;

use eframe::egui::Pos2;

use super::types::NodeId;
use super::GraphPanel;

pub const LIMITE_HISTORICO: usize = 50;

impl GraphPanel {
    pub fn empurrar_historico(&mut self) {
        self.marcar_sujo();
        self.history.push(self.to_project());
    }

    pub fn undo(&mut self) -> bool {
        if let Some(proj) = self.history.undo() {
            self.marcar_sujo();
            self.script_text = proj.script_text.clone();
            self.load_project(&proj);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(proj) = self.history.redo() {
            self.marcar_sujo();
            self.script_text = proj.script_text.clone();
            self.load_project(&proj);
            true
        } else {
            false
        }
    }

    pub fn pode_undo(&self) -> bool {
        self.history.pode_undo()
    }

    pub fn pode_redo(&self) -> bool {
        self.history.pode_redo()
    }

    pub fn to_project(&self) -> Project {
        use std::collections::HashMap;
        let mut idx_map: HashMap<NodeId, usize> = HashMap::new();
        let nodes: Vec<crate::domain::ProjectNode> = self
            .editor_state
            .graph
            .iter_nodes()
            .enumerate()
            .map(|(i, nid)| {
                idx_map.insert(nid, i);
                let node = &self.editor_state.graph[nid];
                let loc = self
                    .editor_state
                    .node_positions
                    .get(nid)
                    .copied()
                    .unwrap_or(Pos2::ZERO);
                let params = self
                    .params
                    .get(&nid)
                    .cloned()
                    .unwrap_or_else(|| node.user_data.params.clone());
                crate::domain::ProjectNode {
                    tipo: node.user_data.tipo,
                    pos_x: loc.x,
                    pos_y: loc.y,
                    params,
                }
            })
            .collect();

        let edges: Vec<crate::domain::ProjectEdge> = self
            .editor_state
            .graph
            .iter_connections()
            .map(|(input_id, output_id)| {
                let src_nid = self.editor_state.graph[output_id].node;
                let dst_nid = self.editor_state.graph[input_id].node;
                let from = idx_map.get(&src_nid).copied().unwrap_or(0);
                let to = idx_map.get(&dst_nid).copied().unwrap_or(0);
                crate::domain::ProjectEdge {
                    from,
                    to,
                    from_port: 0,
                    from_comp: None,
                    to_port: 0,
                    to_comp: None,
                }
            })
            .collect();

        Project {
            nodes,
            edges,
            script_text: self.script_text.clone(),
        }
    }

    pub fn load_project(&mut self, proj: &Project) {
        self.editor_state.graph = super::types::MyGraph::default();
        self.editor_state.node_order.clear();
        self.editor_state.node_positions = Default::default();
        self.editor_state.node_orientations = Default::default();
        self.editor_state.selected_nodes.clear();
        self.params.clear();
        self.liberados.clear();
        self.dsl_ids.clear();

        let mut idx_to_nid: Vec<NodeId> = Vec::new();
        for n in &proj.nodes {
            let loc = Pos2::new(n.pos_x, n.pos_y);
            let nid = self.adicionar_no_em(n.tipo, loc);
            self.definir_params(nid, n.params.clone());
            idx_to_nid.push(nid);
        }

        for e in &proj.edges {
            if let (Some(&src), Some(&dst)) = (idx_to_nid.get(e.from), idx_to_nid.get(e.to)) {
                self.conectar_por_idx(src, e.from_port, dst, e.to_port);
            }
        }

        self.canvas = idx_to_nid
            .iter()
            .zip(&proj.nodes)
            .find(|(_, n)| n.tipo == TipoNo::Canvas)
            .map(|(n, _)| *n);
        self.master = idx_to_nid
            .iter()
            .zip(&proj.nodes)
            .find(|(_, n)| n.tipo == TipoNo::Saida)
            .map(|(n, _)| *n);
        self.cena = idx_to_nid
            .iter()
            .zip(&proj.nodes)
            .find(|(_, n)| n.tipo == TipoNo::Cena)
            .map(|(n, _)| *n);
    }
}