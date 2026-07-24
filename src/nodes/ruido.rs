use super::{ParametroPorto, PortSpec, NodeParams, P_RUIDO_OUT};

static SAIDAS: [ParametroPorto; 1] = [P_RUIDO_OUT];

pub(crate) fn portos() -> PortSpec {
    PortSpec { entradas: &[], saidas: &SAIDAS }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Ruido {
        seed: 1.0,
        freq: 0.6,
        amp: 50.0,
        veloc: 1.0,
        alvo: 0,
    }
}
