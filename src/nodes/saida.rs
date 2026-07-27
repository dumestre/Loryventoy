use super::{ParametroPorto, PortSpec, NodeParams, SaidaParams, P_CENA};

static ENTRADAS: [ParametroPorto; 1] = [P_CENA];

pub(crate) fn portos() -> PortSpec {
    PortSpec { entradas: &ENTRADAS, saidas: &[] }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Saida(SaidaParams {
        brilho: 1.0,
        contraste: 1.0,
        saturacao: 1.0,
    })
}
