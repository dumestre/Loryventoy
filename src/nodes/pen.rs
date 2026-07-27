use super::{ParametroPorto, PortSpec, NodeParams, P_LAYER, P_POS_XY, P_PEN};
use crate::domain::Color;

pub(crate) const PEN_EXEMPLO: &str = "\
# estrela de 5 pontas
let ra = 200
let rb = 80
move 0 (-ra)
repeat 5 {
  let a = i * 72
  line (cos(a)*ra) (sin(a)*ra)
  let b = a + 36
  line (cos(b)*rb) (sin(b)*rb)
}
close
fill on
color 0.78 0.47 0.08
";

static ENTRADAS: [ParametroPorto; 2] = [P_LAYER, P_POS_XY];
static SAIDAS: [ParametroPorto; 2] = [P_PEN, P_POS_XY];

pub(crate) fn portos() -> PortSpec {
    PortSpec { entradas: &ENTRADAS, saidas: &SAIDAS }
}

pub(crate) fn padrao() -> NodeParams {
    NodeParams::Pen {
        cena: String::new(),
        codigo: PEN_EXEMPLO.to_string(),
        erro: None,
        cor: Color::from_rgba(200, 120, 220, 255),
        cor_fill: Color::from_rgba(200, 120, 220, 255),
        pos_x: 960.0,
        pos_y: 540.0,
        espessura: 3.0,
        preenchimento: true,
        seed: 1.0,
        cantos: 0.0,
        ordem: 0.0,
        escala_x: 1.0,
        escala_y: 1.0,
        trim_inicio: 0.0,
        trim_fim: 1.0,
    }
}
