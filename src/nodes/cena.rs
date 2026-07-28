use super::{CenaParams, NodeParams, ParametroPorto, PortSpec, P_CANVAS, P_CENA};

static ENTRADAS: [ParametroPorto; 1] = [P_CANVAS];
static SAIDAS: [ParametroPorto; 1] = [P_CENA];

pub(crate) fn portos() -> PortSpec {
    PortSpec {
        entradas: &ENTRADAS,
        saidas: &SAIDAS,
    }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Cena(CenaParams {
        nome_cena: "Cena 1".to_string(),
        ativa: true,
        zoom: 1.0,
        angulo: 0.0,
        opacidade: 1.0,
    })
}
