use eframe::egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2};
use eframe::egui::epaint::{EllipseShape, PathShape, RectShape};
pub use glam::Vec2 as GVec2;
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

/// Modulação por um nó Ruído conectado a um parâmetro deste item. Aplicada
/// no instante `t` do render (para animar), somando um deslocamento FBM ao
/// parâmetro `alvo` (0=Posição, 1=Rotação, 2=Escala, 4=Cor).
/// `comp` indica qual componente do ruído usar: None=ambos, Some(0)=X, Some(1)=Y.
#[derive(Debug, Clone, Copy)]
pub struct RuidoDriver {
    pub seed: f32,
    pub freq: f32,
    pub amp: f32,
    pub veloc: f32,
    pub alvo: u8,
    pub comp: Option<usize>,
}

/// Curva de easing de um segmento de animação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Step,
}

impl Easing {
    pub fn from_u8(v: u8) -> Easing {
        match v {
            1 => Easing::EaseIn,
            2 => Easing::EaseOut,
            3 => Easing::EaseInOut,
            4 => Easing::Step,
            _ => Easing::Linear,
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            Easing::Linear => 0,
            Easing::EaseIn => 1,
            Easing::EaseOut => 2,
            Easing::EaseInOut => 3,
            Easing::Step => 4,
        }
    }
    /// Aplica a curva a `x` em [0,1] → retorna o fator interpolado em [0,1].
    pub fn aplicar(self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Easing::Linear => x,
            Easing::EaseIn => x * x,
            Easing::EaseOut => 1.0 - (1.0 - x) * (1.0 - x),
            Easing::EaseInOut => {
                if x < 0.5 {
                    2.0 * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
                }
            }
            Easing::Step => {
                if x >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// Modo de repetição da animação quando `t` passa do último segmento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    /// Segura o último valor (padrão).
    Nenhum,
    /// Reinicia do começo (t módulo duração total).
    Repetir,
    /// Vai e volta (ping-pong).
    PingPong,
}

impl LoopMode {
    pub fn from_u8(v: u8) -> LoopMode {
        match v {
            1 => LoopMode::Repetir,
            2 => LoopMode::PingPong,
            _ => LoopMode::Nenhum,
        }
    }
}

/// Um trecho da função de animação: de `t_ini` a `t_fim` (segundos), o valor
/// (vetorial X/Y) vai de `v_ini` a `v_fim` com a curva `easing`. Para alvos
/// escalares (Rotação/Opacidade) usa-se apenas o componente X.
#[derive(Debug, Clone, Copy)]
pub struct AnimSeg {
    pub t_ini: f32,
    pub t_fim: f32,
    pub v_ini: [f32; 2],
    pub v_fim: [f32; 2],
    pub easing: Easing,
}

/// Driver de animação conectado a um parâmetro de um item. Avaliado no
/// instante `t` como uma função por partes que SUBSTITUI o valor do alvo.
/// `alvo`: 0=Posição, 1=Rotação, 2=Escala, 3=Opacidade, 4=Cor.
/// `comp` indica qual componente usar: None=ambos, Some(0)=X, Some(1)=Y.
#[derive(Debug, Clone)]
pub struct AnimDriver {
    pub segmentos: Vec<AnimSeg>,
    pub loop_mode: LoopMode,
    pub alvo: u8,
    pub comp: Option<usize>,
}

impl AnimDriver {
    /// Duração total (t_fim do último segmento). 0 se vazio.
    fn duracao(&self) -> f32 {
        self.segmentos
            .iter()
            .map(|s| s.t_fim)
            .fold(0.0, f32::max)
    }

    /// Avalia a animação no instante `t`, retornando o valor vetorial (X/Y).
    /// Antes do 1º segmento segura o valor inicial; depois do último, segue o
    /// `loop_mode`.
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
                if m <= dur { m } else { 2.0 * dur - m }
            }
            _ => t,
        };
        // Antes do início: valor inicial do primeiro segmento.
        let primeiro = &self.segmentos[0];
        if tt <= primeiro.t_ini {
            return primeiro.v_ini;
        }
        // Encontra o segmento que contém `tt`.
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
        // Depois do último: segura o valor final.
        self.segmentos.last().map(|s| s.v_fim).unwrap_or([0.0, 0.0])
    }
}

