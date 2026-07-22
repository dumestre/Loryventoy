use crate::nodes::TipoNo;
use crate::procedural::{PreviewData, CenaPreview};

use super::GraphPanel;


impl GraphPanel {
    pub fn formas_para_preview(&self) -> PreviewData {
        let mut preview = PreviewData::default();
        let cfg = self.projeto();
        preview.largura = cfg.largura as f32;
        preview.altura = cfg.altura as f32;
        preview.fundo = cfg.fundo;

        for (&nid, _p) in &self.params {
            let node = &self.editor_state.graph[nid];
            if node.user_data.tipo == TipoNo::Cena {
                preview.cenas.push(CenaPreview::default());
            }
        }

        preview
    }
}
