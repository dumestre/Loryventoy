use std::collections::HashMap;

use eframe::egui::Pos2;

use crate::domain::LayerEntry;
use crate::nodes::{NodeParams, TipoNo};

use super::types::NodeId;
use super::GraphPanel;

impl GraphPanel {
    pub fn cenas_disponiveis(&self) -> Vec<String> {
        let mut v: Vec<String> = self.params.iter()
            .filter_map(|(_, p)| {
                if let NodeParams::Cena(cena) = p {
                    if !cena.nome_cena.is_empty() { Some(cena.nome_cena.clone()) } else { None }
                } else { None }
            })
            .collect();
        v.sort();
        v.dedup();
        v
    }

    pub fn cenas_disponiveis_com_indice(&self) -> Vec<(String, NodeId)> {
        let mut v: Vec<(String, NodeId)> = self.params.iter()
            .filter_map(|(&idx, p)| {
                if let NodeParams::Cena(cena) = p {
                    if !cena.nome_cena.is_empty() { Some((cena.nome_cena.clone(), idx)) } else { None }
                } else { None }
            })
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v.dedup_by(|a, b| a.0 == b.0);
        v
    }

    pub fn normalizar_cena(&mut self, idx: NodeId, cenas: &[String], preferida: Option<String>) {
        if let Some(params) = self.params.get_mut(&idx) {
            let cena = match params {
                NodeParams::Layer(layer) => &mut layer.cena,
                NodeParams::Pen(pen) => &mut pen.cena,
                NodeParams::Texto(texto) => &mut texto.cena,
                NodeParams::Shape(shape) => &mut shape.cena,
                _ => return,
            };
            if cenas.iter().all(|c| c != cena) {
                *cena = preferida.or_else(|| cenas.first().cloned()).unwrap_or_default();
            }
        }
    }

    pub fn sync_layer_ports(&mut self) {
        let layer_nids: Vec<NodeId> = self.params.iter()
            .filter(|(_, p)| matches!(p, NodeParams::Layer(..)))
            .map(|(&nid, _)| nid)
            .collect();

        for nid in layer_nids {
            let entries: Vec<(String, f32)> = match self.params.get(&nid) {
                Some(NodeParams::Layer(layer)) => {
                    layer.layers.iter().map(|e| (e.nome.clone(), e.opacidade)).collect()
                }
                _ => continue,
            };

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

            let current_order: Vec<String> = self.editor_state.graph[nid].outputs.iter()
                .map(|(name, _)| name.clone())
                .collect();

            if current_order == desired {
                continue;
            }

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

            for (_, oid) in &current_outputs {
                self.editor_state.graph.remove_output_param(*oid);
            }

            for name in &desired {
                let new_oid = self.editor_state.graph.add_output_param(
                    nid,
                    name.clone(),
                    super::types::GraphDataType::Scalar,
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
                if let NodeParams::Cena(cena) = p {
                    if !cena.nome_cena.is_empty() { Some(cena.nome_cena.clone()) } else { None }
                } else { None }
            })
        }).or_else(|| cenas.first().cloned()).unwrap_or_default();

        let existing = self.params.iter().find_map(|(&idx, p)| {
            if let NodeParams::Layer(layer) = p {
                if layer.cena == cena_nome { Some(idx) } else { None }
            } else { None }
        });

        if let Some(layer_nid) = existing {
            let count = match self.params.get(&layer_nid) {
                Some(NodeParams::Layer(layer)) => layer.layers.len(),
                _ => 0,
            };
            if let Some(NodeParams::Layer(layer)) = self.params.get_mut(&layer_nid) {
                layer.layers.insert(0, LayerEntry {
                    nome: format!("Layer {}", count + 1),
                    ordem: 0.0,
                    opacidade: 1.0,
                    cor: LayerEntry::cor_por_idx(count),
                    visivel: true,
                });
            }
        } else {
            let loc = Pos2::new(
                (self.contador as f32 % 3.0) * 260.0,
                200.0 + (self.contador as f32 / 3.0) * 150.0,
            );
            let idx = self.adicionar_no_em(TipoNo::Layer, loc);
            if let Some(NodeParams::Layer(layer)) = self.params.get_mut(&idx) {
                layer.cena = cena_nome;
                layer.layers.insert(0, LayerEntry {
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
            Some(NodeParams::Layer(layer)) => {
                if entry_idx < layer.layers.len() && layer.layers.len() > 1 {
                    (false, true)
                } else if layer.layers.len() == 1 {
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
            if let Some(NodeParams::Layer(layer)) = self.params.get_mut(&layer_idx) {
                layer.layers.remove(entry_idx);
                if layer.selected >= layer.layers.len() {
                    layer.selected = layer.layers.len().saturating_sub(1);
                }
            }
        }
    }

    pub fn mover_layer_entry(&mut self, layer_idx: NodeId, entry_idx: usize, delta: i32) {
        if let Some(NodeParams::Layer(layer)) = self.params.get_mut(&layer_idx) {
            let new_idx = (entry_idx as i32 + delta) as usize;
            if new_idx < layer.layers.len() {
                layer.layers.swap(entry_idx, new_idx);
            }
        }
    }

    pub fn sincronizar_marcadores_com_cenas(&mut self, markers: &[crate::ui::timeline::Marker]) {
        self.empurrar_historico();
        let nomes_marc: Vec<String> = markers.iter().map(|m| m.name.clone()).collect();
        let mut cenas_por_nome: HashMap<String, NodeId> = HashMap::new();
        let mut cenas_para_remover: Vec<NodeId> = Vec::new();

        for (&idx, p) in &self.params {
            if let NodeParams::Cena(cena) = p {
                if !cena.nome_cena.is_empty() {
                    if nomes_marc.contains(&cena.nome_cena) {
                        cenas_por_nome.insert(cena.nome_cena.clone(), idx);
                    } else {
                        cenas_para_remover.push(idx);
                    }
                }
            }
        }
        for idx in cenas_para_remover {
            if self.params.get(&idx).map_or(false, |p| matches!(p, NodeParams::Cena(..))) {
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
                if let Some(NodeParams::Cena(cena)) = self.params.get_mut(&idx) {
                    cena.nome_cena = nome.clone();
                }
                cenas_por_nome.insert(nome.clone(), idx);
                self.contador += 1;
            }
        }
    }
}
