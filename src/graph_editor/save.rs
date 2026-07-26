use eframe::egui::Pos2;

use crate::nodes::{NodeParams, TipoNo};

use super::GraphPanel;
use super::ArestaInfo;
use super::types::NodeId;

pub type SnapshotNo = (TipoNo, Pos2, NodeParams);
pub type SnapshotAresta = (usize, usize, ArestaInfo);

pub const LIMITE_HISTORICO: usize = 50;

impl GraphPanel {
    pub fn snapshot(&self) -> (Vec<SnapshotNo>, Vec<SnapshotAresta>) {
        let mut nos = Vec::new();
        let mut idx_map: std::collections::HashMap<NodeId, usize> = std::collections::HashMap::new();

        for (i, nid) in self.editor_state.graph.iter_nodes().enumerate() {
            let node = &self.editor_state.graph[nid];
            let loc = self.editor_state.node_positions.get(nid).copied().unwrap_or(Pos2::ZERO);
            idx_map.insert(nid, i);
            let params = self.params.get(&nid).cloned().unwrap_or_else(|| node.user_data.params.clone());
            nos.push((node.user_data.tipo, loc, params));
        }

        let mut arestas = Vec::new();
        for (input_id, output_id) in self.editor_state.graph.iter_connections() {
            let src_nid = self.editor_state.graph[output_id].node;
            let dst_nid = self.editor_state.graph[input_id].node;
            let de = idx_map.get(&src_nid).copied().unwrap_or(0);
            let para = idx_map.get(&dst_nid).copied().unwrap_or(0);
            let info = ArestaInfo {
                saida: 0,
                saida_comp: None,
                entrada: 0,
                entrada_comp: None,
            };
            arestas.push((de, para, info));
        }

        (nos, arestas)
    }

    pub fn carregar_snapshot(&mut self, nos: &[(TipoNo, Pos2, NodeParams)], arestas: &[SnapshotAresta]) {
        self.editor_state.graph = super::types::MyGraph::default();
        self.editor_state.node_order.clear();
        self.editor_state.node_positions = Default::default();
        self.editor_state.node_orientations = Default::default();
        self.editor_state.selected_nodes.clear();
        self.params.clear();
        self.liberados.clear();
        self.dsl_ids.clear();

        let mut idx_to_nid: Vec<NodeId> = Vec::new();
        for (tipo, loc, params) in nos.iter() {
            let nid = self.adicionar_no_em(*tipo, *loc);
            self.definir_params(nid, params.clone());
            idx_to_nid.push(nid);
        }

        for (de, para, info) in arestas {
            if let (Some(&src), Some(&dst)) = (idx_to_nid.get(*de), idx_to_nid.get(*para)) {
                self.conectar_por_idx(src, info.saida, dst, info.entrada);
            }
        }

        self.canvas = idx_to_nid.iter().zip(nos.iter()).find(|(_, (t, _, _))| *t == TipoNo::Canvas).map(|(n, _)| *n);
        self.master = idx_to_nid.iter().zip(nos.iter()).find(|(_, (t, _, _))| *t == TipoNo::Saida).map(|(n, _)| *n);
        self.cena = idx_to_nid.iter().zip(nos.iter()).find(|(_, (t, _, _))| *t == TipoNo::Cena).map(|(n, _)| *n);
    }

    pub fn empurrar_historico(&mut self) {
        self.marcar_sujo();
        let snap = self.snapshot();
        self.undo_stack.push(snap);
        if self.undo_stack.len() > LIMITE_HISTORICO {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(snap) = self.undo_stack.pop() {
            self.marcar_sujo();
            let current = self.snapshot();
            self.redo_stack.push(current);
            self.carregar_snapshot(&snap.0, &snap.1);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(snap) = self.redo_stack.pop() {
            self.marcar_sujo();
            let current = self.snapshot();
            self.undo_stack.push(current);
            self.carregar_snapshot(&snap.0, &snap.1);
            true
        } else {
            false
        }
    }

    pub fn pode_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn pode_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
