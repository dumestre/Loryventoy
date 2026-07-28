use super::{AnimParams, NodeParams, ParametroPorto, PortSpec, P_ANIM_OUT};

static SAIDAS: [ParametroPorto; 1] = [P_ANIM_OUT];

pub(crate) fn portos() -> PortSpec {
    PortSpec {
        entradas: &[],
        saidas: &SAIDAS,
    }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Anim(AnimParams {
        alvo: 0,
        loop_mode: 0,
        segmentos: vec![crate::domain::AnimSeg {
            t_ini: 0.0,
            t_fim: 1.0,
            v_ini: [960.0, 540.0],
            v_fim: [960.0, 540.0],
            easing: crate::domain::Easing::EaseInOut,
        }],
    })
}
