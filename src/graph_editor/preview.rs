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
                    if let NodeParams::Pen(pen) = p {
                        Some((nid, pen.codigo.clone()))
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
        preview.fundo = eframe::egui::Color32::from_rgba_unmultiplied(
            cfg.fundo.r,
            cfg.fundo.g,
            cfg.fundo.b,
            cfg.fundo.a,
        );

        // Map Layer NodeId -> cena name
        let mut layer_to_cena: HashMap<NodeId, String> = HashMap::new();
        for (&nid, params) in &self.params {
            if let NodeParams::Layer(layer) = params {
                layer_to_cena.insert(nid, layer.cena.clone());
            }
        }

        // Group scenes by name
        let mut cena_names: Vec<(String, NodeId)> = Vec::new();
        for (&nid, params) in &self.params {
            if let NodeParams::Cena(cena) = params {
                cena_names.push((cena.nome_cena.clone(), nid));
            }
        }

        // For each scene, build CenaPreview
        for (nome_cena, cena_nid) in &cena_names {
            let cena_opac = match self.params.get(cena_nid) {
                Some(NodeParams::Cena(cena)) => cena.opacidade,
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
                    Some(NodeParams::Layer(layer)) => {
                        layer.layers.iter().enumerate()
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
                                NodeParams::Shape(..) | NodeParams::Texto(..) | NodeParams::Pen(..) => {
                                    let graph = &self.editor_state.graph;
                                    let node = &graph[nid];
                                    for (_, input_id) in &node.inputs {
                                        if let Some(connected_out) = graph.connection(*input_id) {
                                            if connected_out == oid {
                                                has_connections = true;
                                                match params {
                                                    NodeParams::Shape(..) => {
                                                        if let Some(gen) = self.build_shape_generator(nid, params) {
                                                            layer_preview.formas.push(gen);
                                                        }
                                                    }
                                                    NodeParams::Texto(..) => {
                                                        if let Some(item) = self.build_texto_item(nid, params) {
                                                            layer_preview.textos.push(item);
                                                        }
                                                    }
                                                    NodeParams::Pen(..) => {
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
                                NodeParams::Shape(..) | NodeParams::Texto(..) | NodeParams::Pen(..) => {
                                    let node_cena = match params {
                                        NodeParams::Shape(shape) => shape.cena.as_str(),
                                        NodeParams::Texto(texto) => texto.cena.as_str(),
                                        NodeParams::Pen(pen) => pen.cena.as_str(),
                                        _ => "",
                                    };
                                    if node_cena == nome_cena {
                                        match params {
                                            NodeParams::Shape(..) => {
                                                if let Some(gen) = self.build_shape_generator(nid, params) {
                                                    layer_preview.formas.push(gen);
                                                }
                                            }
                                            NodeParams::Texto(..) => {
                                                if let Some(item) = self.build_texto_item(nid, params) {
                                                    layer_preview.textos.push(item);
                                                }
                                            }
                                            NodeParams::Pen(..) => {
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
                        NodeParams::Shape(..) | NodeParams::Texto(..) | NodeParams::Pen(..) => {
                            let node_cena = match params {
                                NodeParams::Shape(shape) => shape.cena.as_str(),
                                NodeParams::Texto(texto) => texto.cena.as_str(),
                                NodeParams::Pen(pen) => pen.cena.as_str(),
                                _ => "",
                            };
                            if node_cena == nome_cena {
                                match params {
                                    NodeParams::Shape(..) => {
                                        if let Some(gen) = self.build_shape_generator(nid, params) {
                                            layer_preview.formas.push(gen);
                                        }
                                    }
                                    NodeParams::Texto(..) => {
                                        if let Some(item) = self.build_texto_item(nid, params) {
                                            layer_preview.textos.push(item);
                                        }
                                    }
                                    NodeParams::Pen(..) => {
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
        if let NodeParams::Shape(shape) = params {
            let kind = ShapeKind::from_u8(shape.tipo);

            // Check for connected Ruido/Anim nodes
            let ruido = self.find_connected_ruido(_nid);
            let anim = self.find_connected_anim(_nid);

            Some(ShapeGenerator {
                kind,
                pos: GVec2::new(shape.px, shape.py),
                tam: GVec2::new(shape.largura, shape.altura),
                rot: shape.rotacao,
                cor: eframe::egui::Color32::from_rgba_unmultiplied(shape.cor.r, shape.cor.g, shape.cor.b, shape.cor.a),
                seed: shape.seed,
                noise_scale: shape.noise_scale,
                amp: shape.amp,
                veloc: shape.veloc,
                ruido,
                anim,
                trim_inicio: shape.trim_inicio,
                trim_fim: shape.trim_fim,
                duracao: self.projeto().duracao_seg,
            })
        } else {
            None
        }
    }

    fn build_texto_item(&self, _nid: NodeId, params: &NodeParams) -> Option<TextoItem> {
        if let NodeParams::Texto(texto) = params {
            let ruido = self.find_connected_ruido(_nid);
            let anim = self.find_connected_anim(_nid);

            Some(TextoItem {
                px: texto.px,
                py: texto.py,
                conteudo: texto.conteudo.clone(),
                tamanho: texto.tamanho,
                negrito: texto.negrito,
                italico: texto.italico,
                cor: eframe::egui::Color32::from_rgba_unmultiplied(texto.cor.r, texto.cor.g, texto.cor.b, texto.cor.a),
                escala_x: 1.0,
                escala_y: 1.0,
                ruido,
                anim,
                trim_inicio: texto.trim_inicio,
                trim_fim: texto.trim_fim,
            })
        } else {
            None
        }
    }

    fn build_pen_path(&self, _nid: NodeId, params: &NodeParams) -> Option<PenPath> {
        if let NodeParams::Pen(pen) = params {
            let program = self.pen_cache.get(&pen.codigo)
                .cloned()
                .unwrap_or_default();

            let ruido = self.find_connected_ruido(_nid);
            let anim = self.find_connected_anim(_nid);

            Some(PenPath {
                program,
                pos: GVec2::new(pen.pos_x, pen.pos_y),
                cor: eframe::egui::Color32::from_rgba_unmultiplied(pen.cor.r, pen.cor.g, pen.cor.b, pen.cor.a),
                cor_fill: eframe::egui::Color32::from_rgba_unmultiplied(pen.cor_fill.r, pen.cor_fill.g, pen.cor_fill.b, pen.cor_fill.a),
                espessura: pen.espessura,
                preenchimento: pen.preenchimento,
                seed: pen.seed as u32,
                cantos: pen.cantos,
                ordem: pen.ordem,
                escala_x: pen.escala_x,
                escala_y: pen.escala_y,
                ruido,
                anim,
                erro_eval: None,
                trim_inicio: pen.trim_inicio,
                trim_fim: pen.trim_fim,
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
                if let Some(NodeParams::Ruido(ruido)) = self.params.get(&src_nid) {
                    return Some(crate::procedural::RuidoDriver {
                        seed: ruido.seed,
                        freq: ruido.freq,
                        amp: ruido.amp,
                        veloc: ruido.veloc,
                        alvo: ruido.alvo,
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
                if let Some(NodeParams::Anim(anim)) = self.params.get(&src_nid) {
                    return Some(crate::procedural::AnimDriver {
                        segmentos: anim.segmentos.clone(),
                        loop_mode: crate::domain::LoopMode::from_u8(anim.loop_mode),
                        alvo: anim.alvo,
                        comp: None,
                    });
                }
            }
        }
        None
    }
}
