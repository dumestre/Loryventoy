use std::collections::{HashMap, HashSet};

use eframe::egui::{
    self, Area, Button, Id, Key, Order,
    Pos2, Rect, Sense, Stroke, Ui, Vec2,
};
use eframe::egui::epaint::{CircleShape, TextShape};
use eframe::egui::Popup;

use crate::nodes::{NodeParams, ProjetoConfig, TipoNo, portos};
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

const ZOOM_MIN: f32 = 0.2;
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
        self.conectar_por_idx(layer, 0, cena, 0);

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
            let dt = if p.is_vetor() { types::GraphDataType::Vec2 } else { types::GraphDataType::Scalar };
            self.editor_state.graph.add_input_param(
                nid, p.nome.to_string(), dt, types::GraphValueType::None,
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
        if let Some(NodeParams::Layer { cena, .. } | NodeParams::Shape { cena, .. } | NodeParams::Texto { cena, .. } | NodeParams::Pen { cena, .. }) =
            self.params.get_mut(&idx)
        {
            if cenas.iter().all(|c| c != cena) {
                *cena = preferida.or_else(|| cenas.first().cloned()).unwrap_or_default();
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

        let loc = Pos2::new(
            (self.contador as f32 % 3.0) * 260.0,
            200.0 + (self.contador as f32 / 3.0) * 150.0,
        );
        let idx = self.adicionar_no_em(TipoNo::Layer, loc);
        let cena_nome_conn = cena_nome.clone();
        if let Some(NodeParams::Layer { cena, nome, .. }) = self.params.get_mut(&idx) {
            *cena = cena_nome;
            *nome = format!("Layer {}", self.contador);
        }
        if let Some(cena_idx) = self.cena_ativa.or_else(|| {
            self.params.iter().find_map(|(&nidx, p)| {
                if let NodeParams::Cena { nome_cena, .. } = p {
                    if *nome_cena == cena_nome_conn { Some(nidx) } else { None }
                } else { None }
            })
        }) {
            self.conectar_por_idx(idx, 0, cena_idx, 0);
        }
        self.contador += 1;
    }

    pub fn remover_layer_atual(&mut self, layer_idx: NodeId) {
        if self.params.get(&layer_idx).map_or(false, |p| matches!(p, NodeParams::Layer { .. })) {
            self.empurrar_historico();
            self.editor_state.graph.remove_node(layer_idx);
            self.params.remove(&layer_idx);
            self.liberados.remove(&layer_idx);
            self.limpar_grupos();
        }
    }

    pub fn mover_layer_atual(&mut self, layer_idx: NodeId, delta: i32) {
        if let Some(NodeParams::Layer { ordem, .. }) = self.params.get_mut(&layer_idx) {
            let novo = (*ordem as i32 + delta).max(0) as f32;
            *ordem = novo;
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
                self.editor_state.graph.remove_node(idx);
                self.params.remove(&idx);
                self.liberados.remove(&idx);
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
    pub fn screen_para_canvas(&self, screen: Pos2, pan: Vec2, zoom: f32, rect: Rect) -> Pos2 {
        let center = rect.center().to_vec2();
        ((screen.to_vec2() - center) / zoom - pan).to_pos2()
    }

    pub fn canvas_para_screen(&self, canvas: Pos2, pan: Vec2, zoom: f32, rect: Rect) -> Pos2 {
        let center = rect.center().to_vec2();
        ((canvas.to_vec2() + pan) * zoom + center).to_pos2()
    }

    #[allow(dead_code)]
    pub fn node_sob_cursor(&self, p: Pos2, pan: Vec2, zoom: f32, rect: Rect) -> Option<NodeId> {
        let canvas_p = self.screen_para_canvas(p, pan, zoom, rect);
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
    pub fn sobre_cabecalho_no(&self, p: Pos2, pan: Vec2, zoom: f32, rect: Rect) -> Option<NodeId> {
        let canvas_p = self.screen_para_canvas(p, pan, zoom, rect);
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
    pub fn porta_saida_mais_proxima(&self, p: Pos2, pan: Vec2, zoom: f32, rect: Rect) -> Option<(NodeId, usize)> {
        let canvas_p = self.screen_para_canvas(p, pan, zoom, rect);
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
    pub fn porta_entrada_mais_proxima(&self, p: Pos2, max: f32, pan: Vec2, zoom: f32, rect: Rect) -> Option<(NodeId, usize)> {
        let canvas_p = self.screen_para_canvas(p, pan, zoom, rect);
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
                self.editor_state.graph.remove_node(extra);
                self.params.remove(&extra);
                self.liberados.remove(&extra);
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



    pub fn show(&mut self, ui: &mut Ui) {
        let rect = ui.available_rect_before_wrap();
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(rect.width(), rect.height() - 2.0),
            Sense::hover(),
        );

        ui.painter().rect_filled(rect, 8.0, eframe::egui::Color32::from_rgb(22, 22, 30));

        let pan = self.editor_state.pan_zoom.pan;
        let zoom = self.editor_state.pan_zoom.zoom;

        rendering::desenhar_grade(&ui.painter().with_clip_rect(rect), rect, pan, zoom);

        self.grupos = self.grupos.clone();

        self.reafirmar_posicoes();

        self.toolbar.show(ui, rect, self.pode_undo(), self.pode_redo());

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
        self.desenhar_grupos_fundo(ui, rect, pan, zoom);

        let responses = self.editor_state.draw_graph_editor(
            ui,
            AllNodeTemplates,
            &mut UserState::default(),
            Vec::default(),
        );

        for resp in &responses.node_responses {
            match resp {
                egui_graph_edit::NodeResponse::DeleteNodeUi(nid) => {
                    if !self.is_fixo(*nid) {
                        self.empurrar_historico();
                        self.params.remove(nid);
                        self.liberados.remove(nid);
                    }
                }
                egui_graph_edit::NodeResponse::DeleteNodeFull { node_id, node: _ } => {
                    if !self.is_fixo(*node_id) {
                        self.params.remove(node_id);
                        self.liberados.remove(node_id);
                        self.limpar_grupos();
                    }
                }
                egui_graph_edit::NodeResponse::MoveNode { node, drag_delta: _ } => {
                    self.liberados.insert(*node);
                }
                egui_graph_edit::NodeResponse::ConnectEventEnded { output, input } => {
                    let _ = (output, input);
                }
                egui_graph_edit::NodeResponse::DisconnectEvent { output, input } => {
                    let _ = (output, input);
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
                }
                _ => {}
            }
        }

        self.garantir_master();
        self.reafirmar_posicoes();

        self.desenhar_grupos_header(ui, rect, pan, zoom);

        let p_screen = ui.ctx().pointer_interact_pos();

        if ui.ctx().input(|i| i.key_pressed(Key::F)) {
            let sel: Vec<NodeId> = self.editor_state.selected_nodes.iter().cloned().collect();
            if let Some(idx) = sel.first() {
                if let Some(pos) = self.editor_state.node_positions.get(*idx) {
                    let center = rect.center().to_vec2();
                    self.editor_state.pan_zoom.pan = center - pos.to_vec2() * zoom;
                    self.editor_state.pan_zoom.zoom = zoom.max(1.0).clamp(ZOOM_MIN, ZOOM_MAX);
                }
            }
        }

        if ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(Key::G)) {
            self.agrupar_selecionados();
        }

        if ui.ctx().input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            let sel: Vec<NodeId> = self.editor_state.selected_nodes.iter().cloned().collect();
            if !sel.is_empty() {
                self.empurrar_historico();
                for idx in &sel {
                    if !self.is_fixo(*idx) {
                        self.editor_state.graph.remove_node(*idx);
                        self.params.remove(idx);
                        self.liberados.remove(idx);
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
            && p_screen.map_or(false, |p| rect.contains(p));

        if abrir_menu {
            self.menu_canvas = p_screen.map(|p| {
                let center = rect.center().to_vec2();
                ((p.to_vec2() - center) / zoom - pan).to_pos2()
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
            Some(selection::AcaoMenu::Agrupar) => self.agrupar_selecionados(),
            None => {}
        }

        let mut infos: Vec<(NodeId, TipoNo, bool, Pos2)> = Vec::new();
        for idx in self.editor_state.graph.iter_nodes() {
            if let Some(pos) = self.editor_state.node_positions.get(idx) {
                let tipo = self.obter_tipo(idx);
                let selected = self.editor_state.selected_nodes.contains(&idx);
                infos.push((idx, tipo, selected, *pos));
            }
        }
        infos.sort_by(|a, b| {
            let a_sel = a.2;
            let b_sel = b.2;
            a_sel.cmp(&b_sel)
        });

        for &(idx, tipo, _selected, loc) in &infos {
            let half = node_component::content_size(tipo);
            let center = ((loc.to_vec2() + pan) * zoom + rect.center().to_vec2()).to_pos2();
            let node_rect = Rect::from_center_size(center, half * 2.0 * zoom);
            if !rect.intersects(node_rect) {
                continue;
            }
            let body_min = Pos2::new(
                node_rect.min.x + node_component::MARGEM_X * zoom,
                node_rect.min.y + (node_component::CABECALHO_H + node_component::MARGEM_Y) * zoom,
            );
            let clip_no = node_rect.intersect(rect);
            let cenas = self.cenas_disponiveis_com_indice();
            let params = self.params.get_mut(&idx);
            let mut acao_inspector = node_component::AcaoInspector::Nenhuma;
            let i_raw = Id::new(idx);
            Area::new(Id::new(("no_conteudo", i_raw)))
                .order(Order::Foreground)
                .fixed_pos(body_min)
                .movable(false)
                .constrain(false)
                .show(ui.ctx(), |ui| {
                    ui.set_clip_rect(clip_no);
                    node_component::escalar_estilo(ui, zoom);
                    ui.push_id(i_raw, |ui| {
                        acao_inspector = node_component::show_content(
                            ui, tipo, params, &cenas, body_min.y, zoom,
                        );
                    });
                });
            match acao_inspector {
                node_component::AcaoInspector::FocarCena(ci) => {
                    if let Some(pos) = self.editor_state.node_positions.get(ci) {
                        let center2 = rect.center().to_vec2();
                        self.editor_state.pan_zoom.pan = center2 - pos.to_vec2() * zoom;
                    }
                    self.cena_ativa = Some(ci);
                }
                node_component::AcaoInspector::CriarLayer => {
                    self.criar_layer_para_cena_atual();
                }
                node_component::AcaoInspector::RemoverLayer => {
                    self.remover_layer_atual(idx);
                }
                node_component::AcaoInspector::SubirLayer => {
                    self.mover_layer_atual(idx, 1);
                }
                node_component::AcaoInspector::DescerLayer => {
                    self.mover_layer_atual(idx, -1);
                }
                node_component::AcaoInspector::Nenhuma => {}
            }
            node_component::registrar_medida(tipo, node_rect.size(), zoom);
        }

        let p_canvas = p_screen.map(|p| {
            let center = rect.center().to_vec2();
            ((p.to_vec2() - center) / zoom - pan).to_pos2()
        });

        if let Some(gi) = self.grupo_header_sob(p_screen.unwrap_or_default(), pan, zoom, rect) {
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
            let painter = ui.painter().with_clip_rect(rect);
            let inner = rect.shrink(12.0);
            let raio_dot = 6.0;
            let mut dots: Vec<(NodeId, Pos2, eframe::egui::Color32)> = Vec::new();
            for idx in self.editor_state.graph.iter_nodes() {
                if let Some(pos) = self.editor_state.node_positions.get(idx) {
                    let screen = ((pos.to_vec2() + pan) * zoom + rect.center().to_vec2()).to_pos2();
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
                    rect.center().x - galley.size().x / 2.0,
                    rect.max.y - galley.size().y - 6.0,
                );
                painter.add(TextShape::new(pos, galley, eframe::egui::Color32::from_rgb(220, 220, 230)));
            }
            if let Some(idx) = click_target {
                if let Some(pos) = self.editor_state.node_positions.get(idx) {
                    let center = rect.center().to_vec2();
                    self.editor_state.pan_zoom.pan = center - pos.to_vec2() * zoom;
                    self.editor_state.pan_zoom.zoom = zoom.max(1.0).clamp(ZOOM_MIN, ZOOM_MAX);
                }
            }
        }

        ui.ctx().request_repaint();
    }
}
