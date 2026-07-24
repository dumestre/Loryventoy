use super::{ParametroPorto, PortSpec, NodeParams, P_LAYER, P_POS_XY, P_TAMANHO, P_OPACIDADE, P_ESC_XY};
use eframe::egui::Color32;

static ENTRADAS: [ParametroPorto; 5] = [P_LAYER, P_POS_XY, P_TAMANHO, P_OPACIDADE, P_ESC_XY];
static SAIDAS: [ParametroPorto; 2] = [P_POS_XY, P_TAMANHO];

pub(crate) fn portos() -> PortSpec {
    PortSpec { entradas: &ENTRADAS, saidas: &SAIDAS }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Texto {
        cena: String::new(),
        conteudo: "Texto".to_string(),
        tamanho: 48.0,
        negrito: false,
        italico: false,
        px: 960.0,
        py: 540.0,
        cor: Color32::from_rgb(20, 20, 26),
        trim_inicio: 0.0,
        trim_fim: 1.0,
    }
}
