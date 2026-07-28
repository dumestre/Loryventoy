//! Tipos e avaliação procedural puros do domínio.
//! Não depende de `egui`, apenas de `glam` e `noise`.

use crate::domain::{AnimSeg, Color, LoopMode, Pos2, Vec2};
use crate::dsl::Program;
use noise::{NoiseFn, Simplex};

/// Tipos de forma suportados pelo gerador procedural.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Retangulo,
    Elipse,
    Triangulo,
    Estrela,
    Losango,
    Poligono,
    Seta,
}

impl ShapeKind {
    pub fn from_u8(v: u8) -> ShapeKind {
        match v {
            1 => ShapeKind::Elipse,
            2 => ShapeKind::Triangulo,
            3 => ShapeKind::Estrela,
            4 => ShapeKind::Losango,
            5 => ShapeKind::Poligono,
            6 => ShapeKind::Seta,
            _ => ShapeKind::Retangulo,
        }
    }
}

/// Modulação por um nó Ruído conectado a um parâmetro deste item.
#[derive(Debug, Clone, Copy)]
pub struct RuidoDriver {
    pub seed: f32,
    pub freq: f32,
    pub amp: f32,
    pub veloc: f32,
    pub alvo: u8,
    pub comp: Option<usize>,
}

/// Driver de animação conectado a um parâmetro de um item.
#[derive(Debug, Clone)]
pub struct AnimDriver {
    pub segmentos: Vec<AnimSeg>,
    pub loop_mode: LoopMode,
    pub alvo: u8,
    pub comp: Option<usize>,
}

impl AnimDriver {
    fn duracao(&self) -> f32 {
        self.segmentos.iter().map(|s| s.t_fim).fold(0.0, f32::max)
    }

    pub fn valor(&self, t: f32) -> [f32; 2] {
        if self.segmentos.is_empty() {
            return [0.0, 0.0];
        }
        let dur = self.duracao();
        let tt = match self.loop_mode {
            LoopMode::Nenhum => t,
            LoopMode::Repetir if dur > 0.0 => t.rem_euclid(dur),
            LoopMode::PingPong if dur > 0.0 => {
                let m = t.rem_euclid(2.0 * dur);
                if m <= dur {
                    m
                } else {
                    2.0 * dur - m
                }
            }
            _ => t,
        };
        let primeiro = &self.segmentos[0];
        if tt <= primeiro.t_ini {
            return primeiro.v_ini;
        }
        for s in &self.segmentos {
            if tt >= s.t_ini && tt <= s.t_fim {
                let dur_s = (s.t_fim - s.t_ini).max(1e-6);
                let x = ((tt - s.t_ini) / dur_s).clamp(0.0, 1.0);
                let k = s.easing.aplicar(x);
                return [
                    s.v_ini[0] + (s.v_fim[0] - s.v_ini[0]) * k,
                    s.v_ini[1] + (s.v_fim[1] - s.v_ini[1]) * k,
                ];
            }
        }
        self.segmentos.last().map(|s| s.v_fim).unwrap_or([0.0, 0.0])
    }
}

/// Item de texto a desenhar no preview, em coordenadas de projeto.
#[derive(Debug, Clone)]
pub struct TextoItem {
    pub px: f32,
    pub py: f32,
    pub conteudo: String,
    pub tamanho: f32,
    pub negrito: bool,
    pub italico: bool,
    pub cor: Color,
    pub escala_x: f32,
    pub escala_y: f32,
    pub ruido: Option<RuidoDriver>,
    pub anim: Option<AnimDriver>,
    pub trim_inicio: f32,
    pub trim_fim: f32,
}

