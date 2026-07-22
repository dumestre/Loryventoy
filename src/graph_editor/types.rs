#![allow(dead_code)]

use std::borrow::Cow;

use eframe::egui::{Color32, Vec2, Ui};
use egui_graph_edit::{
    DataTypeTrait, NodeDataTrait, NodeResponse, NodeTemplateIter,
    WidgetValueTrait, InputParamKind, NodeTemplateTrait, UserResponseTrait,
};
pub use egui_graph_edit::id_type::NodeId;

use crate::nodes::{NodeParams, TipoNo, portos};

#[derive(Default)]
pub struct UserState;

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
        _param_name: &str,
        _node_id: NodeId,
        _ui: &mut Ui,
        _user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<GraphResponse> {
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
        _ui: &mut Ui,
        _node_id: NodeId,
        _graph: &egui_graph_edit::Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>> {
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
            let dt = if p.is_vetor() {
                GraphDataType::Vec2
            } else {
                GraphDataType::Scalar
            };
            graph.add_input_param(
                node_id,
                p.nome.to_string(),
                dt,
                GraphValueType::None,
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