/// Item de texto a desenhar no preview, em coordenadas de projeto.
/// Rasterizado via `cosmic-text` (ver `ui/text_raster.rs`).
#[derive(Debug, Clone)]
pub struct TextoItem {
    pub px: f32,
    pub py: f32,
    pub conteudo: String,
    pub tamanho: f32,
    pub negrito: bool,
    pub italico: bool,
    pub cor: Color32,
    /// Escala base do texto (1.0 = tamanho original).
    pub escala_x: f32,
    pub escala_y: f32,
    /// Ruído conectado (modula a Posição do texto), se houver.
    pub ruido: Option<RuidoDriver>,
    /// Animação conectada (substitui Posição/Opacidade/Escala), se houver.
    pub anim: Option<AnimDriver>,
    pub trim_inicio: f32,
    pub trim_fim: f32,
}

impl TextoItem {
    /// Posição do texto no instante `t`: a Animação (alvo=Posição) SUBSTITUI a
    /// base; o Ruído (alvo=Posição) SOMA por cima.
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

    /// Escala do texto no instante `t`: a Animação (alvo=Escala) SUBSTITUI a
    /// base; senão usa a escala base do nó.
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

    /// Opacidade extra (0..1) do texto no instante `t` vinda da Animação
    /// (alvo=Opacidade). 1.0 se não houver animação de opacidade.
    pub fn opac_em(&self, t: f32) -> f32 {
        match &self.anim {
            Some(a) if a.alvo == 3 => a.valor(t)[0].clamp(0.0, 1.0),
            _ => 1.0,
        }
    }
}

/// Geometria procedural gerada por um nó Pen: o programa DSL já parseado
/// (cache), o deslocamento da "caneta" na cena e o estilo padrão. O preview
/// avalia o `program` por frame com o tempo `t`.
#[derive(Debug, Clone)]
pub struct PenPath {
    pub program: crate::dsl::Program,
    pub pos: GVec2,
    /// Cor do traço (contorno).
    pub cor: Color32,
    /// Cor do preenchimento.
    pub cor_fill: Color32,
    pub espessura: f32,
    pub preenchimento: bool,
    pub seed: u32,
    pub cantos: f32,
    pub ordem: f32,
    pub escala_x: f32,
    pub escala_y: f32,
    /// Ruído conectado (modula a Posição da caneta), se houver.
    pub ruido: Option<RuidoDriver>,
    /// Animação conectada (substitui Posição / Opacidade), se houver.
    pub anim: Option<AnimDriver>,
    /// Erro de eval em tempo real (preenchido pelo preview em cada frame).
    pub erro_eval: Option<String>,
    pub trim_inicio: f32,
    pub trim_fim: f32,
    /// Duração total da animação em segundos (usada para animar o trim).
    pub duracao: f32,
}

