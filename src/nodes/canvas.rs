use super::{NodeParams, PortSpec, ProjetoConfig};

pub(crate) fn portos() -> PortSpec {
    PortSpec {
        entradas: &[],
        saidas: &[],
    }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Canvas(ProjetoConfig::default())
}
