use std::collections::{HashMap, HashSet};

use eframe::egui::{
    self, Button, Key,
    Pos2, Rect, Sense, Stroke, Ui, Vec2,
};
use eframe::egui::epaint::{CircleShape, TextShape};
use eframe::egui::Popup;

use crate::nodes::{NodeParams, LayerEntry, ProjetoConfig, TipoNo, portos};
use crate::ui::graph_toolbar::{GraphToolbar, AcaoToolbar};
use crate::ui::node_component;

use types::{GraphNode, MyEditorState, UserState, AllNodeTemplates};
pub use types::NodeId;

pub mod types;
pub mod ports;
pub mod rendering;
pub mod selection;
pub mod groups;
pub mod save;
pub mod preview;
pub mod dsl;

#[allow(dead_code)]
const ZOOM_MIN: f32 = 0.2;
#[allow(dead_code)]
const ZOOM_MAX: f32 = 1.2;

#[allow(dead_code)]
struct MenuComponentes {
    src: NodeId,
    saida: usize,
    drop_screen: Pos2,
    drop_canvas: Pos2,
    alvo: Option<(NodeId, usize)>,
    escolha: Option<usize>,
    rect: Option<Rect>,
    dest_nome: Option<String>,
}

pub struct GraphPanel {
    pub editor_state: MyEditorState,
    pub toolbar: GraphToolbar,
    pub contador: usize,
    master: Option<NodeId>,
    master_loc: Pos2,
    canvas: Option<NodeId>,
    canvas_loc: Pos2,
    cena: Option<NodeId>,
    cena_loc: Pos2,
    cena_ativa: Option<NodeId>,
    liberados: HashSet<NodeId>,
    params: HashMap<NodeId, NodeParams>,
    grupos: Vec<groups::Grupo>,
    #[allow(dead_code)]
    grupo_seq: usize,
    clipboard: Vec<selection::NoCopia>,
    menu_canvas: Pos2,
    arrastando_grupo: Option<(usize, Pos2)>,
    undo_stack: Vec<(Vec<save::SnapshotNo>, Vec<save::SnapshotAresta>)>,
    redo_stack: Vec<(Vec<save::SnapshotNo>, Vec<save::SnapshotAresta>)>,
    dsl_ids: HashMap<String, NodeId>,
    renaming_layer: Option<(NodeId, usize)>,
    dirty_repaint: bool,
    pen_cache: HashMap<String, crate::dsl::Program>,
    preview_dirty: bool,
}

#[derive(Clone, Copy, Debug, Default)]
#[allow(dead_code)]
pub struct ArestaInfo {
    pub saida: usize,
    pub saida_comp: Option<usize>,
    pub entrada: usize,
    pub entrada_comp: Option<usize>,
}


#[allow(dead_code)]
pub struct GraphResponse;

