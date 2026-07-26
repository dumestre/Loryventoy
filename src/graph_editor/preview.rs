use std::collections::HashMap;

use glam::Vec2 as GVec2;

use crate::nodes::NodeParams;
use crate::procedural::{
    CenaPreview, LayerPreview, PreviewData, ShapeGenerator, ShapeKind, TextoItem, PenPath,
};

use super::types::NodeId;
use super::GraphPanel;

impl GraphPanel {
    pub fn formas_para_preview(&mut self) -> Option<PreviewData> {
        if !self.preview_dirty {
            return None;
        }
        self.preview_dirty = false;

        // Pre-popula cache de pen DSL antes de iterar params (evita conflito de
        // borrow entre &self.params e &mut self.pen_cache dentro dos loops).
        {
            let pen_codes: Vec<(NodeId, String)> = self.params.iter()
                .filter_map(|(&nid, p)| {
                    if let NodeParams::Pen { codigo, .. } = p {
                        Some((nid, codigo.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            for (_nid, codigo) in &pen_codes {
                if !self.pen_cache.contains_key(codigo) {
                    let program = crate::dsl::Program::parse(codigo).unwrap_or_default();
                    if self.pen_cache.len() >= 256 {
                        self.pen_cache.clear();
                    }
                    self.pen_cache.insert(codigo.clone(), program);
                }
            }
        }

        let mut preview = PreviewData::default();
        let cfg = self.projeto();
        preview.largura = cfg.largura as f32;
        preview.altura = cfg.altura as f32;
        preview.fundo = cfg.fundo;

        // Map Layer NodeId -> cena name
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

            // Collect Layer nodes belonging to this scene
            let layer_nids: Vec<NodeId> = layer_to_cena.iter()
                .filter(|(_, c)| *c == nome_cena)
                .map(|(&nid, _)| nid)
                .collect();

            // For each Layer node, iterate its internal layer entries
            for &layer_nid in &layer_nids {
                let entries: Vec<(usize, String, f32, f32, bool)> = match self.params.get(&layer_nid) {
                    Some(NodeParams::Layer { layers, .. }) => {
                        layers.iter().enumerate()
                            .map(|(i, e)| (i, e.nome.clone(), e.ordem, e.opacidade, e.visivel))
                            .collect()
                    }
                    _ => continue,
                };

                for (_entry_idx, nome, _ordem, opac, visivel) in &entries {
                    if !visivel {
                        continue;
                    }
                    let mut layer_preview = LayerPreview {
                        nome: nome.clone(),
                        opacidade: *opac,
                        formas: Vec::new(),
                        textos: Vec::new(),
                        pen: Vec::new(),
                    };

                    // Find all nodes connected to this layer entry's output port
                    let output_port_name = layer_preview.nome.clone();
                    let output_id = self.editor_state.graph[layer_nid].outputs.iter()
                        .find(|(name, _)| *name == output_port_name)
                        .map(|(_, id)| *id);

                    // Check if any Shape/Texto/Pen is connected to this output
                    let mut has_connections = false;
                    if let Some(oid) = output_id {
                        for (&nid, params) in &self.params {
                            match params {
                                NodeParams::Shape { .. } | NodeParams::Texto { .. } | NodeParams::Pen { .. } => {
                                    let graph = &self.editor_state.graph;
                                    let node = &graph[nid];
                                    for (_, input_id) in &node.inputs {
                                        if let Some(connected_out) = graph.connection(*input_id) {
                                            if connected_out == oid {
                                                has_connections = true;
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
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    // Fallback: if no connections and only one layer entry, include all scene nodes
                    if !has_connections && entries.len() <= 1 {
                        for (&nid, params) in &self.params {
                            match params {
                                NodeParams::Shape { .. } | NodeParams::Texto { .. } | NodeParams::Pen { .. } => {
                                    let node_cena = match params {
                                        NodeParams::Shape { cena, .. }
                                        | NodeParams::Texto { cena, .. }
                                        | NodeParams::Pen { cena, .. } => cena.as_str(),
                                        _ => "",
                                    };
                                    if node_cena == nome_cena {
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
                    }

                    cena_preview.layers.push(layer_preview);
                }
            }

            // If no layers exist, create a default one with all scene nodes
            if cena_preview.layers.is_empty() {
                let mut layer_preview = LayerPreview {
                    nome: "Layer 1".to_string(),
                    opacidade: 1.0,
                    formas: Vec::new(),
                    textos: Vec::new(),
                    pen: Vec::new(),
                };
                for (&nid, params) in &self.params {
                    match params {
                        NodeParams::Shape { .. } | NodeParams::Texto { .. } | NodeParams::Pen { .. } => {
                            let node_cena = match params {
                                NodeParams::Shape { cena, .. }
                                | NodeParams::Texto { cena, .. }
                                | NodeParams::Pen { cena, .. } => cena.as_str(),
                                _ => "",
                            };
                            if node_cena == nome_cena {
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

        Some(preview)
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
            let program = self.pen_cache.get(codigo)
                .cloned()
                .unwrap_or_default();

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