impl TextoItem {
    pub fn pos_em(&self, t: f32) -> (f32, f32) {
        let (mut x, mut y) = (self.px, self.py);
        if let Some(a) = &self.anim {
            if a.alvo == 0 {
                let v = a.valor(t);
                x = a.comp.map_or(v[0], |c| if c == 0 { v[0] } else { x });
                y = a.comp.map_or(v[1], |c| if c == 1 { v[1] } else { y });
            }
        }
        if let Some(r) = self.ruido {
            if r.alvo == 0 {
                let (dx, dy) = ruido_offset(r.seed, r.freq, r.amp, r.veloc, t);
                x += r.comp.map_or(dx, |c| if c == 0 { dx } else { 0.0 });
                y += r.comp.map_or(dy, |c| if c == 1 { dy } else { 0.0 });
            }
        }
        (x, y)
    }

    pub fn escala_em(&self, t: f32) -> (f32, f32) {
        let (mut sx, mut sy) = (self.escala_x, self.escala_y);
        if let Some(a) = &self.anim {
            if a.alvo == 2 {
                let v = a.valor(t);
                sx = a.comp.map_or(v[0], |c| if c == 0 { v[0] } else { sx });
                sy = a.comp.map_or(v[1], |c| if c == 1 { v[1] } else { sy });
            }
        }
        (sx, sy)
    }

