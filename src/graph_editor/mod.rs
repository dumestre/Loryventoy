use std::collections::{HashMap, HashSet};

use eframe::egui::epaint::{CircleShape, TextShape};
use eframe::egui::Popup;
use eframe::egui::{self, Button, Key, Pos2, Rect, Sense, Stroke, Ui};

use crate::domain::Project;
use crate::dsl::application::Application;
use crate::dsl::project_dsl::ProjectBlock;
use crate::history::History;
use crate::nodes::{self, portos, NodeParams, ProjetoConfig, TipoNo};
use crate::ui::graph_toolbar::{AcaoToolbar, GraphToolbar};
use crate::ui::inspector;

pub use types::NodeId;
use types::{cor_tipo_no, AllNodeTemplates, MyEditorState, UserState};

pub mod groups;
pub mod layer_ops;
pub mod layout;
pub mod node_factory;
pub mod ports;
pub mod preview;
pub mod rendering;
pub mod save;
pub mod search;
pub mod selection;
pub mod types;

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
    history: History<Project>,
    script_text: String,
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
            history: History::new(save::LIMITE_HISTORICO),
            script_text: String::new(),
            dsl_ids: HashMap::new(),
            renaming_layer: None,
            dirty_repaint: false,
            pen_cache: HashMap::new(),
            preview_dirty: true,
        };
        panel.criar_nos_padrao();
        panel
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

    pub fn conectar_por_nome(
        &mut self,
        src: NodeId,
        saida_nome: &str,
        dst: NodeId,
        entrada_nome: &str,
    ) {
        let output_id = self.find_output(src, saida_nome);
        let input_id = self.find_input(dst, entrada_nome);
        if let (Some(out), Some(inp)) = (output_id, input_id) {
            self.editor_state.graph.add_connection(out, inp);
        }
    }

    fn find_output(&self, node: NodeId, name: &str) -> Option<egui_graph_edit::OutputId> {
        self.editor_state.graph[node]
            .outputs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, id)| *id)
    }

    fn find_input(&self, node: NodeId, name: &str) -> Option<egui_graph_edit::InputId> {
        self.editor_state.graph[node]
            .inputs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, id)| *id)
    }

    fn obter_tipo(&self, idx: NodeId) -> TipoNo {
        self.editor_state
            .graph
            .nodes
            .get(idx)
            .map(|n| n.user_data.tipo)
            .unwrap_or(TipoNo::Transform)
    }

    fn tipo_do_node(&self, idx: NodeId) -> Option<TipoNo> {
        self.editor_state
            .graph
            .nodes
            .get(idx)
            .map(|n| n.user_data.tipo)
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
        self.editor_state
            .graph
            .nodes
            .get(idx)
            .map(|n| n.user_data.tipo.nome().to_string())
            .unwrap_or_default()
    }

    pub fn projeto(&self) -> ProjetoConfig {
        self.canvas
            .and_then(|c| self.params.get(&c))
            .and_then(|p| {
                if let NodeParams::Canvas(cfg) = p {
                    Some(cfg.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(ProjetoConfig::default)
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

    pub fn garantir_master(&mut self) {
        let mestres: Vec<NodeId> = self
            .params
            .iter()
            .filter(|(_, p)| matches!(p, NodeParams::Saida(..)))
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

    fn marcar_sujo(&mut self) {
        self.dirty_repaint = true;
        self.preview_dirty = true;
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let editor_rect = ui.max_rect();

        ui.painter().rect_filled(
            editor_rect,
            8.0,
            eframe::egui::Color32::from_rgb(22, 22, 30),
        );

        let pan = self.editor_state.pan_zoom.pan;
        let zoom = self.editor_state.pan_zoom.zoom;

        rendering::desenhar_grade(
            &ui.painter().with_clip_rect(editor_rect),
            editor_rect,
            pan,
            zoom,
        );

        let (_, response) = ui.allocate_exact_size(editor_rect.size(), Sense::hover());

        self.reafirmar_posicoes();
        self.toolbar
            .show(ui, editor_rect, self.pode_undo(), self.pode_redo());

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
                AcaoToolbar::Undo => {
                    self.undo();
                }
                AcaoToolbar::Redo => {
                    self.redo();
                }
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
            acao_inspector: crate::ui::inspector::AcaoInspector::Nenhuma,
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

        if ui.ctx().input(|i| {
            i.pointer.any_down()
                || i.events.iter().any(|e| {
                    matches!(
                        e,
                        eframe::egui::Event::Key { .. } | eframe::egui::Event::Text { .. }
                    )
                })
        }) {
            self.preview_dirty = true;
            self.dirty_repaint = true;
        }

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
                egui_graph_edit::NodeResponse::MoveNode {
                    node,
                    drag_delta: _,
                } => {
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
                    self.params.insert(*nid, nodes::node_params_padrao(tipo));
                    self.liberados.insert(*nid);
                    let cenas = self.cenas_disponiveis();
                    let cena_preferida = self.cena_ativa.and_then(|ci| {
                        self.params.get(&ci).and_then(|p| {
                            if let NodeParams::Cena(cena) = p {
                                if !cena.nome_cena.is_empty() {
                                    Some(cena.nome_cena.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
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
                    self.editor_state.pan_zoom.pan =
                        (editor_rect.center().to_vec2() - pos.to_vec2()) * zoom
                            - editor_rect.min.to_vec2();
                }
            }
        }

        if ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(Key::G))
        {
            self.agrupar_selecionados();
            self.marcar_sujo();
        }

        if self.renaming_layer.is_none()
            && ui
                .ctx()
                .input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace))
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

        if ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(Key::Z))
        {
            self.undo();
        }
        if ui
            .ctx()
            .input(|i| i.modifiers.ctrl && i.key_pressed(Key::Y))
        {
            self.redo();
        }

        let mut acao_menu: Option<selection::AcaoMenu> = None;
        let sel_count = self.selecionados().len();
        let tem_clip = !self.clipboard.is_empty();
        let abrir_menu = ui.ctx().input(|i| i.pointer.secondary_clicked())
            && p_screen.map_or(false, |p| editor_rect.contains(p));

        if abrir_menu {
            self.menu_canvas = p_screen
                .map(|p| self.screen_para_canvas(p, pan, editor_rect))
                .unwrap_or_default();
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
                if ui
                    .add_enabled(sel_count >= 1, Button::new("Copiar"))
                    .clicked()
                {
                    acao_menu = Some(selection::AcaoMenu::Copiar);
                    ui.close();
                }
                if ui.add_enabled(tem_clip, Button::new("Colar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Colar);
                    ui.close();
                }
                if ui
                    .add_enabled(sel_count >= 1, Button::new("Duplicar"))
                    .clicked()
                {
                    acao_menu = Some(selection::AcaoMenu::Duplicar);
                    ui.close();
                }
                if ui
                    .add_enabled(sel_count >= 1, Button::new("Deletar"))
                    .clicked()
                {
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
                        (
                            TipoNo::Saida,
                            "Master",
                            eframe::egui::Color32::from_rgb(120, 220, 140),
                        ),
                        (
                            TipoNo::Transform,
                            "Transform",
                            eframe::egui::Color32::from_rgb(235, 185, 95),
                        ),
                        (
                            TipoNo::Canvas,
                            "Canvas",
                            eframe::egui::Color32::from_rgb(170, 120, 235),
                        ),
                        (
                            TipoNo::Cena,
                            "Cena",
                            eframe::egui::Color32::from_rgb(90, 190, 190),
                        ),
                        (
                            TipoNo::Shape,
                            "Shape",
                            eframe::egui::Color32::from_rgb(235, 150, 120),
                        ),
                        (
                            TipoNo::Texto,
                            "Texto",
                            eframe::egui::Color32::from_rgb(150, 200, 120),
                        ),
                        (
                            TipoNo::Pen,
                            "Pen",
                            eframe::egui::Color32::from_rgb(200, 120, 220),
                        ),
                        (
                            TipoNo::Ruido,
                            "Ruído",
                            eframe::egui::Color32::from_rgb(120, 200, 220),
                        ),
                        (
                            TipoNo::Anim,
                            "Animação",
                            eframe::egui::Color32::from_rgb(230, 130, 170),
                        ),
                        (
                            TipoNo::Layer,
                            "Layers",
                            eframe::egui::Color32::from_rgb(120, 170, 235),
                        ),
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
            inspector::AcaoInspector::FocarCena(ci) => {
                if let Some(pos) = self.editor_state.node_positions.get(ci) {
                    self.editor_state.pan_zoom.pan =
                        (editor_rect.center().to_vec2() - pos.to_vec2()) * zoom
                            - editor_rect.min.to_vec2();
                }
                self.cena_ativa = Some(ci);
            }
            inspector::AcaoInspector::CriarLayerEntry => {
                self.criar_layer_para_cena_atual();
                self.marcar_sujo();
            }
            inspector::AcaoInspector::RemoverLayerEntry(nid, entry_idx) => {
                self.remover_layer_entry(nid, entry_idx);
                self.marcar_sujo();
            }
            inspector::AcaoInspector::SubirLayerEntry(nid, entry_idx) => {
                self.mover_layer_entry(nid, entry_idx, -1);
                self.marcar_sujo();
            }
            inspector::AcaoInspector::DescerLayerEntry(nid, entry_idx) => {
                self.mover_layer_entry(nid, entry_idx, 1);
                self.marcar_sujo();
            }
            inspector::AcaoInspector::SelecionarLayer(nid, entry_idx) => {
                if let Some(NodeParams::Layer(layer)) = self.params.get_mut(&nid) {
                    layer.selected = entry_idx;
                    self.marcar_sujo();
                }
            }
            inspector::AcaoInspector::Nenhuma => {}
            inspector::AcaoInspector::ToggleVisivelLayer(_, _) => {}
            inspector::AcaoInspector::RenomearLayerEntry(nid, entry_idx) => {
                if let Some(NodeParams::Layer(layer)) = self.params.get_mut(&nid) {
                    if let Some(_layer) = layer.layers.get_mut(entry_idx) {
                        self.sync_layer_ports();
                        self.marcar_sujo();
                    }
                }
            }
        }

        let p_canvas = p_screen.map(|p| self.screen_para_canvas(p, pan, editor_rect));

        if let Some(gi) =
            self.grupo_header_sob(p_screen.unwrap_or_default(), pan, zoom, editor_rect)
        {
            ui.ctx().set_cursor_icon(eframe::egui::CursorIcon::Move);
            if ui.ctx().input(|i| {
                i.pointer
                    .button_pressed(eframe::egui::PointerButton::Primary)
            }) {
                self.empurrar_historico();
                self.arrastando_grupo = Some((gi, p_canvas.unwrap_or_default()));
            }
        }
        if let Some((gi, prev)) = self.arrastando_grupo {
            if ui
                .ctx()
                .input(|i| i.pointer.button_down(eframe::egui::PointerButton::Primary))
            {
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
                        let cor = cor_tipo_no(self.obter_tipo(idx));
                        dots.push((idx, Pos2::new(cx, cy), cor));
                    }
                }
            }
            let hovered_dot = dots
                .iter()
                .find(|(_, c, _)| p_screen.map_or(false, |pp| pp.distance(*c) <= raio_dot + 3.0))
                .map(|(idx, _, _)| *idx);
            let mut click_target: Option<NodeId> = None;
            if let Some(idx) = hovered_dot {
                ui.ctx()
                    .set_cursor_icon(eframe::egui::CursorIcon::PointingHand);
                if ui.ctx().input(|i| {
                    i.pointer
                        .button_pressed(eframe::egui::PointerButton::Primary)
                }) {
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
                painter.add(TextShape::new(
                    pos,
                    galley,
                    eframe::egui::Color32::from_rgb(220, 220, 230),
                ));
            }
            if let Some(idx) = click_target {
                if let Some(pos) = self.editor_state.node_positions.get(idx) {
                    self.editor_state.pan_zoom.pan =
                        (editor_rect.center().to_vec2() - pos.to_vec2()) * zoom
                            - editor_rect.min.to_vec2();
                }
            }
        }

        let precisa = self.dirty_repaint
            || ui.ctx().input(|i| i.pointer.any_down())
            || ui.ctx().input(|i| i.pointer.any_pressed())
            || ui.ctx().input(|i| {
                i.events.iter().any(|e| {
                    matches!(
                        e,
                        eframe::egui::Event::Key { .. } | eframe::egui::Event::Text { .. }
                    )
                })
            });
        if precisa {
            ui.ctx().request_repaint();
        }
        self.dirty_repaint = false;
    }
}

impl Application for GraphPanel {
    type NodeId = NodeId;

    fn criar_no(&mut self, tipo: TipoNo, pos: Pos2) -> NodeId {
        self.adicionar_no_em(tipo, pos)
    }

    fn remover_no(&mut self, idx: NodeId) {
        self.remover_no(idx);
    }

    fn obter_tipo(&self, idx: NodeId) -> TipoNo {
        self.obter_tipo(idx)
    }

    fn obter_params_mut(&mut self, idx: NodeId) -> Option<&mut NodeParams> {
        self.params.get_mut(&idx)
    }

    fn conectar_por_nome(
        &mut self,
        src: NodeId,
        saida_nome: &str,
        dst: NodeId,
        entrada_nome: &str,
    ) {
        self.conectar_por_nome(src, saida_nome, dst, entrada_nome);
    }

    fn conectar_por_idx(&mut self, src: NodeId, saida_idx: usize, dst: NodeId, entrada_idx: usize) {
        self.conectar_por_idx(src, saida_idx, dst, entrada_idx);
    }

    fn empurrar_historico(&mut self) {
        self.empurrar_historico();
    }

    fn dsl_ids(&self) -> &HashMap<String, NodeId> {
        &self.dsl_ids
    }

    fn dsl_ids_mut(&mut self) -> &mut HashMap<String, NodeId> {
        &mut self.dsl_ids
    }

    fn sync_layer_ports(&mut self) {
        self.sync_layer_ports();
    }

    fn aplicar_project_config(&mut self, bloco: &ProjectBlock) {
        if let Some(canvas_idx) = self.canvas {
            if let Some(NodeParams::Canvas(cfg)) = self.params.get_mut(&canvas_idx) {
                if let Some(v) = bloco.largura {
                    cfg.largura = v as u32;
                }
                if let Some(v) = bloco.altura {
                    cfg.altura = v as u32;
                }
                if let Some(v) = bloco.fps {
                    cfg.fps = v;
                }
                if let Some(v) = bloco.duracao {
                    cfg.duracao_seg = v;
                }
                if let Some(c) = bloco.fundo {
                    cfg.fundo = crate::domain::Color::from_rgba(c.r(), c.g(), c.b(), c.a());
                }
            }
        }
    }

    fn porto_saida_por_nome(&self, tipo: TipoNo, nome: &str) -> Option<usize> {
        let specs = portos(tipo);
        specs.saidas.iter().position(|p| p.nome == nome)
    }

    fn porto_entrada_por_nome(&self, tipo: TipoNo, nome: &str) -> Option<usize> {
        let specs = portos(tipo);
        specs.entradas.iter().position(|p| p.nome == nome)
    }

    fn tipo_portos(&self, tipo: TipoNo) -> crate::nodes::PortSpec {
        portos(tipo)
    }
}