impl PenPath {
    /// Posição da caneta no instante `t`: a Animação (alvo=Posição) SUBSTITUI
    /// a base; o Ruído (alvo=Posição) SOMA por cima.
    pub fn pos_em(&self, t: f32) -> GVec2 {
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

    /// Opacidade (0..1) da caneta no instante `t` vinda da Animação
    /// (alvo=Opacidade). 1.0 se não houver.
    pub fn opac_em(&self, t: f32) -> f32 {
        match &self.anim {
            Some(a) if a.alvo == 3 => a.valor(t)[0].clamp(0.0, 1.0),
            _ => 1.0,
        }
    }
}

/// Conteúdo completo de uma camada dentro de uma cena: nome, opacidade,
/// formas, textos e caneta, em ordem de desenho.
#[derive(Debug, Clone, Default)]
pub struct LayerPreview {
    pub nome: String,
    pub opacidade: f32,
    pub formas: Vec<ShapeGenerator>,
    pub textos: Vec<TextoItem>,
    pub pen: Vec<PenPath>,
}

/// Conteúdo completo de uma cena para o preview: camadas (cada uma com
/// suas formas e textos), em coordenadas de projeto, na ordem de desenho.
#[derive(Debug, Clone, Default)]
pub struct CenaPreview {
    pub opacidade: f32,
    pub layers: Vec<LayerPreview>,
}

/// Tudo o que o preview precisa para desenhar um quadro: dimensões do projeto,
/// fundo e a lista ordenada de cenas (cada uma com suas camadas).
#[derive(Debug, Clone, Default)]
pub struct PreviewData {
    pub largura: f32,
    pub altura: f32,
    pub fundo: Color32,
    pub cenas: Vec<CenaPreview>,
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

/// Gerador procedural de uma forma. A partir dos parâmetros base (posição,
/// tamanho, rotação, cor) e dos parâmetros de ruído (seed, escala, amplitude,
/// velocidade), produz a forma animada num instante `t` (segundos).
#[derive(Debug, Clone)]
pub struct ShapeGenerator {
    pub kind: ShapeKind,
    pub pos: GVec2,
    pub tam: GVec2,
    pub rot: f32,
    pub cor: Color32,
    pub seed: f32,
    pub noise_scale: f32,
    pub amp: f32,
    pub veloc: f32,
    /// Ruído externo conectado (modula Posição/Rotação/Escala), se houver.
    pub ruido: Option<RuidoDriver>,
    /// Animação conectada (substitui Posição/Rotação/Escala/Opacidade).
    pub anim: Option<AnimDriver>,
    /// Trim (0..1) do percurso da forma.
    pub trim_inicio: f32,
    pub trim_fim: f32,
    /// Duração total da animação em segundos (usada para animar o trim).
    pub duracao: f32,
}

impl ShapeGenerator {
    /// Opacidade (0..1) da forma no instante `t` vinda da Animação
    /// (alvo=Opacidade). 1.0 se não houver.
    pub fn opac_em(&self, t: f32) -> f32 {
        match &self.anim {
            Some(a) if a.alvo == 3 => a.valor(t)[0].clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    /// Gera a forma (em coordenadas de canvas do projeto) no instante `t`.
    ///
    /// A BASE (posição, tamanho, rotação, cor) é SEMPRE estável e respeita
    /// exatamente os parâmetros do nó — sem ruído. O ruído só é aplicado como
    /// uma perturbação SUAVE e opcional, controlada por `amp`/`veloc`, e é
    /// ancorado num ponto FIXO (derivado do `seed`) para que arrastar os valores
    /// do nó não faça a forma "saltar"/tremeluzir: a entrada do ruído não muda
    /// com a posição do nó.
    pub fn generate(&self, t: f32) -> Shape {
        // centro e tamanho base (estáveis)
        let mut center = self.pos;
        let mut tam = self.tam;
        let mut rot = self.rot;
        let mut cor = self.cor;

        // Animação EXTERNA (nó Animação conectado): SUBSTITUI o parâmetro alvo
        // com o valor da função no tempo `t`. Aplicada ANTES do ruído, para
        // que o ruído/FBM some por cima como perturbação.
        if let Some(a) = &self.anim {
            let v = a.valor(t);
            let vx = v[0];
            let vy = v[1];
            match a.alvo {
                0 => match a.comp {
                    Some(0) => center.x = vx,
                    Some(1) => center.y = vy,
                    _ => center = GVec2::new(vx, vy),
                }
                1 => rot = vx,
                2 => match a.comp {
                    Some(0) => tam.x = vx,
                    Some(1) => tam.y = vy,
                    _ => tam = GVec2::new(vx, vy),
                }
                4 => {
                    let f = vx.clamp(0.0, 1.0);
                    cor = Color32::from_rgba_premultiplied(
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

        // Só perturba se o usuário quiser animação (amp ou veloc > 0).
        // `amp` controla a INTENSIDADE (não a frequência) do deslocamento,
        // para que o movimento seja sempre fluido e estável, nunca jitter.
        if self.amp > 0.001 || self.veloc > 0.001 {
            let simplex = Simplex::new(self.seed as u32);
            // ponto fixo ancorado no seed: não depende de `pos`, então arrastar
            // o nó não causa salto no ruído.
            let ax = self.seed as f64 * 137.13;
            let ay = self.seed as f64 * 311.17 + 50.0;
            let tt = (t * self.veloc.max(0.0)) as f64;
            let s = ns as f64;

            // FBM (Fractal Brownian Motion): soma de várias oitavas de simplex
            // para um movimento mais orgânico e suave que uma oitava só.
            // Cada eixo usa um canal de ruído diferente (offsets distintos),
            // dando um caminho 2D natural em vez de diagonal.
            let nx = fbm(&simplex, [ax * s, ay * s, tt * 0.25]);
            let ny = fbm(&simplex, [ax * s + 100.0, ay * s + 100.0, tt * 0.25]);
            // já em [-1,1] → escala pela amplitude (metade p/ cada lado).
            let off = GVec2::new(nx as f32, ny as f32) * 0.5 * self.amp;
            center += off;

            let amp01 = self.amp.max(0.0).min(1.0) as f64;
            // tamanho "respira" de forma suave (varia no máximo ~15% da amp).
            let nsz = 1.0 + 0.15 * amp01 * fbm(&simplex, [ax * s + 200.0, ay * s, tt * 0.2]);
            tam *= (nsz as f32).max(0.2);

            // rotação dinâmica suave (também proporcional à amplitude).
            rot += (20.0 * amp01 * fbm(&simplex, [ax * s + 300.0, ay * s, tt * 0.15])) as f32;
        }

        // Ruído EXTERNO (nó Ruído conectado): modula o parâmetro escolhido.
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
                    cor = Color32::from_rgba_premultiplied(
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
                // Sem rotação: RectShape (permite corner radius arredondado).
                // Com rotação: vira um path de 4 vértices girados, pois o
                // RectShape do egui não suporta rotação.
                if rot.abs() < 0.01 {
                    Shape::Rect(RectShape::filled(
                        Rect::from_center_size(c, Vec2::new(tam.x, tam.y)),
                        // corner radius PROPORCIONAL ao tamanho (10%), para
                        // acompanhar zoom/escala de forma consistente.
                        (tam.x.min(tam.y) * 0.1).clamp(0.0, 255.0) as u8,
                        cor,
                    ))
                } else {
                    let pts = retangulo_rot(c, tam, rot);
                    Shape::Path(PathShape::convex_polygon(
                        pts,
                        cor,
                        eframe::egui::Stroke::NONE,
                    ))
                }
            }
            ShapeKind::Elipse => {
                let rx = (tam.x * 0.5).max(1.0);
                let ry = (tam.y * 0.5).max(1.0);
                if rot.abs() < 0.01 {
                    Shape::Ellipse(EllipseShape::filled(c, Vec2::new(rx, ry), cor))
                } else {
                    let pts = elipse_rot(c, rx, ry, rot);
                    let mut p = PathShape::closed_line(pts, eframe::egui::Stroke::NONE);
                    p.fill = cor;
                    Shape::Path(p)
                }
            }
            ShapeKind::Triangulo => {
                let r = (tam.x * 0.5).max(1.0);
                let pts = poligono_regular(c, r, 3, rot);
                Shape::Path(PathShape::convex_polygon(pts, cor, eframe::egui::Stroke::NONE))
            }
            ShapeKind::Losango => {
                let r = (tam.x * 0.5).max(1.0);
                let pts = poligono_regular(c, r, 4, rot);
                Shape::Path(PathShape::convex_polygon(pts, cor, eframe::egui::Stroke::NONE))
            }
            ShapeKind::Poligono => {
                // lados derivados da "altura" (2..12), para reaproveitar os
                // parâmetros do nó sem campo novo.
                let lados = (tam.y.clamp(2.0, 12.0)).round().max(3.0) as usize;
                let r = (tam.x * 0.5).max(1.0);
                let pts = poligono_regular(c, r, lados, rot);
                Shape::Path(PathShape::convex_polygon(pts, cor, eframe::egui::Stroke::NONE))
            }
            ShapeKind::Estrela => {
                let r = (tam.x * 0.5).max(1.0);
                let pts = estrela(c, r, r * 0.45, 5, rot);
                // estrela NÃO é convexa: usa poliline fechada com fill (e não
                // convex_polygon, que distorceria os vértices internos).
                let mut p = PathShape::closed_line(pts, eframe::egui::Stroke::NONE);
                p.fill = cor;
                Shape::Path(p)
            }
            ShapeKind::Seta => seta(c, tam, rot, cor),
        };

        // Aplica trim se ativo: converte a forma preenchida num contorno
        // recortado (stroke) com a mesma cor.
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
                Shape::Rect(r) => {
                    let rect = r.rect;
                    Some(vec![
                        Pos2::new(rect.left(), rect.top()),
                        Pos2::new(rect.right(), rect.top()),
                        Pos2::new(rect.right(), rect.bottom()),
                        Pos2::new(rect.left(), rect.bottom()),
                    ])
                }
                Shape::Ellipse(e) => {
                    let c = e.center;
                    let rx = e.radius.x;
                    let ry = e.radius.y;
                    let n = 48;
                    Some((0..n).map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / n as f32;
                        Pos2::new(c.x + rx * a.cos(), c.y + ry * a.sin())
                    }).collect())
                }
                Shape::Path(p) => Some(p.points.clone()),
                _ => None,
            };
            if let Some(mut pts) = pts {
                let closed = true;
                pts = trim_path_pts(&pts, closed, trim_inicio, trim_fim);
                let mut p = PathShape::line(pts, Stroke::new(3.0, cor));
                p.fill = Color32::TRANSPARENT;
                shape = Shape::Path(p);
            }
        }

        shape
    }
}

/// Fractal Brownian Motion (FBM): soma de 4 oitavas de simplex com frequência
/// dobrando e amplitude caindo pela metade a cada oitava. Retorna valor
/// aproximadamente em [-1, 1] (normalizado pela soma das amplitudes), dando um
/// ruído mais rico/orgânico que uma única oitava.
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

/// Deslocamento 2D (dx, dy) de um nó Ruído no instante `t`. Usa FBM ancorado
/// no `seed` (independe da posição), com `freq` controlando a frequência
/// espacial, `amp` a intensidade e `veloc` a velocidade temporal. Cada eixo
/// usa um canal de ruído distinto para um caminho natural.
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

/// Vértices de um retângulo `tam` centrado em `c`, girado por `rot` (graus).
fn retangulo_rot(c: Pos2, tam: GVec2, rot: f32) -> Vec<Pos2> {
    let rad = rot.to_radians();
    let (co, si) = (rad.cos(), rad.sin());
    let hw = tam.x * 0.5;
    let hh = tam.y * 0.5;
    [(-hw, -hh), (hw, -hh), (hw, hh), (-hw, hh)]
        .into_iter()
        .map(|(x, y)| Pos2::new(c.x + x * co - y * si, c.y + x * si + y * co))
        .collect()
}

/// Vértices de uma elipse (raios `rx`/`ry`) centrada em `c`, girada por `rot`
/// (graus), amostrada em 48 segmentos para um contorno liso.
fn elipse_rot(c: Pos2, rx: f32, ry: f32, rot: f32) -> Vec<Pos2> {
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

/// Vértices de um polígono regular de `n` lados centrado em `c`, raio `r`,
/// com rotação `rot` (graus). O primeiro vértice fica no topo.
fn poligono_regular(c: Pos2, r: f32, n: usize, rot: f32) -> Vec<Pos2> {
    let rad = rot.to_radians();
    (0..n)
        .map(|i| {
            let a = rad + i as f32 * std::f32::consts::TAU / n as f32 - std::f32::consts::FRAC_PI_2;
            Pos2::new(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

/// Vértices de uma estrela de `pontas` pontas centrada em `c`, raio externo
/// `ro`, raio interno `ri`, com rotação `rot` (graus).
fn estrela(c: Pos2, ro: f32, ri: f32, pontas: usize, rot: f32) -> Vec<Pos2> {
    let rad = rot.to_radians();
    let total = pontas * 2;
    (0..total)
        .map(|i| {
            let r = if i % 2 == 0 { ro } else { ri };
            let a = rad + i as f32 * std::f32::consts::TAU / total as f32
                - std::f32::consts::FRAC_PI_2;
            Pos2::new(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

/// Seta apontando para a direita, centrada em `c`, com caule de espessura
/// proporcional a `tam.y` e ponta triangular. Rotacionada por `rot` (graus).
fn seta(c: Pos2, tam: GVec2, rot: f32, cor: Color32) -> Shape {
    let w = tam.x.max(2.0);
    let h = tam.y.max(2.0);
    let esp = (h * 0.35).max(2.0); // espessura do caule
    let comp = w * 0.45; // comprimento do caule (resto é a ponta)
    let rad = rot.to_radians();
    let rot2 = |x: f32, y: f32| -> Pos2 {
        Pos2::new(
            c.x + x * rad.cos() - y * rad.sin(),
            c.y + x * rad.sin() + y * rad.cos(),
        )
    };
    let pontos = vec![
        rot2(-w / 2.0, -esp / 2.0),
        rot2(-w / 2.0 + comp, -esp / 2.0),
        rot2(-w / 2.0 + comp, -h / 2.0),
        rot2(w / 2.0, 0.0),
        rot2(-w / 2.0 + comp, h / 2.0),
        rot2(-w / 2.0 + comp, esp / 2.0),
        rot2(-w / 2.0, esp / 2.0),
    ];
    let mut p = PathShape::closed_line(pontos, eframe::egui::Stroke::NONE);
    p.fill = cor;
    Shape::Path(p)
}

/// Recorta uma polyline aberta mantendo apenas o trecho entre `inicio` e
/// `fim` (normalizados em [0, 1]). `closed` indica se o último ponto conecta
/// de volta ao primeiro (incluindo o segmento de fecho).
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

    let lerp = |a: Pos2, b: Pos2, t: f32| Pos2::new(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
    );
    let nxt = |i: usize| {
        if closed && i + 1 >= n { 0 } else { (i + 1).min(n - 1) }
    };

    let si = (s.floor() as usize).min(segs - 1);
    let ei = (e.floor() as usize).min(segs - 1);
    let sf = s - si as f32;
    let ef = e - ei as f32;

    let mut result = Vec::new();
    result.push(if sf > 0.0 { lerp(pts[si], pts[nxt(si)], sf) } else { pts[si] });

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

    let end_p = if ef > 0.0 { lerp(pts[ei], pts[nxt(ei)], ef) } else { pts[ei] };
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
        let pts = vec![Pos2::new(0.0, 0.0), Pos2::new(10.0, 0.0), Pos2::new(10.0, 10.0)];
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
        // 0.75 → 1.0 wraps around: 3/4 of the way to 4/4
        let r = trim_path_pts(&pts, true, 0.75, 1.0);
        assert!(r.len() >= 2);
        assert!((r[0].x - 0.0).abs() < 0.1);  // 75% = 3/4 of way from pt[2]→pt[3] = (0,10)
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
