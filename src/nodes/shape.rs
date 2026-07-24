use super::{ParametroPorto, TipoPorto, PortSpec, NodeParams, P_LAYER, P_POS_XY, P_LARGURA, P_ALTURA};
use eframe::egui::Color32;

static P_ROT: ParametroPorto = ParametroPorto { nome: "Rotação", tipo: TipoPorto::Escalar };
static ENTRADAS: [ParametroPorto; 5] = [P_LAYER, P_POS_XY, P_LARGURA, P_ALTURA, P_ROT];
static SAIDAS: [ParametroPorto; 4] = [P_POS_XY, P_LARGURA, P_ALTURA, P_ROT];

pub(crate) fn portos() -> PortSpec {
    PortSpec { entradas: &ENTRADAS, saidas: &SAIDAS }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Shape {
        cena: String::new(),
        tipo: 0,
        px: 960.0,
        py: 540.0,
        largura: 200.0,
        altura: 200.0,
        rotacao: 0.0,
        cor: Color32::from_rgb(235, 150, 120),
        seed: 1.0,
        noise_scale: 0.6,
        amp: 0.0,
        veloc: 0.0,
        trim_inicio: 0.0,
        trim_fim: 1.0,
    }
}
