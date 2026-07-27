use std::collections::HashSet;

use eframe::egui::Pos2;

use crate::nodes::{self, NodeParams, TipoNo};

use super::GraphPanel;
use super::types::NodeId;
use super::groups::{CORES_GRUPO, Grupo};

#[derive(Clone)]
pub struct NoCopia {
    pub tipo: TipoNo,
    pub params: NodeParams,
    pub offset: eframe::egui::Vec2,
}

#[derive(Clone, Copy)]
pub enum AcaoMenu {
    Copiar,
    Colar,
    Duplicar,
    Deletar,
    Agrupar,
}

impl GraphPanel {
    pub fn selecionados(&self) -> Vec<NodeId> {
        self.editor_state
            .selected_nodes
            .clone()
    }

    pub fn selecionar_no(&mut self, idx: NodeId, adicionar: bool) {
        if let Some(tipo) = self.tipo_do_node(idx) {
            if tipo == TipoNo::Cena && !adicionar {
                self.cena_ativa = Some(idx);
            }
        }
        if adicionar {
            if let Some(pos) = self.editor_state.selected_nodes.iter().position(|&n| n == idx) {
                self.editor_state.selected_nodes.remove(pos);
            } else {
                self.editor_state.selected_nodes.push(idx);
            }
            return;
        }
        self.editor_state.selected_nodes.clear();
        self.editor_state.selected_nodes.push(idx);
    }

    pub fn centro_selecionados(&self) -> Option<Pos2> {
        let sel = self.selecionados();
        let mut soma = eframe::egui::Vec2::ZERO;
        let mut n = 0.0;
        for idx in &sel {
            if let Some(pos) = self.editor_state.node_positions.get(*idx) {
                soma += pos.to_vec2();
                n += 1.0;
            }
        }
        if n == 0.0 {
            None
        } else {
            Some((soma / n).to_pos2())
        }
    }

    pub fn agrupar_selecionados(&mut self) {
        let sel = self.selecionados();
        if sel.is_empty() {
            return;
        }
        let cor = CORES_GRUPO[self.grupo_seq % CORES_GRUPO.len()];
        self.grupo_seq += 1;
        self.grupos.push(Grupo {
            nos: sel,
            titulo: format!("Grupo {}", self.grupo_seq),
            cor,
            handle: eframe::egui::Rect::ZERO,
        });
    }

    pub fn copiar_selecionados(&mut self) {
        let centro = match self.centro_selecionados() {
            Some(c) => c,
            None => return,
        };
        let mut itens = Vec::new();
        for idx in self.selecionados() {
            let tipo = match self.tipo_do_node(idx) {
                Some(t) => t,
                None => continue,
            };
            if tipo == TipoNo::Saida || tipo == TipoNo::Canvas {
                continue;
            }
            let params = self.obter_params(idx)
                .cloned()
                .unwrap_or_else(|| nodes::node_params_padrao(tipo));
            let loc = self.editor_state.node_positions.get(idx).copied().unwrap_or(Pos2::ZERO);
            itens.push(NoCopia {
                tipo,
                params,
                offset: loc - centro,
            });
        }
        if !itens.is_empty() {
            self.clipboard = itens;
        }
    }

    pub fn colar_em(&mut self, pos: Pos2) {
        if self.clipboard.is_empty() {
            return;
        }
        let itens = self.clipboard.clone();
        self.editor_state.selected_nodes.clear();
        for it in itens {
            let idx = self.adicionar_no_em(it.tipo, pos + it.offset);
            self.definir_params(idx, it.params);
            self.liberados.insert(idx);
            self.editor_state.selected_nodes.push(idx);
        }
    }

    pub fn duplicar_selecionados(&mut self) {
        self.copiar_selecionados();
        if let Some(centro) = self.centro_selecionados() {
            self.colar_em(centro + eframe::egui::Vec2::new(30.0, 30.0));
        }
    }

    pub fn deletar_selecionados(&mut self) {
        for idx in self.selecionados() {
            if self.is_fixo(idx) {
                continue;
            }
            self.remover_no(idx);
        }
        self.limpar_grupos();
    }

    pub fn limpar_grupos(&mut self) {
        let vivos: HashSet<NodeId> =
            self.editor_state.graph.iter_nodes().collect();
        for grp in &mut self.grupos {
            grp.nos.retain(|i| vivos.contains(i));
        }
        self.grupos.retain(|g| !g.nos.is_empty());
    }
}
