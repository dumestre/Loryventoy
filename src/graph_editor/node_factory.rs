use eframe::egui::{Pos2, Vec2};

use crate::nodes::{self, portos, TipoNo, NodeParams};

use super::types::{GraphNode, NodeId};
use super::GraphPanel;

impl GraphPanel {
    pub fn criar_nos_padrao(&mut self) {
        self.editor_state = super::types::MyEditorState::default();
        self.params.clear();
        self.liberados.clear();
        self.grupos.clear();

        const ESPACO: f32 = 290.0;
        self.canvas_loc = Pos2::new(-ESPACO, 0.0);
        self.cena_loc = Pos2::new(0.0, 0.0);
        self.master_loc = Pos2::new(ESPACO, 0.0);

        let canvas = self.adicionar_no_em(TipoNo::Canvas, self.canvas_loc);
        let cena = self.adicionar_no_em(TipoNo::Cena, self.cena_loc);
        let master = self.adicionar_no_em(TipoNo::Saida, self.master_loc);

        self.conectar_por_idx(canvas, 0, cena, 0);
        self.conectar_por_idx(cena, 0, master, 0);

        self.canvas = Some(canvas);
        self.cena = Some(cena);
        self.master = Some(master);

        let layer_loc = Pos2::new(0.0, 180.0);
        let layer = self.adicionar_no_em(TipoNo::Layer, layer_loc);
        let nome_cena = self.obter_params(cena).and_then(|p| {
            if let NodeParams::Cena(cena) = p {
                Some(cena.nome_cena.clone())
            } else {
                None
            }
        });
        if let Some(nome) = &nome_cena {
            if let Some(NodeParams::Layer(layer)) = self.params.get_mut(&layer) {
                layer.cena = nome.clone();
            }
        }

        self.dsl_ids.clear();
        self.dsl_ids.insert("canvas".to_string(), canvas);
        self.dsl_ids.insert("scene".to_string(), cena);
        self.dsl_ids.insert("master".to_string(), master);
    }

    pub fn adicionar_no_em(&mut self, tipo: TipoNo, loc: Pos2) -> NodeId {
        let user_data = GraphNode {
            tipo,
            params: nodes::node_params_padrao(tipo),
        };
        let label = tipo.nome().to_string();
        let nid = self.editor_state.graph.add_node(label, user_data, |_g, _id| {});
        self.editor_state.node_positions.insert(nid, loc);
        self.editor_state.node_orientations.insert(nid, egui_graph_edit::NodeOrientation::LeftToRight);
        if !self.editor_state.node_order.contains(&nid) {
            self.editor_state.node_order.push(nid);
        }

        let spec = portos(tipo);
        for p in spec.entradas.iter() {
            let (dt, vt) = if p.is_vetor() {
                (super::types::GraphDataType::Vec2, super::types::GraphValueType::Vec2(Vec2::ZERO))
            } else {
                (super::types::GraphDataType::Scalar, super::types::GraphValueType::Scalar(0.0))
            };
            self.editor_state.graph.add_input_param(
                nid, p.nome.to_string(), dt, vt,
                egui_graph_edit::InputParamKind::ConnectionOrConstant, true,
            );
        }
        for p in spec.saidas.iter() {
            let dt = if p.is_vetor() { super::types::GraphDataType::Vec2 } else { super::types::GraphDataType::Scalar };
            self.editor_state.graph.add_output_param(nid, p.nome.to_string(), dt);
        }

        self.params.insert(nid, nodes::node_params_padrao(tipo));
        let cenas = self.cenas_disponiveis();
        let cena_preferida = self.cena_ativa.and_then(|ci| {
            self.params.get(&ci).and_then(|p| {
                if let NodeParams::Cena(cena) = p {
                    if !cena.nome_cena.is_empty() { Some(cena.nome_cena.clone()) } else { None }
                } else { None }
            })
        });
        self.normalizar_cena(nid, &cenas, cena_preferida);
        nid
    }

    pub fn adicionar_no(&mut self, tipo: TipoNo) {
        if tipo == TipoNo::Saida && self.master.is_some() { return; }
        let col = (self.contador % 3) as f32;
        let lin = (self.contador / 3) as f32;
        let loc = Pos2::new(40.0 + col * 260.0, 40.0 + lin * 150.0);
        self.adicionar_no_em(tipo, loc);
        self.contador += 1;
    }
}
