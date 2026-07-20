use petgraph::stable_graph::NodeIndex;
use std::collections::HashSet;

use crate::nodes::{NodeParams, TipoNo};
use crate::graph_editor::groups::{CORES_GRUPO, Grupo};

use super::GraphPanel;

/// Nó copiado para a área de transferência (tipo + parâmetros + posição
/// relativa ao centro do conjunto copiado).
#[derive(Clone)]
pub struct NoCopia {
    pub tipo: TipoNo,
    pub params: NodeParams,
    pub offset: eframe::egui::Vec2,
}

/// Ação escolhida no menu de contexto (botão direito).
#[derive(Clone, Copy)]
pub enum AcaoMenu {
    Copiar,
    Colar,
    Duplicar,
    Deletar,
    Agrupar,
}

impl GraphPanel {
    /// Índices dos nós atualmente selecionados.
    pub fn selecionados(&self) -> Vec<NodeIndex> {
        self.g
            .nodes_iter()
            .filter(|(_, n)| n.selected())
            .map(|(i, _)| i)
            .collect()
    }

    /// Seleciona um nó. Se `adicionar` (shift), alterna a seleção desse nó;
    /// caso contrário, limpa as demais e seleciona só este.
    /// Se o nó for do tipo Cena, também o define como cena ativa.
    pub fn selecionar_no(&mut self, idx: NodeIndex, adicionar: bool) {
        if let Some(tipo) = self.tipo_do_node(idx) {
            if tipo == TipoNo::Cena && !adicionar {
                self.cena_ativa = Some(idx);
            }
        }
        if adicionar {
            if let Some(n) = self.g.node_mut(idx) {
                let sel = !n.selected();
                n.set_selected(sel);
            }
            return;
        }
        let todos: Vec<NodeIndex> = self.g.nodes_iter().map(|(i, _)| i).collect();
        for i in todos {
            if let Some(n) = self.g.node_mut(i) {
                n.set_selected(i == idx);
            }
        }
    }

    /// Centro (canvas) dos nós selecionados.
    pub fn centro_selecionados(&self) -> Option<eframe::egui::Pos2> {
        let sel = self.selecionados();
        let mut soma = eframe::egui::Vec2::ZERO;
        let mut n = 0.0;
        for idx in &sel {
            if let Some(node) = self.g.node(*idx) {
                soma += node.location().to_vec2();
                n += 1.0;
            }
        }
        if n == 0.0 {
            None
        } else {
            Some((soma / n).to_pos2())
        }
    }

    /// Agrupa os nós selecionados (>= 1) num novo grupo com cor automática.
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

    /// Copia os nós selecionados (exceto fixos/únicos) para a área de
    /// transferência, guardando a posição relativa ao centro do conjunto.
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
            // Saída (master) e Canvas são únicos/fixos: não copiáveis
            if tipo == TipoNo::Saida || tipo == TipoNo::Canvas {
                continue;
            }
            let params = self
                .params
                .get(&idx)
                .cloned()
                .unwrap_or_else(|| NodeParams::padrao(tipo));
            let loc = self.g.node(idx).unwrap().location();
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

    /// Cola os nós da área de transferência com o centro em `pos` (canvas),
    /// deixando os novos nós selecionados.
    pub fn colar_em(&mut self, pos: eframe::egui::Pos2) {
        if self.clipboard.is_empty() {
            return;
        }
        let itens = self.clipboard.clone();
        // desmarca a seleção atual
        for idx in self.selecionados() {
            if let Some(n) = self.g.node_mut(idx) {
                n.set_selected(false);
            }
        }
        for it in itens {
            let idx = self.adicionar_no_em(it.tipo, pos + it.offset);
            self.params.insert(idx, it.params);
            self.liberados.insert(idx);
            if let Some(n) = self.g.node_mut(idx) {
                n.set_selected(true);
            }
        }
    }

    /// Duplica os nós selecionados com um pequeno deslocamento.
    pub fn duplicar_selecionados(&mut self) {
        self.copiar_selecionados();
        if let Some(centro) = self.centro_selecionados() {
            self.colar_em(centro + eframe::egui::Vec2::new(30.0, 30.0));
        }
    }

    /// Remove os nós selecionados (exceto fixos: master e Canvas).
    pub fn deletar_selecionados(&mut self) {
        for idx in self.selecionados() {
            if self.is_fixo(idx) {
                continue;
            }
            self.g.remove_node(idx);
            self.params.remove(&idx);
            self.liberados.remove(&idx);
        }
        self.limpar_grupos();
    }

    /// Descarta de cada grupo os nós que já não existem; remove grupos vazios.
    pub fn limpar_grupos(&mut self) {
        let vivos: HashSet<NodeIndex> =
            self.g.nodes_iter().map(|(i, _)| i).collect();
        for grp in &mut self.grupos {
            grp.nos.retain(|i| vivos.contains(i));
        }
        self.grupos.retain(|g| !g.nos.is_empty());
    }
}
