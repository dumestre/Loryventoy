use std::collections::HashMap;

use glam::Vec2 as GVec2;

use crate::nodes::NodeParams;
use crate::procedural::{
    CenaPreview, LayerPreview, PreviewData, ShapeGenerator, ShapeKind, TextoItem, PenPath,
};

use super::types::NodeId;
use super::GraphPanel;

impl GraphPanel {
    pub fn formas_para_preview(&self) -> PreviewData {
        let mut preview = PreviewData::default();
        let cfg = self.projeto();
        preview.largura = cfg.largura as f32;
        preview.altura = cfg.altura as f32;
        preview.fundo = cfg.fundo;

        let _graph = &self.editor_state.graph;

        // Map Layer NodeId -> cena name, for quick lookup
        let mut layer_to_cena: HashMap<NodeId, String> = HashMap::new();
        for (&nid, params) in &self.params {
            if let NodeParams::Layer { cena, .. } = params {
                layer_to_cena.insert(nid, cena.clone());
            }
        }

        // Group scenes by name
        let mut cena_names: Vec<(String, NodeId)> = Vec::new();
        for (&nid, params) in &self.params {
            if let NodeParams::Cena { nome_cena, .. } = params {
                cena_names.push((nome_cena.clone(), nid));
            }
        }

        // For each scene, build CenaPreview
        for (nome_cena, cena_nid) in &cena_names {
            let cena_opac = match self.params.get(cena_nid) {
                Some(NodeParams::Cena { opacidade, .. }) => *opacidade,
                _ => 1.0,
            };

            let mut cena_preview = CenaPreview {
                opacidade: cena_opac,
                layers: Vec::new(),
            };

            // Collect layers belonging to this scene
            let mut layer_nids: Vec<NodeId> = layer_to_cena.iter()
                .filter(|(_, c)| *c == nome_cena)
                .map(|(&nid, _)| nid)
                .collect();

            // Sort by ordem
            layer_nids.sort_by(|a, b| {
                let ord_a = match self.params.get(a) {
                    Some(NodeParams::Layer { ordem, .. }) => *ordem,
                    _ => 0.0,
                };
                let ord_b = match self.params.get(b) {
                    Some(NodeParams::Layer { ordem, .. }) => *ordem,
                    _ => 0.0,
                };
                ord_a.partial_cmp(&ord_b).unwrap_or(std::cmp::Ordering::Equal)
            });

            // If no layers, create a default one
            if layer_nids.is_empty() {
                layer_nids.push(NodeId::default());
            }

            for &layer_nid in &layer_nids {
                let (nome, opac) = match self.params.get(&layer_nid) {
                    Some(NodeParams::Layer { nome, opacidade, .. }) => {
                        (nome.clone(), *opacidade)
                    }
                    _ => ("Layer 1".to_string(), 1.0),
                };

                let mut layer_preview = LayerPreview {
                    nome,
                    opacidade: opac,
                    formas: Vec::new(),
                    textos: Vec::new(),
                    pen: Vec::new(),
                };

                // Find all nodes that belong to this scene
                for (&nid, params) in &self.params {
                    match params {
                        NodeParams::Shape { .. } | NodeParams::Texto { .. } | NodeParams::Pen { .. } => {
                            if self.node_belongs_to_layer(nid, layer_nid, &layer_to_cena, nome_cena) {
                                match params {
                                    NodeParams::Shape { .. } => {
                                        if let Some(gen) = self.build_shape_generator(nid, params) {
                                            layer_preview.formas.push(gen);
                                        }
                                    }
                                    NodeParams::Texto { .. } => {
                                        if let Some(item) = self.build_texto_item(nid, params) {
                                            layer_preview.textos.push(item);
                                        }
                                    }
                                    NodeParams::Pen { .. } => {
                                        if let Some(pp) = self.build_pen_path(nid, params) {
                                            layer_preview.pen.push(pp);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                cena_preview.layers.push(layer_preview);
            }

            preview.cenas.push(cena_preview);
        }

        preview
    }

    /// Checks if a Shape/Texto/Pen node belongs to a given layer.
    /// A node belongs to a layer if:
    /// 1. Its first input ("Canvas") connects to this layer's output, OR
    /// 2. No layer connection exists and the node's cena matches the layer's cena
    fn node_belongs_to_layer(
        &self,
        nid: NodeId,
        layer_nid: NodeId,
        layer_to_cena: &HashMap<NodeId, String>,
        cena_name: &str,
    ) -> bool {
        let graph = &self.editor_state.graph;
        let node = &graph[nid];

        // Check if any input is connected to this layer's output
        if let Some(layer_node) = graph.nodes.get(layer_nid) {
            if let Some((_, layer_out_id)) = layer_node.outputs.first() {
                for (_, input_id) in &node.inputs {
                    if let Some(connected_out) = graph.connection(*input_id) {
                        if connected_out == *layer_out_id {
                            return true;
                        }
                    }
                }
            }
        }

        // Fallback: if no layer connection, node belongs to first layer of its scene
        // (when there's only one layer, everything goes there)
        let scene_layers: Vec<&NodeId> = layer_to_cena.iter()
            .filter(|(_, c)| *c == cena_name)
            .map(|(nid, _)| nid)
            .collect();

        if scene_layers.len() <= 1 {
            let node_cena = match &self.params.get(&nid) {
                Some(NodeParams::Shape { cena, .. }) => cena.as_str(),
                Some(NodeParams::Texto { cena, .. }) => cena.as_str(),
                Some(NodeParams::Pen { cena, .. }) => cena.as_str(),
                _ => "",
            };
            return node_cena == cena_name;
        }

        false
    }

    fn build_shape_generator(&self, _nid: NodeId, params: &NodeParams) -> Option<ShapeGenerator> {
        if let NodeParams::Shape {
            tipo,
            px, py,
            largura, altura,
            rotacao,
            cor,
            seed,
            noise_scale,
            amp,
            veloc,
            trim_inicio,
            trim_fim,
            ..
        } = params {
            let kind = ShapeKind::from_u8(*tipo);

            // Check for connected Ruido/Anim nodes
            let ruido = self.find_connected_ruido(_nid);
            let anim = self.find_connected_anim(_nid);

            Some(ShapeGenerator {
                kind,
                pos: GVec2::new(*px, *py),
                tam: GVec2::new(*largura, *altura),
                rot: *rotacao,
                cor: *cor,
                seed: *seed,
                noise_scale: *noise_scale,
                amp: *amp,
                veloc: *veloc,
                ruido,
                anim,
                trim_inicio: *trim_inicio,
                trim_fim: *trim_fim,
                duracao: self.projeto().duracao_seg,
            })
        } else {
            None
        }
    }

    fn build_texto_item(&self, _nid: NodeId, params: &NodeParams) -> Option<TextoItem> {
        if let NodeParams::Texto {
            px, py,
            conteudo,
            tamanho,
            negrito,
            italico,
            cor,
            trim_inicio,
            trim_fim,
            ..
        } = params {
            let ruido = self.find_connected_ruido(_nid);
            let anim = self.find_connected_anim(_nid);

            Some(TextoItem {
                px: *px,
                py: *py,
                conteudo: conteudo.clone(),
                tamanho: *tamanho,
                negrito: *negrito,
                italico: *italico,
                cor: *cor,
                escala_x: 1.0,
                escala_y: 1.0,
                ruido,
                anim,
                trim_inicio: *trim_inicio,
                trim_fim: *trim_fim,
            })
        } else {
            None
        }
    }

    fn build_pen_path(&self, _nid: NodeId, params: &NodeParams) -> Option<PenPath> {
        if let NodeParams::Pen {
            codigo,
            cor,
            cor_fill,
            pos_x,
            pos_y,
            espessura,
            preenchimento,
            seed,
            cantos,
            ordem,
            escala_x,
            escala_y,
            trim_inicio,
            trim_fim,
            ..
        } = params {
            let program = match crate::dsl::Program::parse(codigo) {
                Ok(p) => p,
                Err(_) => crate::dsl::Program::default(),
            };

            let ruido = self.find_connected_ruido(_nid);
            let anim = self.find_connected_anim(_nid);

            Some(PenPath {
                program,
                pos: GVec2::new(*pos_x, *pos_y),
                cor: *cor,
                cor_fill: *cor_fill,
                espessura: *espessura,
                preenchimento: *preenchimento,
                seed: *seed as u32,
                cantos: *cantos,
                ordem: *ordem,
                escala_x: *escala_x,
                escala_y: *escala_y,
                ruido,
                anim,
                erro_eval: None,
                trim_inicio: *trim_inicio,
                trim_fim: *trim_fim,
                duracao: self.projeto().duracao_seg,
            })
        } else {
            None
        }
    }

    fn find_connected_ruido(&self, nid: NodeId) -> Option<crate::procedural::RuidoDriver> {
        let graph = &self.editor_state.graph;
        let node = &graph[nid];

        for (_, input_id) in &node.inputs {
            if let Some(output_id) = graph.connection(*input_id) {
                let src_nid = graph.outputs[output_id].node;
                if let Some(NodeParams::Ruido { seed, freq, amp, veloc, alvo }) = self.params.get(&src_nid) {
                    return Some(crate::procedural::RuidoDriver {
                        seed: *seed,
                        freq: *freq,
                        amp: *amp,
                        veloc: *veloc,
                        alvo: *alvo,
                        comp: None,
                    });
                }
            }
        }
        None
    }

    fn find_connected_anim(&self, nid: NodeId) -> Option<crate::procedural::AnimDriver> {
        let graph = &self.editor_state.graph;
        let node = &graph[nid];

        for (_, input_id) in &node.inputs {
            if let Some(output_id) = graph.connection(*input_id) {
                let src_nid = graph.outputs[output_id].node;
                if let Some(NodeParams::Anim { alvo, loop_mode, segmentos }) = self.params.get(&src_nid) {
                    return Some(crate::procedural::AnimDriver {
                        segmentos: segmentos.clone(),
                        loop_mode: crate::procedural::LoopMode::from_u8(*loop_mode),
                        alvo: *alvo,
                        comp: None,
                    });
                }
            }
        }
        None
    }
}