impl GraphPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            editor_state: MyEditorState::default(),
            toolbar: GraphToolbar::new(),
            contador: 0,
            master: None,
            master_loc: Pos2::ZERO,
            canvas: None,
            canvas_loc: Pos2::ZERO,
            cena: None,
            cena_loc: Pos2::ZERO,
            cena_ativa: None,
            liberados: HashSet::new(),
            params: HashMap::new(),
            grupos: Vec::new(),
            grupo_seq: 0,
            clipboard: Vec::new(),
            menu_canvas: Pos2::ZERO,
            arrastando_grupo: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dsl_ids: HashMap::new(),
            renaming_layer: None,
            dirty_repaint: false,
            pen_cache: HashMap::new(),
            preview_dirty: true,
        };
        panel.criar_nos_padrao();
        panel
    }

    fn criar_nos_padrao(&mut self) {
        self.editor_state = MyEditorState::default();
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
            if let NodeParams::Cena { nome_cena, .. } = p {
                Some(nome_cena.clone())
            } else {
                None
            }
        });
        if let Some(nome) = &nome_cena {
            if let Some(NodeParams::Layer { cena: lc, .. }) = self.params.get_mut(&layer) {
                *lc = nome.clone();
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
            params: NodeParams::padrao(tipo),
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
                (types::GraphDataType::Vec2, types::GraphValueType::Vec2(Vec2::ZERO))
            } else {
                (types::GraphDataType::Scalar, types::GraphValueType::Scalar(0.0))
            };
            self.editor_state.graph.add_input_param(
                nid, p.nome.to_string(), dt, vt,
                egui_graph_edit::InputParamKind::ConnectionOrConstant, true,
            );
        }
        for p in spec.saidas.iter() {
            let dt = if p.is_vetor() { types::GraphDataType::Vec2 } else { types::GraphDataType::Scalar };
            self.editor_state.graph.add_output_param(nid, p.nome.to_string(), dt);
        }

        self.params.insert(nid, NodeParams::padrao(tipo));
        let cenas = self.cenas_disponiveis();
        let cena_preferida = self.cena_ativa.and_then(|ci| {
            self.params.get(&ci).and_then(|p| {
                if let NodeParams::Cena { nome_cena, .. } = p {
                    if !nome_cena.is_empty() { Some(nome_cena.clone()) } else { None }
                } else { None }
            })
        });
        self.normalizar_cena(nid, &cenas, cena_preferida);
        nid
    }

    fn adicionar_no(&mut self, tipo: TipoNo) {
        if tipo == TipoNo::Saida && self.master.is_some() { return; }
        let col = (self.contador % 3) as f32;
        let lin = (self.contador / 3) as f32;
        let loc = Pos2::new(40.0 + col * 260.0, 40.0 + lin * 150.0);
        self.adicionar_no_em(tipo, loc);
        self.contador += 1;
    }

    fn conectar_por_idx(&mut self, src: NodeId, saida: usize, dst: NodeId, entrada: usize) {
        let spec_src = portos(self.obter_tipo(src));
        let spec_dst = portos(self.obter_tipo(dst));
        if let (Some(saida_nome), Some(entrada_nome)) = (
            spec_src.saidas.get(saida).map(|p| p.nome),
            spec_dst.entradas.get(entrada).map(|p| p.nome),
        ) {
            self.conectar_por_nome(src, &saida_nome, dst, &entrada_nome);
        }
    }

    pub fn conectar_por_nome(&mut self, src: NodeId, saida_nome: &str, dst: NodeId, entrada_nome: &str) {
        let output_id = self.find_output(src, saida_nome);
        let input_id = self.find_input(dst, entrada_nome);
        if let (Some(out), Some(inp)) = (output_id, input_id) {
            self.editor_state.graph.add_connection(out, inp);
        }
    }

    fn find_output(&self, node: NodeId, name: &str) -> Option<egui_graph_edit::OutputId> {
        self.editor_state.graph[node].outputs.iter()
            .find(|(n, _)| n == name)
            .map(|(_, id)| *id)
    }

    fn find_input(&self, node: NodeId, name: &str) -> Option<egui_graph_edit::InputId> {
        self.editor_state.graph[node].inputs.iter()
            .find(|(n, _)| n == name)
            .map(|(_, id)| *id)
    }

    fn obter_tipo(&self, idx: NodeId) -> TipoNo {
        self.editor_state.graph.nodes.get(idx)
            .map(|n| n.user_data.tipo)
            .unwrap_or(TipoNo::Transform)
    }

    fn tipo_do_node(&self, idx: NodeId) -> Option<TipoNo> {
        self.editor_state.graph.nodes.get(idx).map(|n| n.user_data.tipo)
    }

    fn is_master(&self, idx: NodeId) -> bool {
        self.master == Some(idx)
    }

    fn is_canvas(&self, idx: NodeId) -> bool {
        self.canvas == Some(idx)
    }

    fn is_fixo(&self, idx: NodeId) -> bool {
        self.is_master(idx) || self.is_canvas(idx)
    }

    fn remover_no(&mut self, idx: NodeId) {
        self.editor_state.graph.remove_node(idx);
        self.editor_state.node_order.retain(|n| *n != idx);
        self.editor_state.node_positions.remove(idx);
        self.editor_state.node_orientations.remove(idx);
        self.editor_state.selected_nodes.retain(|n| *n != idx);
        self.params.remove(&idx);
        self.liberados.remove(&idx);
    }

    fn obter_params(&self, idx: NodeId) -> Option<&NodeParams> {
        self.params.get(&idx)
    }

    fn definir_params(&mut self, idx: NodeId, params: NodeParams) {
        self.params.insert(idx, params);
    }

    fn obter_label(&self, idx: NodeId) -> String {
        self.editor_state.graph.nodes.get(idx).map(|n| n.user_data.tipo.nome().to_string()).unwrap_or_default()
    }

    fn cenas_disponiveis(&self) -> Vec<String> {
        let mut v: Vec<String> = self.params.iter()
            .filter_map(|(_, p)| {
                if let NodeParams::Cena { nome_cena, .. } = p {
                    if !nome_cena.is_empty() { Some(nome_cena.clone()) } else { None }
                } else { None }
            })
            .collect();
        v.sort();
        v.dedup();
        v
    }

    fn cenas_disponiveis_com_indice(&self) -> Vec<(String, NodeId)> {
        let mut v: Vec<(String, NodeId)> = self.params.iter()
            .filter_map(|(&idx, p)| {
                if let NodeParams::Cena { nome_cena, .. } = p {
                    if !nome_cena.is_empty() { Some((nome_cena.clone(), idx)) } else { None }
                } else { None }
            })
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v.dedup_by(|a, b| a.0 == b.0);
        v
    }

    fn normalizar_cena(&mut self, idx: NodeId, cenas: &[String], preferida: Option<String>) {
        if let Some(params) = self.params.get_mut(&idx) {
            let cena = match params {
                NodeParams::Layer { cena, .. }
                | NodeParams::Texto { cena, .. }
                | NodeParams::Pen { cena, .. } => cena,
                NodeParams::Shape(shape) => &mut shape.cena,
                _ => return,
            };
            if cenas.iter().all(|c| c != cena) {
                *cena = preferida.or_else(|| cenas.first().cloned()).unwrap_or_default();
            }
        }
    }

    fn sync_layer_ports(&mut self) {
        let layer_nids: Vec<NodeId> = self.params.iter()
            .filter(|(_, p)| matches!(p, NodeParams::Layer { .. }))
            .map(|(&nid, _)| nid)
            .collect();

        for nid in layer_nids {
            let entries: Vec<(String, f32)> = match self.params.get(&nid) {
                Some(NodeParams::Layer { layers, .. }) => {
                    layers.iter().map(|e| (e.nome.clone(), e.opacidade)).collect()
                }
                _ => continue,
            };

            // Build desired port names (reversed so bottom = oldest)
            let mut desired: Vec<String> = entries.iter().enumerate()
                .map(|(i, (nome, _))| {
                    if nome.is_empty() {
                        format!("Layer {}", i + 1)
                    } else {
                        nome.clone()
                    }
                })
                .collect();
            desired.reverse();

            // Save current order for comparison
            let current_order: Vec<String> = self.editor_state.graph[nid].outputs.iter()
                .map(|(name, _)| name.clone())
                .collect();

            if current_order == desired {
                continue;
            }

            // Save connections: output_name -> list of InputIds connected to it
            let current_outputs: Vec<(String, egui_graph_edit::OutputId)> =
                self.editor_state.graph[nid].outputs.clone();
            let mut saved_connections: HashMap<String, Vec<egui_graph_edit::InputId>> = HashMap::new();
            for (input_id, &output_id) in self.editor_state.graph.connections.iter() {
                if self.editor_state.graph.outputs[output_id].node == nid {
                    if let Some((name, _)) = current_outputs.iter().find(|(_, id)| *id == output_id) {
                        saved_connections.entry(name.clone()).or_default().push(input_id);
                    }
                }
            }

            // Remove ALL output ports (this also removes their connections)
            for (_, oid) in &current_outputs {
                self.editor_state.graph.remove_output_param(*oid);
            }

            // Re-add in desired order and reconnect
            for name in &desired {
                let new_oid = self.editor_state.graph.add_output_param(
                    nid,
                    name.clone(),
                    types::GraphDataType::Scalar,
                );
                if let Some(inputs) = saved_connections.get(name) {
                    for &input_id in inputs {
                        self.editor_state.graph.add_connection(new_oid, input_id);
                    }
                }
            }
        }
    }

    pub fn criar_layer_para_cena_atual(&mut self) {
        self.empurrar_historico();
        let cenas = self.cenas_disponiveis();
        let cena_nome = self.cena_ativa.and_then(|ci| {
            self.params.get(&ci).and_then(|p| {
                if let NodeParams::Cena { nome_cena, .. } = p {
                    if !nome_cena.is_empty() { Some(nome_cena.clone()) } else { None }
                } else { None }
            })
        }).or_else(|| cenas.first().cloned()).unwrap_or_default();

        // Find existing Layer node for this scene
        let existing = self.params.iter().find_map(|(&idx, p)| {
            if let NodeParams::Layer { cena, .. } = p {
                if *cena == cena_nome { Some(idx) } else { None }
            } else { None }
        });

        if let Some(layer_nid) = existing {
            // Add new entry to existing Layer node (at the top)
            let count = match self.params.get(&layer_nid) {
                Some(NodeParams::Layer { layers, .. }) => layers.len(),
                _ => 0,
            };
            if let Some(NodeParams::Layer { layers, .. }) = self.params.get_mut(&layer_nid) {
                layers.insert(0, LayerEntry {
                    nome: format!("Layer {}", count + 1),
                    ordem: 0.0,
                    opacidade: 1.0,
                    cor: LayerEntry::cor_por_idx(count),
                    visivel: true,
                });
            }
        } else {
            // Create new Layer node for this scene
            let loc = Pos2::new(
                (self.contador as f32 % 3.0) * 260.0,
                200.0 + (self.contador as f32 / 3.0) * 150.0,
            );
            let idx = self.adicionar_no_em(TipoNo::Layer, loc);
            if let Some(NodeParams::Layer { cena, layers, .. }) = self.params.get_mut(&idx) {
                *cena = cena_nome;
                layers.insert(0, LayerEntry {
                    nome: "Layer 1".to_string(),
                    ordem: 0.0,
                    opacidade: 1.0,
                    cor: LayerEntry::cor_por_idx(0),
                    visivel: true,
                });
            }
            self.contador += 1;
        }
    }

    pub fn remover_layer_entry(&mut self, layer_idx: NodeId, entry_idx: usize) {
        let (should_remove_node, should_remove_entry) = match self.params.get(&layer_idx) {
            Some(NodeParams::Layer { layers, .. }) => {
                if entry_idx < layers.len() && layers.len() > 1 {
                    (false, true)
                } else if layers.len() == 1 {
                    (true, false)
                } else {
                    (false, false)
                }
            }
            _ => (false, false),
        };
        if should_remove_node {
            self.empurrar_historico();
            self.remover_no(layer_idx);
            self.limpar_grupos();
        } else if should_remove_entry {
            self.empurrar_historico();
            if let Some(NodeParams::Layer { layers, selected, .. }) = self.params.get_mut(&layer_idx) {
                layers.remove(entry_idx);
                if *selected >= layers.len() {
                    *selected = layers.len().saturating_sub(1);
                }
            }
        }
    }

    pub fn mover_layer_entry(&mut self, layer_idx: NodeId, entry_idx: usize, delta: i32) {
        if let Some(NodeParams::Layer { layers, .. }) = self.params.get_mut(&layer_idx) {
            let new_idx = (entry_idx as i32 + delta) as usize;
            if new_idx < layers.len() {
                layers.swap(entry_idx, new_idx);
            }
        }
    }

    pub fn sincronizar_marcadores_com_cenas(&mut self, markers: &[crate::ui::timeline::Marker]) {
        self.empurrar_historico();
        let nomes_marc: Vec<String> = markers.iter().map(|m| m.name.clone()).collect();
        let mut cenas_por_nome: HashMap<String, NodeId> = HashMap::new();
        let mut cenas_para_remover: Vec<NodeId> = Vec::new();

        for (&idx, p) in &self.params {
            if let NodeParams::Cena { nome_cena, .. } = p {
                if !nome_cena.is_empty() {
                    if nomes_marc.contains(nome_cena) {
                        cenas_por_nome.insert(nome_cena.clone(), idx);
                    } else {
                        cenas_para_remover.push(idx);
                    }
                }
            }
        }
        for idx in cenas_para_remover {
            if self.params.get(&idx).map_or(false, |p| matches!(p, NodeParams::Cena { .. })) {
                self.remover_no(idx);
            }
        }
        for nome in &nomes_marc {
            if !cenas_por_nome.contains_key(nome) {
                let loc = Pos2::new(
                    (self.contador as f32 % 3.0) * 260.0,
                    (self.contador as f32 / 3.0) * 150.0,
                );
                let idx = self.adicionar_no_em(TipoNo::Cena, loc);
                if let Some(NodeParams::Cena { nome_cena, .. }) = self.params.get_mut(&idx) {
                    *nome_cena = nome.clone();
                }
                cenas_por_nome.insert(nome.clone(), idx);
                self.contador += 1;
            }
        }
    }

    pub fn projeto(&self) -> ProjetoConfig {
        self.canvas.and_then(|c| self.params.get(&c)).and_then(|p| {
            if let NodeParams::Canvas(cfg) = p { Some(cfg.clone()) } else { None }
        }).unwrap_or_else(ProjetoConfig::default)
    }

    #[allow(dead_code)]
    fn get_input_node(&self, _input_id: egui_graph_edit::InputId) -> Option<NodeId> {
        None
    }

    #[allow(dead_code)]
    fn get_output_node(&self, _output_id: egui_graph_edit::OutputId) -> Option<NodeId> {
        None
    }

    fn aplicar_modelo_anim_texto(&mut self, _id: u8) {
        self.contador += 2;
    }

    pub fn buscar(&mut self, termo: &str) {
        for nid in self.editor_state.graph.iter_nodes() {
            let node = &self.editor_state.graph[nid];
            let selected = node.user_data.tipo.nome().to_lowercase().contains(&termo.to_lowercase());
            if selected {
                if !self.editor_state.selected_nodes.contains(&nid) {
                    self.editor_state.selected_nodes.push(nid);
                }
            } else {
                self.editor_state.selected_nodes.retain(|&n| n != nid);
            }
        }
    }

    #[allow(dead_code)]
    pub fn screen_para_canvas(&self, screen: Pos2, pan: Vec2, editor_rect: Rect) -> Pos2 {
        (screen.to_vec2() - pan - editor_rect.min.to_vec2()).to_pos2()
    }

    pub fn canvas_para_screen(&self, canvas: Pos2, pan: Vec2, zoom: f32, editor_rect: Rect) -> Pos2 {
        let center = editor_rect.center().to_vec2();
        ((canvas.to_vec2() - center) * zoom + center + pan + editor_rect.min.to_vec2()).to_pos2()
    }

    #[allow(dead_code)]
    pub fn node_sob_cursor(&self, p: Pos2, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<NodeId> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
        for nid in self.editor_state.graph.iter_nodes() {
            if let Some(pos) = self.editor_state.node_positions.get(nid) {
                let label = self.obter_label(nid);
                let half = ports::tamanho(&label);
                if (canvas_p.x - pos.x).abs() <= half.x && (canvas_p.y - pos.y).abs() <= half.y {
                    return Some(nid);
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn sobre_cabecalho_no(&self, p: Pos2, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<NodeId> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
        for nid in self.editor_state.graph.iter_nodes() {
            if let Some(pos) = self.editor_state.node_positions.get(nid) {
                let label = self.obter_label(nid);
                let half = ports::tamanho(&label);
                let header_h = node_component::CABECALHO_H;
                if (canvas_p.x - pos.x).abs() <= half.x
                    && (canvas_p.y - pos.y).abs() <= half.y
                    && (pos.y - half.y + header_h - canvas_p.y) >= 0.0
                {
                    return Some(nid);
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn portos_offsets(&self, idx: NodeId) -> Option<(Vec<Vec2>, Vec<Vec2>)> {
        let label = self.obter_label(idx);
        let half = ports::tamanho(&label);
        let tipo = self.obter_tipo(idx);
        Some(ports::port_offsets(tipo, half))
    }

    #[allow(dead_code)]
    pub fn porta_saida_mais_proxima(&self, p: Pos2, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<(NodeId, usize)> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
        let mut melhor: Option<(NodeId, usize, f32)> = None;
        for nid in self.editor_state.graph.iter_nodes() {
            if let Some(pos) = self.editor_state.node_positions.get(nid) {
                let label = self.obter_label(nid);
                let half = ports::tamanho(&label);
                let tipo = self.obter_tipo(nid);
                let (_, outs) = ports::port_offsets(tipo, half);
                for (i, off) in outs.iter().enumerate() {
                    let p_port = *pos + *off;
                    let d = (canvas_p - p_port).length();
                    if d < 12.0 && melhor.map_or(true, |(_, _, bd)| d < bd) {
                        melhor = Some((nid, i, d));
                    }
                }
            }
        }
        melhor.map(|(nid, i, _)| (nid, i))
    }

    #[allow(dead_code)]
    pub fn porta_entrada_mais_proxima(&self, p: Pos2, max: f32, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<(NodeId, usize)> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
        let mut melhor: Option<(NodeId, usize, f32)> = None;
        for nid in self.editor_state.graph.iter_nodes() {
            if let Some(pos) = self.editor_state.node_positions.get(nid) {
                let label = self.obter_label(nid);
                let half = ports::tamanho(&label);
                let tipo = self.obter_tipo(nid);
                let (ins, _) = ports::port_offsets(tipo, half);
                for (i, off) in ins.iter().enumerate() {
                    let p_port = *pos + *off;
                    let d = (canvas_p - p_port).length();
                    if d < max && melhor.map_or(true, |(_, _, bd)| d < bd) {
                        melhor = Some((nid, i, d));
                    }
                }
            }
        }
        melhor.map(|(nid, i, _)| (nid, i))
    }

    #[allow(dead_code)]
    pub fn porta_entrada_canvas(&self, idx: NodeId, porta: usize) -> Option<Pos2> {
        let pos = self.editor_state.node_positions.get(idx).copied()?;
        let label = self.obter_label(idx);
        let half = ports::tamanho(&label);
        let tipo = self.obter_tipo(idx);
        let (ins, _) = ports::port_offsets(tipo, half);
        ins.get(porta).map(|off| pos + *off)
    }

    pub fn garantir_master(&mut self) {
        let mestres: Vec<NodeId> = self.params.iter()
            .filter(|(_, p)| matches!(p, NodeParams::Saida { .. }))
            .map(|(&idx, _)| idx)
            .collect();

        if mestres.is_empty() {
            let master = self.adicionar_no_em(TipoNo::Saida, self.master_loc);
            self.master = Some(master);
        } else if mestres.len() > 1 {
            let keep = mestres[0];
            for &extra in &mestres[1..] {
                self.remover_no(extra);
            }
            self.master = Some(keep);
        } else {
            self.master = Some(mestres[0]);
        }
    }

    pub fn reafirmar_posicoes(&mut self) {
        let fixar = |idx: Option<NodeId>, loc: Pos2, liberados: &HashSet<NodeId>, positions: &mut slotmap::SecondaryMap<NodeId, Pos2>| {
            if let Some(i) = idx {
                if !liberados.contains(&i) {
                    positions.insert(i, loc);
                }
            }
        };
        fixar(self.canvas, self.canvas_loc, &self.liberados, &mut self.editor_state.node_positions);
        fixar(self.cena, self.cena_loc, &self.liberados, &mut self.editor_state.node_positions);
        fixar(self.master, self.master_loc, &self.liberados, &mut self.editor_state.node_positions);
    }



    fn marcar_sujo(&mut self) {
        self.dirty_repaint = true;
        self.preview_dirty = true;
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let editor_rect = ui.max_rect();

        ui.painter().rect_filled(editor_rect, 8.0, eframe::egui::Color32::from_rgb(22, 22, 30));

        let pan = self.editor_state.pan_zoom.pan;
        let zoom = self.editor_state.pan_zoom.zoom;

        rendering::desenhar_grade(&ui.painter().with_clip_rect(editor_rect), editor_rect, pan, zoom);

        let (_, response) = ui.allocate_exact_size(editor_rect.size(), Sense::hover());

        self.reafirmar_posicoes();
        self.toolbar.show(ui, editor_rect, self.pode_undo(), self.pode_redo());

        if let Some(acao) = self.toolbar.acao.take() {
            match acao {
                AcaoToolbar::Adicionar(t) => {
                    self.empurrar_historico();
                    self.adicionar_no(t);
                }
                AcaoToolbar::ModeloAnimTexto(id) => {
                    self.empurrar_historico();
                    self.aplicar_modelo_anim_texto(id);
                }
                AcaoToolbar::Undo => { self.undo(); }
                AcaoToolbar::Redo => { self.redo(); }
            }
        }

        if self.toolbar.focus_search {
            self.toolbar.focus_search = false;
            let q = self.toolbar.search_query.clone();
            self.buscar(&q);
        }

        self.limpar_grupos();
        self.desenhar_grupos_fundo(ui, editor_rect, pan, zoom);

        let mut user_state = UserState {
            params: self.params.clone(),
            cenas: self.cenas_disponiveis_com_indice(),
            acao_inspector: crate::ui::node_component::AcaoInspector::Nenhuma,
            renaming_layer: self.renaming_layer,
        };

        let responses = self.editor_state.draw_graph_editor(
            ui,
            AllNodeTemplates,
            &mut user_state,
            Vec::default(),
        );

        self.params = user_state.params;
        self.renaming_layer = user_state.renaming_layer;

        // If user is actively editing (mouse held or typing), mark preview dirty
        // so slider drags, text edits, etc. cause preview to rebuild.
        if ui.ctx().input(|i| i.pointer.any_down() || i.events.iter().any(|e| matches!(e, eframe::egui::Event::Key { .. } | eframe::egui::Event::Text { .. }))) {
            self.preview_dirty = true;
            self.dirty_repaint = true;
        }

        // Sync dynamic output ports for Layer nodes
        self.sync_layer_ports();

        for resp in &responses.node_responses {
            match resp {
                egui_graph_edit::NodeResponse::DeleteNodeUi(nid) => {
                    if !self.is_fixo(*nid) {
                        self.empurrar_historico();
                        self.params.remove(nid);
                        self.liberados.remove(nid);
                        self.marcar_sujo();
                    }
                }
                egui_graph_edit::NodeResponse::DeleteNodeFull { node_id, node: _ } => {
                    if !self.is_fixo(*node_id) {
                        self.params.remove(node_id);
                        self.liberados.remove(node_id);
                        self.limpar_grupos();
                        self.marcar_sujo();
                    }
                }
                egui_graph_edit::NodeResponse::MoveNode { node, drag_delta: _ } => {
                    self.liberados.insert(*node);
                    self.dirty_repaint = true;
                }
                egui_graph_edit::NodeResponse::ConnectEventEnded { output, input } => {
                    let _ = (output, input);
                    self.marcar_sujo();
                }
                egui_graph_edit::NodeResponse::DisconnectEvent { output, input } => {
                    let _ = (output, input);
                    self.marcar_sujo();
                }
                egui_graph_edit::NodeResponse::CreatedNode(nid) => {
                    let tipo = self.obter_tipo(*nid);
                    self.params.insert(*nid, NodeParams::padrao(tipo));
                    self.liberados.insert(*nid);
                    let cenas = self.cenas_disponiveis();
                    let cena_preferida = self.cena_ativa.and_then(|ci| {
                        self.params.get(&ci).and_then(|p| {
                            if let NodeParams::Cena { nome_cena, .. } = p {
                                if !nome_cena.is_empty() { Some(nome_cena.clone()) } else { None }
                            } else { None }
                        })
                    });
                    self.normalizar_cena(*nid, &cenas, cena_preferida);
                    self.marcar_sujo();
                }
                _ => {}
            }
        }

        self.garantir_master();
        self.reafirmar_posicoes();

        self.desenhar_grupos_header(ui, editor_rect, pan, zoom);

        let pan = self.editor_state.pan_zoom.pan;
        let zoom = self.editor_state.pan_zoom.zoom;

        let p_screen = ui.ctx().pointer_interact_pos();

        if ui.ctx().input(|i| i.key_pressed(Key::F)) {
            let sel: Vec<NodeId> = self.editor_state.selected_nodes.iter().cloned().collect();
            if let Some(idx) = sel.first() {
                if let Some(pos) = self.editor_state.node_positions.get(*idx) {
                    self.editor_state.pan_zoom.pan = (editor_rect.center().to_vec2() - pos.to_vec2()) * zoom - editor_rect.min.to_vec2();
                }
            }
        }

        if ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(Key::G)) {
            self.agrupar_selecionados();
            self.marcar_sujo();
        }

        if self.renaming_layer.is_none()
            && ui.ctx().input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace))
        {
            let sel: Vec<NodeId> = self.editor_state.selected_nodes.iter().cloned().collect();
            if !sel.is_empty() {
                self.empurrar_historico();
                for idx in &sel {
                    if !self.is_fixo(*idx) {
                        self.remover_no(*idx);
                    }
                }
                self.editor_state.selected_nodes.clear();
                self.limpar_grupos();
            }
        }

        if ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(Key::Z)) {
            self.undo();
        }
        if ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(Key::Y)) {
            self.redo();
        }

        let mut acao_menu: Option<selection::AcaoMenu> = None;
        let sel_count = self.selecionados().len();
        let tem_clip = !self.clipboard.is_empty();
        let abrir_menu = ui.ctx().input(|i| i.pointer.secondary_clicked())
            && p_screen.map_or(false, |p| editor_rect.contains(p));

        if abrir_menu {
            self.menu_canvas = p_screen.map(|p| {
                self.screen_para_canvas(p, pan, editor_rect)
            }).unwrap_or_default();
        }

        Popup::menu(&response)
            .open_memory(if abrir_menu {
                Some(eframe::egui::SetOpenCommand::Bool(true))
            } else {
                None
            })
            .at_pointer_fixed()
            .show(|ui| {
                ui.set_min_width(120.0);
                if ui.add_enabled(sel_count >= 1, Button::new("Copiar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Copiar);
                    ui.close();
                }
                if ui.add_enabled(tem_clip, Button::new("Colar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Colar);
                    ui.close();
                }
                if ui.add_enabled(sel_count >= 1, Button::new("Duplicar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Duplicar);
                    ui.close();
                }
                if ui.add_enabled(sel_count >= 1, Button::new("Deletar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Deletar);
                    ui.close();
                }
                if sel_count >= 1 {
                    ui.separator();
                    if ui.button("Agrupar").clicked() {
                        acao_menu = Some(selection::AcaoMenu::Agrupar);
                        ui.close();
                    }
                }
                ui.separator();
                ui.menu_button("Adicionar nó", |ui| {
                    let tipos: [(TipoNo, &str, eframe::egui::Color32); 10] = [
                        (TipoNo::Saida, "Master", eframe::egui::Color32::from_rgb(120, 220, 140)),
                        (TipoNo::Transform, "Transform", eframe::egui::Color32::from_rgb(235, 185, 95)),
                        (TipoNo::Canvas, "Canvas", eframe::egui::Color32::from_rgb(170, 120, 235)),
                        (TipoNo::Cena, "Cena", eframe::egui::Color32::from_rgb(90, 190, 190)),
                        (TipoNo::Shape, "Shape", eframe::egui::Color32::from_rgb(235, 150, 120)),
                        (TipoNo::Texto, "Texto", eframe::egui::Color32::from_rgb(150, 200, 120)),
                        (TipoNo::Pen, "Pen", eframe::egui::Color32::from_rgb(200, 120, 220)),
                        (TipoNo::Ruido, "Ruído", eframe::egui::Color32::from_rgb(120, 200, 220)),
                        (TipoNo::Anim, "Animação", eframe::egui::Color32::from_rgb(230, 130, 170)),
                        (TipoNo::Layer, "Layers", eframe::egui::Color32::from_rgb(120, 170, 235)),
                    ];
                    for (t, nome, cor) in &tipos {
                        if ui.button(egui::RichText::new(*nome).color(*cor)).clicked() {
                            let p = self.menu_canvas;
                            let idx = self.adicionar_no_em(*t, p);
                            self.selecionar_no(idx, false);
                            ui.close();
                        }
                    }
                });
            });

        match acao_menu {
            Some(selection::AcaoMenu::Copiar) => self.copiar_selecionados(),
            Some(selection::AcaoMenu::Colar) => {
                self.empurrar_historico();
                let p = self.menu_canvas;
                self.colar_em(p);
            }
            Some(selection::AcaoMenu::Duplicar) => {
                self.empurrar_historico();
                self.duplicar_selecionados();
            }
            Some(selection::AcaoMenu::Deletar) => {
                self.empurrar_historico();
                self.deletar_selecionados();
            }
            Some(selection::AcaoMenu::Agrupar) => {
                self.agrupar_selecionados();
                self.marcar_sujo();
            }
            None => {}
        }

        match user_state.acao_inspector {
            node_component::AcaoInspector::FocarCena(ci) => {
                if let Some(pos) = self.editor_state.node_positions.get(ci) {
                    self.editor_state.pan_zoom.pan = (editor_rect.center().to_vec2() - pos.to_vec2()) * zoom - editor_rect.min.to_vec2();
                }
                self.cena_ativa = Some(ci);
            }
            node_component::AcaoInspector::CriarLayerEntry => {
                self.criar_layer_para_cena_atual();
                self.marcar_sujo();
            }
            node_component::AcaoInspector::RemoverLayerEntry(nid, entry_idx) => {
                self.remover_layer_entry(nid, entry_idx);
                self.marcar_sujo();
            }
            node_component::AcaoInspector::SubirLayerEntry(nid, entry_idx) => {
                self.mover_layer_entry(nid, entry_idx, -1);
                self.marcar_sujo();
            }
            node_component::AcaoInspector::DescerLayerEntry(nid, entry_idx) => {
                self.mover_layer_entry(nid, entry_idx, 1);
                self.marcar_sujo();
            }
            node_component::AcaoInspector::SelecionarLayer(nid, entry_idx) => {
                if let Some(NodeParams::Layer { selected, .. }) = self.params.get_mut(&nid) {
                    *selected = entry_idx;
                    self.marcar_sujo();
                }
            }
            node_component::AcaoInspector::Nenhuma => {}
            node_component::AcaoInspector::ToggleVisivelLayer(_, _) => {}
            node_component::AcaoInspector::RenomearLayerEntry(nid, entry_idx) => {
                if let Some(NodeParams::Layer { layers, .. }) = self.params.get_mut(&nid) {
                    if let Some(_layer) = layers.get_mut(entry_idx) {
                        self.sync_layer_ports();
                        self.marcar_sujo();
                    }
                }
            }
        }

        let p_canvas = p_screen.map(|p| {
            self.screen_para_canvas(p, pan, editor_rect)
        });

        if let Some(gi) = self.grupo_header_sob(p_screen.unwrap_or_default(), pan, zoom, editor_rect) {
            ui.ctx().set_cursor_icon(eframe::egui::CursorIcon::Move);
            if ui.ctx().input(|i| i.pointer.button_pressed(eframe::egui::PointerButton::Primary)) {
                self.empurrar_historico();
                self.arrastando_grupo = Some((gi, p_canvas.unwrap_or_default()));
            }
        }
        if let Some((gi, prev)) = self.arrastando_grupo {
            if ui.ctx().input(|i| i.pointer.button_down(eframe::egui::PointerButton::Primary)) {
                if let Some(pc) = p_canvas {
                    let delta = pc - prev;
                    self.arrastando_grupo = Some((gi, pc));
                    let nos = self.grupos[gi].nos.clone();
                    for idx in nos {
                        if let Some(pos) = self.editor_state.node_positions.get_mut(idx) {
                            *pos = *pos + delta;
                        }
                        self.liberados.insert(idx);
                    }
                }
            } else {
                self.arrastando_grupo = None;
            }
        }

        {
            let painter = ui.painter().with_clip_rect(editor_rect);
            let inner = editor_rect.shrink(12.0);
            let raio_dot = 6.0;
            let mut dots: Vec<(NodeId, Pos2, eframe::egui::Color32)> = Vec::new();
            let center = editor_rect.center().to_vec2();
            let e_min = editor_rect.min.to_vec2();
            for idx in self.editor_state.graph.iter_nodes() {
                if self.is_fixo(idx) {
                    continue;
                }
                if let Some(pos) = self.editor_state.node_positions.get(idx) {
                    let screen = ((pos.to_vec2() - center) * zoom + center + pan + e_min).to_pos2();
                    if !inner.contains(screen) {
                        let cx = screen.x.clamp(inner.min.x, inner.max.x);
                        let cy = screen.y.clamp(inner.min.y, inner.max.y);
                        let cor = self.obter_tipo(idx).cor();
                        dots.push((idx, Pos2::new(cx, cy), cor));
                    }
                }
            }
            let hovered_dot = dots.iter().find(|(_, c, _)| {
                p_screen.map_or(false, |pp| pp.distance(*c) <= raio_dot + 3.0)
            }).map(|(idx, _, _)| *idx);
            let mut click_target: Option<NodeId> = None;
            if let Some(idx) = hovered_dot {
                ui.ctx().set_cursor_icon(eframe::egui::CursorIcon::PointingHand);
                if ui.ctx().input(|i| i.pointer.button_pressed(eframe::egui::PointerButton::Primary)) {
                    click_target = Some(idx);
                }
            }
            for (idx, c, cor) in &dots {
                let hov = hovered_dot == Some(*idx);
                let r = if hov { raio_dot + 2.0 } else { raio_dot };
                painter.add(CircleShape {
                    center: *c,
                    radius: r,
                    fill: if hov { cor.gamma_multiply(1.35) } else { *cor },
                    stroke: Stroke::new(1.5, eframe::egui::Color32::from_rgb(20, 20, 26)),
                });
                if hov {
                    painter.add(CircleShape {
                        center: *c,
                        radius: r + 3.0,
                        fill: eframe::egui::Color32::TRANSPARENT,
                        stroke: Stroke::new(1.0, cor.gamma_multiply(0.6)),
                    });
                }
            }
            if !dots.is_empty() {
                let txt = format!("{} nó(s) fora da view", dots.len());
                let galley = painter.layout_no_wrap(
                    txt,
                    eframe::egui::FontId::proportional(11.0),
                    eframe::egui::Color32::from_rgb(220, 220, 230),
                );
                let pos = Pos2::new(
                    editor_rect.center().x - galley.size().x / 2.0,
                    editor_rect.max.y - galley.size().y - 6.0,
                );
                painter.add(TextShape::new(pos, galley, eframe::egui::Color32::from_rgb(220, 220, 230)));
            }
            if let Some(idx) = click_target {
                if let Some(pos) = self.editor_state.node_positions.get(idx) {
                    self.editor_state.pan_zoom.pan = (editor_rect.center().to_vec2() - pos.to_vec2()) * zoom - editor_rect.min.to_vec2();
                }
            }
        }

        let precisa = self.dirty_repaint
            || ui.ctx().input(|i| i.pointer.any_down())
            || ui.ctx().input(|i| i.pointer.any_pressed())
            || ui.ctx().input(|i| i.events.iter().any(|e| matches!(e, eframe::egui::Event::Key { .. } | eframe::egui::Event::Text { .. })));
        if precisa {
            ui.ctx().request_repaint();
        }
        self.dirty_repaint = false;
    }
}
