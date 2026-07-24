#![allow(dead_code)]

use std::borrow::Cow;
use std::collections::HashMap;

use eframe::egui::{Color32, RichText, Stroke, Vec2, Ui, Align, Layout};
use egui_graph_edit::{
    DataTypeTrait, NodeDataTrait, NodeResponse, NodeTemplateIter,
    WidgetValueTrait, InputParamKind, NodeTemplateTrait, UserResponseTrait,
};
pub use egui_graph_edit::id_type::NodeId;

use crate::nodes::{NodeParams, TipoNo, portos};
use crate::ui::node_component::AcaoInspector;

#[derive(Default)]
pub struct UserState {
    pub params: HashMap<NodeId, NodeParams>,
    pub cenas: Vec<(String, NodeId)>,
    pub acao_inspector: AcaoInspector,
}

#[derive(Clone)]
pub struct GraphNode {
    pub tipo: TipoNo,
    pub params: NodeParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GraphDataType {
    Scalar,
    Vec2,
    Any,
}

#[derive(Clone, Debug)]
pub enum GraphValueType {
    None,
    Scalar(f32),
    Vec2(Vec2),
}

impl Default for GraphValueType {
    fn default() -> Self {
        GraphValueType::None
    }
}

#[derive(Clone)]
pub enum NodeTemplate {
    Canvas,
    Transform,
    Cena,
    Layer,
    Shape,
    Texto,
    Pen,
    Ruido,
    Anim,
    Saida,
}

impl NodeTemplate {
    pub fn tipo(&self) -> TipoNo {
        match self {
            NodeTemplate::Canvas => TipoNo::Canvas,
            NodeTemplate::Transform => TipoNo::Transform,
            NodeTemplate::Cena => TipoNo::Cena,
            NodeTemplate::Layer => TipoNo::Layer,
            NodeTemplate::Shape => TipoNo::Shape,
            NodeTemplate::Texto => TipoNo::Texto,
            NodeTemplate::Pen => TipoNo::Pen,
            NodeTemplate::Ruido => TipoNo::Ruido,
            NodeTemplate::Anim => TipoNo::Anim,
            NodeTemplate::Saida => TipoNo::Saida,
        }
    }
}

pub type MyGraph = egui_graph_edit::Graph<GraphNode, GraphDataType, GraphValueType>;
pub type MyEditorState = egui_graph_edit::GraphEditorState<
    GraphNode,
    GraphDataType,
    GraphValueType,
    NodeTemplate,
    UserState,
>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphResponse;

impl UserResponseTrait for GraphResponse {}

impl DataTypeTrait<UserState> for GraphDataType {
    fn data_type_color(&self, _user_state: &mut UserState) -> Color32 {
        match self {
            GraphDataType::Scalar => Color32::from_gray(160),
            GraphDataType::Vec2 => Color32::from_rgb(120, 180, 220),
            GraphDataType::Any => Color32::from_gray(120),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            GraphDataType::Scalar => "Scalar".into(),
            GraphDataType::Vec2 => "Vec2".into(),
            GraphDataType::Any => "Any".into(),
        }
    }
}

impl WidgetValueTrait for GraphValueType {
    type Response = GraphResponse;
    type UserState = UserState;
    type NodeData = GraphNode;

    fn value_widget(
        &mut self,
        param_name: &str,
        _node_id: NodeId,
        ui: &mut Ui,
        _user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<GraphResponse> {
        match self {
            GraphValueType::Scalar(v) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(eframe::egui::DragValue::new(v).speed(0.01));
                });
            }
            GraphValueType::Vec2(v) => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.label("X");
                    ui.add(eframe::egui::DragValue::new(&mut v.x).speed(0.5));
                    ui.label("Y");
                    ui.add(eframe::egui::DragValue::new(&mut v.y).speed(0.5));
                });
            }
            GraphValueType::None => {}
        }
        vec![]
    }
}

impl NodeDataTrait for GraphNode {
    type Response = GraphResponse;
    type UserState = UserState;
    type DataType = GraphDataType;
    type ValueType = GraphValueType;

