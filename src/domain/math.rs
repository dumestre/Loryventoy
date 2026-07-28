/// Módulo de tipos matemáticos simples do domínio.
pub use super::color::{Pos2, Vec2};

/// Funções utilitárias de geometria.
pub fn retangulo_rot(c: Pos2, tam: Vec2, rot: f32) -> Vec<Pos2> {
    let rad = rot.to_radians();
    let (co, si) = (rad.cos(), rad.sin());
    let hw = tam.x * 0.5;
    let hh = tam.y * 0.5;
    [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
        .into_iter()
        .map(|(x, y)| Pos2::new(c.x + x * co - y * si, c.y + x * si + y * co))
        .collect()
}

pub fn elipse_rot(c: Pos2, rx: f32, ry: f32, rot: f32) -> Vec<Pos2> {
    let rad = rot.to_radians();
    let (co, si) = (rad.cos(), rad.sin());
    let n = 48;
    (0..n)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / n as f32;
            let x = rx * a.cos();
            let y = ry * a.sin();
            Pos2::new(c.x + x * co - y * si, c.y + x * si + y * co)
        })
        .collect()
}

pub fn poligono_regular(c: Pos2, r: f32, n: usize, rot: f32) -> Vec<Pos2> {
    let rad = rot.to_radians();
    (0..n)
        .map(|i| {
            let a = rad + i as f32 * std::f32::consts::TAU / n as f32 - std::f32::consts::FRAC_PI_2;
            Pos2::new(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

pub fn estrela(c: Pos2, ro: f32, ri: f32, pontas: usize, rot: f32) -> Vec<Pos2> {
    let rad = rot.to_radians();
    let total = pontas * 2;
    (0..total)
        .map(|i| {
            let r = if i % 2 == 0 { ro } else { ri };
            let a =
                rad + i as f32 * std::f32::consts::TAU / total as f32 - std::f32::consts::FRAC_PI_2;
            Pos2::new(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}
