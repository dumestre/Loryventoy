use super::{NodeParams, ParametroPorto, PortSpec, TipoPorto, TransformParams, COMP_XYZ};

static P_POS: ParametroPorto = ParametroPorto {
    nome: "Posição",
    tipo: TipoPorto::Vetor(COMP_XYZ),
};
static P_ROT: ParametroPorto = ParametroPorto {
    nome: "Rotação",
    tipo: TipoPorto::Vetor(COMP_XYZ),
};
static P_ESC: ParametroPorto = ParametroPorto {
    nome: "Escala",
    tipo: TipoPorto::Vetor(COMP_XYZ),
};

static ENTRADAS: [ParametroPorto; 3] = [P_POS, P_ROT, P_ESC];
static SAIDAS: [ParametroPorto; 3] = [P_POS, P_ROT, P_ESC];

pub(crate) fn portos() -> PortSpec {
    PortSpec {
        entradas: &ENTRADAS,
        saidas: &SAIDAS,
    }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Transform(TransformParams {
        px: 0.0,
        py: 0.0,
        pz: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,
        sx: 1.0,
        sy: 1.0,
        sz: 1.0,
    })
}