    fn bottom_ui(
        &self,
        ui: &mut Ui,
        node_id: NodeId,
        _graph: &egui_graph_edit::Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>> {
        let params = user_state.params.get_mut(&node_id);
        let cenas = &user_state.cenas;
        let acao = crate::ui::node_component::show_content(
            ui, self.tipo, params, cenas, 0.0, 1.0,
        );
        user_state.acao_inspector = acao;
        vec![]
    }

    fn titlebar_color(&self, _ui: &Ui, _node_id: NodeId, _graph: &egui_graph_edit::Graph<Self, Self::DataType, Self::ValueType>, _user_state: &mut Self::UserState) -> Option<Color32> {
        Some(self.tipo.cor())
    }

    fn border_color(&self, _ui: &Ui, _node_id: NodeId, _graph: &egui_graph_edit::Graph<Self, Self::DataType, Self::ValueType>, _user_state: &mut Self::UserState) -> Option<Color32> {
        Some(self.tipo.cor())
    }

    fn border_width(&self, _ui: &Ui, _node_id: NodeId, _graph: &egui_graph_edit::Graph<Self, Self::DataType, Self::ValueType>, _user_state: &mut Self::UserState) -> f32 {
        1.5
    }

    fn output_ui(
        &self,
        ui: &mut Ui,
        node_id: NodeId,
        _graph: &egui_graph_edit::Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        param_name: &str,
    ) -> Vec<NodeResponse<Self::Response, Self>> {
        if self.tipo == TipoNo::Layer {
            let entry_info = user_state.params.get(&node_id).and_then(|p| {
                if let NodeParams::Layer { layers, selected, .. } = p {
                    layers.iter().position(|l| l.nome == param_name).map(|idx| {
                        (idx, *selected == idx, layers[idx].nome.clone())
                    })
                } else {
                    None
                }
            });

            if let Some((entry_idx, is_selected, entry_nome)) = entry_info {
                let mut acao = AcaoInspector::Nenhuma;
                let icon_size = Vec2::new(16.0, 16.0);

                let row_resp = ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let sel_text = if is_selected { "\u{25CF}" } else { "\u{25CB}" };
                    if ui.selectable_label(is_selected, RichText::new(sel_text).strong()).clicked() {
                        acao = AcaoInspector::SelecionarLayer(entry_idx);
                    }

                    ui.label(&entry_nome);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(egui::Image::new(eframe::egui::include_image!("../ui/icons/delete.svg")).fit_to_exact_size(icon_size)).clicked() {
                            acao = AcaoInspector::RemoverLayerEntry(entry_idx);
                        }
                        if ui.add(egui::Image::new(eframe::egui::include_image!("../ui/icons/arrow_down.svg")).fit_to_exact_size(icon_size)).clicked() {
                            acao = AcaoInspector::DescerLayerEntry(entry_idx);
                        }
                        if ui.add(egui::Image::new(eframe::egui::include_image!("../ui/icons/arrow_up.svg")).fit_to_exact_size(icon_size)).clicked() {
                            acao = AcaoInspector::SubirLayerEntry(entry_idx);
                        }
                    });
                });

                if is_selected {
                    ui.painter().rect_stroke(
                        row_resp.response.rect,
                        0.0,
                        Stroke::new(2.0, Color32::from_rgb(100, 180, 255)),
                        egui::StrokeKind::Inside,
                    );
                }

                if acao != AcaoInspector::Nenhuma {
                    user_state.acao_inspector = acao;
                }
                return vec![];
            }
        }

        ui.label(param_name);
        vec![]
    }
}

impl NodeTemplateTrait for NodeTemplate {
    type NodeData = GraphNode;
    type DataType = GraphDataType;
    type ValueType = GraphValueType;
    type UserState = UserState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        self.tipo().nome().into()
    }

    fn node_graph_label(&self, _user_state: &mut Self::UserState) -> String {
        self.tipo().nome().to_string()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        let tipo = self.tipo();
        GraphNode {
            tipo,
            params: NodeParams::padrao(tipo),
        }
    }

    fn build_node(
        &self,
        graph: &mut egui_graph_edit::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        let tipo = self.tipo();
        let spec = portos(tipo);
        for p in spec.entradas.iter() {
            let (dt, vt) = if p.is_vetor() {
                (GraphDataType::Vec2, GraphValueType::Vec2(Vec2::ZERO))
            } else {
                (GraphDataType::Scalar, GraphValueType::Scalar(0.0))
            };
            graph.add_input_param(
                node_id,
                p.nome.to_string(),
                dt,
                vt,
                InputParamKind::ConnectionOrConstant,
                true,
            );
        }
        for p in spec.saidas.iter() {
            let dt = if p.is_vetor() {
                GraphDataType::Vec2
            } else {
                GraphDataType::Scalar
            };
            graph.add_output_param(node_id, p.nome.to_string(), dt);
        }
    }

    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<Self::CategoryType> {
        vec!["Nodes"]
    }
}

pub struct AllNodeTemplates;

impl NodeTemplateIter for AllNodeTemplates {
    type Item = NodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            NodeTemplate::Transform,
            NodeTemplate::Cena,
            NodeTemplate::Layer,
            NodeTemplate::Shape,
            NodeTemplate::Texto,
            NodeTemplate::Pen,
            NodeTemplate::Ruido,
            NodeTemplate::Anim,
        ]
    }
}
