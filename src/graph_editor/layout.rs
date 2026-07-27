use eframe::egui::{Pos2, Rect, Vec2};

use super::ports;
use super::types::NodeId;
use super::GraphPanel;

#[allow(dead_code)]
impl GraphPanel {
    pub fn screen_para_canvas(&self, screen: Pos2, pan: Vec2, editor_rect: Rect) -> Pos2 {
        (screen.to_vec2() - pan - editor_rect.min.to_vec2()).to_pos2()
    }

    pub fn canvas_para_screen(&self, canvas: Pos2, pan: Vec2, zoom: f32, editor_rect: Rect) -> Pos2 {
        let center = editor_rect.center().to_vec2();
        ((canvas.to_vec2() - center) * zoom + center + pan + editor_rect.min.to_vec2()).to_pos2()
    }

    pub fn node_sob_cursor(&self, p: Pos2, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<NodeId> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
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

    pub fn sobre_cabecalho_no(&self, p: Pos2, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<NodeId> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
        for nid in self.editor_state.graph.iter_nodes() {
            if let Some(pos) = self.editor_state.node_positions.get(nid) {
                let label = self.obter_label(nid);
                let half = ports::tamanho(&label);
                let header_h = super::node_component::CABECALHO_H;
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

    pub fn portos_offsets(&self, idx: NodeId) -> Option<(Vec<Vec2>, Vec<Vec2>)> {
        let label = self.obter_label(idx);
        let half = ports::tamanho(&label);
        let tipo = self.obter_tipo(idx);
        Some(ports::port_offsets(tipo, half))
    }

    pub fn porta_saida_mais_proxima(&self, p: Pos2, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<(NodeId, usize)> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
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

    pub fn porta_entrada_mais_proxima(&self, p: Pos2, max: f32, pan: Vec2, _zoom: f32, editor_rect: Rect) -> Option<(NodeId, usize)> {
        let canvas_p = self.screen_para_canvas(p, pan, editor_rect);
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

    pub fn porta_entrada_canvas(&self, idx: NodeId, porta: usize) -> Option<Pos2> {
        let pos = self.editor_state.node_positions.get(idx).copied()?;
        let label = self.obter_label(idx);
        let half = ports::tamanho(&label);
        let tipo = self.obter_tipo(idx);
        let (ins, _) = ports::port_offsets(tipo, half);
        ins.get(porta).map(|off| pos + *off)
    }

    pub fn reafirmar_posicoes(&mut self) {
        let fixar = |idx: Option<NodeId>, loc: Pos2, liberados: &std::collections::HashSet<NodeId>, positions: &mut slotmap::SecondaryMap<NodeId, Pos2>| {
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
}
