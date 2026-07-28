use super::GraphPanel;

impl GraphPanel {
    pub fn buscar(&mut self, termo: &str) {
        for nid in self.editor_state.graph.iter_nodes() {
            let node = &self.editor_state.graph[nid];
            let selected = node
                .user_data
                .tipo
                .nome()
                .to_lowercase()
                .contains(&termo.to_lowercase());
            if selected {
                if !self.editor_state.selected_nodes.contains(&nid) {
                    self.editor_state.selected_nodes.push(nid);
                }
            } else {
                self.editor_state.selected_nodes.retain(|&n| n != nid);
            }
        }
    }
}