    pub fn opac_em(&self, t: f32) -> f32 {
        match &self.anim {
            Some(a) if a.alvo == 3 => a.valor(t)[0].clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    pub fn trim_em(&self, t: f32) -> f32 {
        let progress = (t / 6.0).clamp(0.0, 1.0);
        if progress < self.trim_inicio || progress > self.trim_fim {
            0.0
        } else {
            1.0
        }
    }
}

/// Geometria procedural gerada por um nó Pen.
#[derive(Debug, Clone)]
pub struct PenPath {
    pub program: Program,
    pub pos: Vec2,
    pub cor: Color,
    pub cor_fill: Color,
    pub espessura: f32,
    pub preenchimento: bool,
    pub seed: u32,
    pub cantos: f32,
    pub ordem: f32,
    pub escala_x: f32,
    pub escala_y: f32,
    pub ruido: Option<RuidoDriver>,
    pub anim: Option<AnimDriver>,
    pub erro_eval: Option<String>,
    pub trim_inicio: f32,
    pub trim_fim: f32,
    pub duracao: f32,
}

impl PenPath {
    pub fn pos_em(&self, t: f32) -> Vec2 {
        let mut p = self.pos;
        if let Some(a) = &self.anim {
            if a.alvo == 0 {
                let v = a.valor(t);
                p.x = a.comp.map_or(v[0], |c| if c == 0 { v[0] } else { p.x });
                p.y = a.comp.map_or(v[1], |c| if c == 1 { v[1] } else { p.y });
            }
        }
        if let Some(r) = self.ruido {
            if r.alvo == 0 {
                let (dx, dy) = ruido_offset(r.seed, r.freq, r.amp, r.veloc, t);
                p.x += r.comp.map_or(dx, |c| if c == 0 { dx } else { 0.0 });
                p.y += r.comp.map_or(dy, |c| if c == 1 { dy } else { 0.0 });
            }
        }
        p
    }

    pub fn opac_em(&self, t: f32) -> f32 {
        match &self.anim {
            Some(a) if a.alvo == 3 => a.valor(t)[0].clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    pub fn trim_em(&self, t: f32) -> f32 {
        let progress = (t / self.duracao.max(0.001)).clamp(0.0, 1.0);
        let trim_fim_anim = self.trim_inicio + (self.trim_fim - self.trim_inicio) * progress;
        if progress < self.trim_inicio || progress > trim_fim_anim {
            0.0
        } else {
            1.0
        }
    }
}

/// Conteúdo completo de uma camada dentro de uma cena.
#[derive(Debug, Clone, Default)]
pub struct LayerPreview {
    pub nome: String,
    pub opacidade: f32,
    pub formas: Vec<ShapeGenerator>,
    pub textos: Vec<TextoItem>,
    pub pen: Vec<PenPath>,
}

/// Conteúdo completo de uma cena para o preview.
#[derive(Debug, Clone, Default)]
pub struct CenaPreview {
    pub opacidade: f32,
    pub layers: Vec<LayerPreview>,
}

/// Tudo o que o preview precisa para desenhar um quadro.
#[derive(Debug, Clone, Default)]
pub struct PreviewData {
    pub largura: f32,
    pub altura: f32,
    pub fundo: Color,
    pub cenas: Vec<CenaPreview>,
}

/// Gerador procedural de uma forma.
#[derive(Debug, Clone)]
pub struct ShapeGenerator {
    pub kind: ShapeKind,
    pub pos: Vec2,
    pub tam: Vec2,
    pub rot: f32,
    pub cor: Color,
    pub seed: f32,
    pub noise_scale: f32,
    pub amp: f32,
    pub veloc: f32,
    pub ruido: Option<RuidoDriver>,
    pub anim: Option<AnimDriver>,
    pub trim_inicio: f32,
    pub trim_fim: f32,
    pub duracao: f32,
}

impl ShapeGenerator {
    pub fn opac_em(&self, t: f32) -> f32 {
        match &self.anim {
            Some(a) if a.alvo == 3 => a.valor(t)[0].clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    /// Gera a forma (em coordenadas de canvas do projeto) no instante `t`.
    pub fn generate(&self, t: f32) -> Shape {
        let mut center = self.pos;
        let mut tam = self.tam;
        let mut rot = self.rot;
        let mut cor = self.cor;

        if let Some(a) = &self.anim {
            let v = a.valor(t);
            let vx = v[0];
            let vy = v[1];
            match a.alvo {
                0 => match a.comp {
                    Some(0) => center.x = vx,
                    Some(1) => center.y = vy,
                    _ => center = Vec2::new(vx, vy),
                },
                1 => rot = vx,
                2 => match a.comp {
                    Some(0) => tam.x = vx,
                    Some(1) => tam.y = vy,
                    _ => tam = Vec2::new(vx, vy),
                },
                4 => {
                    let f = vx.clamp(0.0, 1.0);
                    cor = Color::from_rgba(
                        (cor.r() as f32 * f) as u8,
                        (cor.g() as f32 * f) as u8,
                        (cor.b() as f32 * f) as u8,
                        cor.a(),
                    );
                }
                _ => {}
            }
        }

        let ns = self.noise_scale.max(0.0001);

        if self.amp > 0.001 || self.veloc > 0.001 {
            let simplex = Simplex::new(self.seed as u32);
            let ax = self.seed as f64 * 137.13;
            let ay = self.seed as f64 * 311.17 + 50.0;
            let tt = (t * self.veloc.max(0.0)) as f64;
            let s = ns as f64;

            let nx = fbm(&simplex, [ax * s, ay * s, tt * 0.25]);
            let ny = fbm(&simplex, [ax * s + 100.0, ay * s + 100.0, tt * 0.25]);
            let off = Vec2::new(nx as f32, ny as f32) * 0.5 * self.amp;
            center += off;

            let amp01 = self.amp.max(0.0).min(1.0) as f64;
            let nsz = 1.0 + 0.15 * amp01 * fbm(&simplex, [ax * s + 200.0, ay * s, tt * 0.2]);
            tam *= (nsz as f32).max(0.2);

            rot += (20.0 * amp01 * fbm(&simplex, [ax * s + 300.0, ay * s, tt * 0.15])) as f32;
        }

        if let Some(r) = self.ruido {
            let (dx, dy) = ruido_offset(r.seed, r.freq, r.amp, r.veloc, t);
            let dx = r.comp.map_or(dx, |c| if c == 0 { dx } else { 0.0 });
            let dy = r.comp.map_or(dy, |c| if c == 1 { dy } else { 0.0 });
            match r.alvo {
                1 => rot += dx,
                2 => {
                    let f = 1.0 + dx / r.amp.max(1.0);
                    tam *= f.max(0.05);
                }
                4 => {
                    let f = (1.0 + dx / r.amp.max(1.0)).clamp(0.0, 2.0);
                    cor = Color::from_rgba(
                        (cor.r() as f32 * f).clamp(0.0, 255.0) as u8,
                        (cor.g() as f32 * f).clamp(0.0, 255.0) as u8,
                        (cor.b() as f32 * f).clamp(0.0, 255.0) as u8,
                        cor.a(),
                    );
                }
                _ => {
                    center.x += dx;
                    center.y += dy;
                }
            }
        }

        let c = Pos2::new(center.x, center.y);
        let mut shape = match self.kind {
            ShapeKind::Retangulo => {
                if rot.abs() < 0.01 {
                    Shape::Rect {
                        c,
                        tam,
                        corner_radius: (tam.x.min(tam.y) * 0.1).clamp(0.0, 255.0) as u8,
                        cor,
                    }
                } else {
                    let pts = crate::domain::retangulo_rot(c, tam, rot);
                    Shape::Path { pts, cor }
                }
            }
            ShapeKind::Elipse => {
                let rx = (tam.x * 0.5).max(1.0);
                let ry = (tam.y * 0.5).max(1.0);
                if rot.abs() < 0.01 {
                    Shape::Ellipse { c, rx, ry, cor }
                } else {
                    let pts = crate::domain::elipse_rot(c, rx, ry, rot);
                    Shape::Path { pts, cor }
                }
            }
            ShapeKind::Triangulo => {
                let r = (tam.x * 0.5).max(1.0);
                let pts = crate::domain::poligono_regular(c, r, 3, rot);
                Shape::Path { pts, cor }
            }
            ShapeKind::Losango => {
                let r = (tam.x * 0.5).max(1.0);
                let pts = crate::domain::poligono_regular(c, r, 4, rot);
                Shape::Path { pts, cor }
            }
            ShapeKind::Poligono => {
                let lados = (tam.y.clamp(2.0, 12.0)).round().max(3.0) as usize;
                let r = (tam.x * 0.5).max(1.0);
                let pts = crate::domain::poligono_regular(c, r, lados, rot);
                Shape::Path { pts, cor }
            }
            ShapeKind::Estrela => {
                let r = (tam.x * 0.5).max(1.0);
                let pts = crate::domain::estrela(c, r, r * 0.45, 5, rot);
                Shape::Path { pts, cor }
            }
            ShapeKind::Seta => seta(c, tam, rot, cor),
        };

        let use_trim = self.trim_inicio > 0.0 || self.trim_fim < 1.0;
        let trim_inicio = self.trim_inicio;
        let trim_fim = if use_trim {
            let progress = (t / self.duracao.max(0.001)).clamp(0.0, 1.0);
            self.trim_inicio + (self.trim_fim - self.trim_inicio) * progress
        } else {
            self.trim_fim
        };
        if trim_inicio > 0.0 || trim_fim < 1.0 {
            let pts = match &shape {
                Shape::Rect { c, tam, .. } => {
                    let hw = tam.x * 0.5;
                    let hh = tam.y * 0.5;
                    Some(vec![
                        Pos2::new(c.x - hw, c.y - hh),
                        Pos2::new(c.x + hw, c.y - hh),
                        Pos2::new(c.x + hw, c.y + hh),
                        Pos2::new(c.x - hw, c.y + hh),
                    ])
                }
                Shape::Ellipse { c, rx, ry, .. } => {
                    let n = 48;
                    Some(
                        (0..n)
                            .map(|i| {
                                let a = i as f32 * std::f32::consts::TAU / n as f32;
                                Pos2::new(c.x + rx * a.cos(), c.y + ry * a.sin())
                            })
                            .collect(),
                    )
                }
                Shape::Path { pts, .. } => Some(pts.clone()),
            };
            if let Some(mut pts) = pts {
                pts = trim_path_pts(&pts, true, trim_inicio, trim_fim);
                shape = Shape::Path { pts, cor };
            }
        }

        shape
    }
}

/// Forma gerada — descrição pura, sem dependência de renderer.
#[derive(Debug, Clone)]
pub enum Shape {
    Rect {
        c: Pos2,
        tam: Vec2,
        corner_radius: u8,
        cor: Color,
    },
    Ellipse {
        c: Pos2,
        rx: f32,
        ry: f32,
        cor: Color,
    },
    Path {
        pts: Vec<Pos2>,
        cor: Color,
    },
}

/// Fractal Brownian Motion (FBM).
pub fn fbm(s: &Simplex, p: [f64; 3]) -> f64 {
    let mut freq = 1.0;
    let mut amp = 1.0;
    let mut soma = 0.0;
    let mut norm = 0.0;
    for _ in 0..4 {
        soma += amp * s.get([p[0] * freq, p[1] * freq, p[2] * freq]);
        norm += amp;
        freq *= 2.0;
        amp *= 0.5;
    }
    if norm > 0.0 {
        soma / norm
    } else {
        soma
    }
}

/// Deslocamento 2D (dx, dy) de um nó Ruído no instante `t`.
pub fn ruido_offset(seed: f32, freq: f32, amp: f32, veloc: f32, t: f32) -> (f32, f32) {
    let s = Simplex::new(seed as u32);
    let f = freq.max(0.0001) as f64;
    let ax = seed as f64 * 137.13;
    let ay = seed as f64 * 311.17 + 50.0;
    let tt = (t * veloc) as f64;
    let dx = fbm(&s, [ax * f, ay * f, tt * 0.25]) as f32 * amp;
    let dy = fbm(&s, [ax * f + 100.0, ay * f + 100.0, tt * 0.25]) as f32 * amp;
    (dx, dy)
}

/// Seta apontando para a direita, centrada em `c`.
fn seta(c: Pos2, tam: Vec2, rot: f32, cor: Color) -> Shape {
    let w = tam.x.max(2.0);
    let h = tam.y.max(2.0);
    let esp = (h * 0.35).max(2.0);
    let comp = w * 0.45;
    let rad = rot.to_radians();
    let rot2 = |x: f32, y: f32| {
        Pos2::new(
            c.x + x * rad.cos() - y * rad.sin(),
            c.y + x * rad.sin() + y * rad.cos(),
        )
    };
    let pts = vec![
        rot2(-w / 2.0, -esp / 2.0),
        rot2(-w / 2.0 + comp, -esp / 2.0),
        rot2(-w / 2.0 + comp, -h / 2.0),
        rot2(w / 2.0, 0.0),
        rot2(-w / 2.0 + comp, h / 2.0),
        rot2(-w / 2.0 + comp, esp / 2.0),
        rot2(-w / 2.0, esp / 2.0),
    ];
    Shape::Path { pts, cor }
}

/// Recorta uma polyline entre `inicio` e `fim` (normalizados [0,1]). `closed` indica se o último ponto conecta de volta ao primeiro.
pub fn trim_path_pts(pts: &[Pos2], closed: bool, inicio: f32, fim: f32) -> Vec<Pos2> {
    if pts.is_empty() || pts.len() < 2 {
        return pts.to_vec();
    }
    if inicio <= 0.0 && fim >= 1.0 {
        return pts.to_vec();
    }
    if fim <= inicio {
        return vec![];
    }

    let n = pts.len();
    let segs = if closed { n } else { n - 1 };
    if segs == 0 {
        return pts.to_vec();
    }
    let total = segs as f32;
    let s = (inicio * total).clamp(0.0, total);
    let e = (fim * total).clamp(0.0, total);
    if e - s < 1e-6 {
        return vec![];
    }

    let lerp = |a: Pos2, b: Pos2, t: f32| Pos2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t);
    let nxt = |i: usize| {
        if closed && i + 1 >= n {
            0
        } else {
            (i + 1).min(n - 1)
        }
    };

    let si = (s.floor() as usize).min(segs - 1);
    let ei = (e.floor() as usize).min(segs - 1);
    let sf = s - si as f32;
    let ef = e - ei as f32;

    let mut result = Vec::new();
    result.push(if sf > 0.0 {
        lerp(pts[si], pts[nxt(si)], sf)
    } else {
        pts[si]
    });

    if si < ei {
        for i in (si + 1)..=ei {
            result.push(pts[i]);
        }
    } else if si > ei && closed {
        for i in (si + 1)..n {
            result.push(pts[i]);
        }
        for i in 0..=ei {
            result.push(pts[i]);
        }
    }

    let end_p = if ef > 0.0 {
        lerp(pts[ei], pts[nxt(ei)], ef)
    } else {
        pts[ei]
    };
    let last = *result.last().unwrap();
    if end_p.distance(last) > 0.5 {
        result.push(end_p);
    } else {
        *result.last_mut().unwrap() = end_p;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Easing;

    fn seg(t0: f32, t1: f32, a: [f32; 2], b: [f32; 2]) -> AnimSeg {
        AnimSeg {
            t_ini: t0,
            t_fim: t1,
            v_ini: a,
            v_fim: b,
            easing: Easing::Linear,
        }
    }

    #[test]
    fn trim_sem_alteracao() {
        let pts = vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 10.0),
        ];
        let r = trim_path_pts(&pts, false, 0.0, 1.0);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn trim_metade() {
        let pts = vec![Pos2::new(0.0, 0.0), Pos2::new(10.0, 0.0)];
        let r = trim_path_pts(&pts, false, 0.0, 0.5);
        assert_eq!(r.len(), 2);
        assert!((r[1].x - 5.0).abs() < 0.1);
    }

    #[test]
    fn trim_closed_wrap() {
        let pts = vec![
            Pos2::new(0.0, 0.0),
            Pos2::new(10.0, 0.0),
            Pos2::new(10.0, 10.0),
            Pos2::new(0.0, 10.0),
        ];
        let r = trim_path_pts(&pts, true, 0.75, 1.0);
        assert!(r.len() >= 2);
        assert!((r[0].x - 0.0).abs() < 0.1);
        assert!((r[0].y - 10.0).abs() < 0.1);
    }

    #[test]
    fn anim_interpola_linear_no_meio() {
        let d = AnimDriver {
            segmentos: vec![seg(0.0, 2.0, [0.0, 0.0], [10.0, 20.0])],
            loop_mode: LoopMode::Nenhum,
            alvo: 0,
            comp: None,
        };
        let v = d.valor(1.0);
        assert!((v[0] - 5.0).abs() < 1e-4);
        assert!((v[1] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn anim_segura_antes_e_depois_sem_loop() {
        let d = AnimDriver {
            segmentos: vec![seg(1.0, 2.0, [3.0, 0.0], [7.0, 0.0])],
            loop_mode: LoopMode::Nenhum,
            alvo: 0,
            comp: None,
        };
        assert!((d.valor(0.0)[0] - 3.0).abs() < 1e-4);
        assert!((d.valor(5.0)[0] - 7.0).abs() < 1e-4);
    }

    #[test]
    fn anim_loop_repetir_volta_ao_inicio() {
        let d = AnimDriver {
            segmentos: vec![seg(0.0, 2.0, [0.0, 0.0], [10.0, 0.0])],
            loop_mode: LoopMode::Repetir,
            alvo: 0,
            comp: None,
        };
        assert!((d.valor(3.0)[0] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn anim_pingpong_reflete() {
        let d = AnimDriver {
            segmentos: vec![seg(0.0, 2.0, [0.0, 0.0], [10.0, 0.0])],
            loop_mode: LoopMode::PingPong,
            alvo: 0,
            comp: None,
        };
        assert!((d.valor(3.0)[0] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn easing_step_salta_no_fim() {
        assert_eq!(Easing::Step.aplicar(0.99), 0.0);
        assert_eq!(Easing::Step.aplicar(1.0), 1.0);
    }
}
