use super::{PortSpec, NodeParams, LayerEntry, LayerParams};

pub(crate) fn portos() -> PortSpec {
    PortSpec { entradas: &[], saidas: &[] }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Layer(LayerParams {
        cena: String::new(),
        layers: vec![LayerEntry {
            nome: "Layer 1".to_string(),
            ordem: 0.0,
            opacidade: 1.0,
            cor: LayerEntry::cor_por_idx(0),
            visivel: true,
        }],
        selected: 0,
    })
}
