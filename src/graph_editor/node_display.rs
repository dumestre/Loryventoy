use eframe::egui::{
    Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Shape, Stroke,
    StrokeKind, Vec2,
};
use eframe::egui::epaint::{CircleShape, RectShape, TextShape};
use egui_graphs::{DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node, NodeProps};
use petgraph::stable_graph::DefaultIx;
use petgraph::Directed;

use crate::nodes::{portos, TipoNo};
use crate::ui::node_component;

/// Peso da aresta: índices dos portos de origem (saída) e destino (entrada)
/// envolvidos na conexão, mais o componente escolhido de cada lado (Some(k)
/// para portos vetoriais, None para escalares / sem divisão). Permite desenhar
/// o fio entre portos específicos em vez do centro do nó.
#[derive(Clone, Copy, Debug, Default)]
pub struct ArestaInfo {
    pub saida: usize,
    /// Componente escolhido do porto de saída (Some(k) p/ vetorial). Lido
    /// pela avaliação dos nós (ainda não implementada).
    #[allow(dead_code)]
    pub saida_comp: Option<usize>,
    pub entrada: usize,
    /// Componente escolhido do porto de entrada (Some(k) p/ vetorial).
    #[allow(dead_code)]
    pub entrada_comp: Option<usize>,
}

/// Offsets (em coordenadas de canvas, relativos ao centro do nó) dos portos
/// de entrada (esquerda) e saída (direita), distribuídos ao longo do corpo.
pub fn port_offsets(tipo: TipoNo, half: Vec2) -> (Vec<Vec2>, Vec<Vec2>) {
    let spec = portos(tipo);
    let n = spec.entradas.len().max(spec.saidas.len()).max(1);
    let top = -half.y + node_component::CABECALHO_H + node_component::MARGEM_Y;
    let bottom = half.y - node_component::MARGEM_Y;
    let span = (bottom - top).max(0.0);
    // Fallback: distribuição uniforme, caso a row do parâmetro ainda não
    // tenha sido medida (ex.: primeiro frame).
    let y_uniforme = |i: usize| -> f32 {
        if n <= 1 {
            0.0
        } else {
            top + (i as f32 + 0.5) * span / (n as f32)
        }
    };
    // Y do porto de um parâmetro: usa a posição REAL da row (medida ao
    // desenhar o inspector), alinhando a bolinha exatamente à sua linha.
    // `linha_y` é relativo ao topo do corpo → soma o offset do topo.
    let y = |i: usize, nome: &str| -> f32 {
        match node_component::linha_y(tipo, nome) {
            Some(ry) => top + ry.clamp(0.0, span),
            None => y_uniforme(i),
        }
    };
    let ins: Vec<Vec2> = spec
        .entradas
        .iter()
        .enumerate()
        .map(|(i, p)| Vec2::new(-half.x, y(i, p.nome)))
        .collect();
    let outs: Vec<Vec2> = spec
        .saidas
        .iter()
        .enumerate()
        .map(|(i, p)| Vec2::new(half.x, y(i, p.nome)))
        .collect();
    (ins, outs)
}

const NODE_RADIUS: u8 = 8;

/// Exibição customizada dos nós: card com header colorido (nome),
/// corpo escuro, sombra e portos de conexão (entrada à esquerda,
/// saída à direita) na cor do tipo de nó.
#[derive(Clone)]
pub struct NoDisplay {
    pos: Pos2,
    label: String,
    color: Option<Color32>,
    selected: bool,
    dragged: bool,
    hovered: bool,
    half: Vec2,
}

impl NoDisplay {
    pub fn tamanho(label: &str) -> Vec2 {
        // Tamanho definido pelo componente reutilizável de nó.
        let tipo = TipoNo::from_label(label).unwrap_or(TipoNo::Transform);
        node_component::content_size(tipo)
    }

    /// Tipo do nó a partir do rótulo.
    fn tipo(&self) -> TipoNo {
        TipoNo::from_label(&self.label).unwrap_or(TipoNo::Transform)
    }

    /// Posição do porto de entrada `i` (esquerda) em coordenadas de canvas.
    pub fn port_in_pos(&self, i: usize) -> Pos2 {
        let (ins, _) = port_offsets(self.tipo(), self.half);
        self.pos
            + ins
                .get(i)
                .copied()
                .unwrap_or_else(|| Vec2::new(-self.half.x, 0.0))
    }

    /// Posição do porto de saída `i` (direita) em coordenadas de canvas.
    pub fn port_out_pos(&self, i: usize) -> Pos2 {
        let (_, outs) = port_offsets(self.tipo(), self.half);
        self.pos
            + outs
                .get(i)
                .copied()
                .unwrap_or_else(|| Vec2::new(self.half.x, 0.0))
    }
}

impl From<NodeProps<()>> for NoDisplay {
    fn from(s: NodeProps<()>) -> Self {
        let half = NoDisplay::tamanho(&s.label);
        NoDisplay {
            pos: s.location(),
            label: s.label.clone(),
            color: s.color(),
            selected: s.selected,
            dragged: s.dragged,
            hovered: s.hovered,
            half,
        }
    }
}

impl DisplayNode<(), ArestaInfo, Directed, DefaultIx> for NoDisplay {
    /// Ponto de contorno usado pelo `egui_graphs` para orientar a aresta.
    /// Retorna o porto de saída (dir →) ou de entrada (dir ←) de índice 0.
    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        if dir.x >= 0.0 {
            self.port_out_pos(0)
        } else {
            self.port_in_pos(0)
        }
    }

    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut res = Vec::new();

        let center = ctx.meta.canvas_to_screen_pos(self.pos);
        let sx = ctx.meta.canvas_to_screen_size(self.half.x);
        let sy = ctx.meta.canvas_to_screen_size(self.half.y);
        let rect = Rect::from_center_size(center, Vec2::new(sx * 2.0, sy * 2.0));

        let accent = self.color.unwrap_or_else(|| Color32::from_gray(190));
        let fill = Color32::from_rgb(30, 30, 40);
        let stroke_w = if self.selected {
            2.5
        } else if self.hovered {
            2.0
        } else {
            1.5
        };
        // Feedback de seleção: borda vermelha ao redor do nó.
        let borda = if self.selected {
            Color32::from_rgb(235, 70, 70)
        } else {
            accent
        };

        // Sombra
        let shadow = rect.translate(Vec2::new(3.0, 5.0));
        res.push(
            RectShape::new(
                shadow,
                CornerRadius::same(NODE_RADIUS),
                Color32::from_rgba_unmultiplied(0, 0, 0, 70),
                Stroke::NONE,
                StrokeKind::Inside,
            )
            .into(),
        );

        // Corpo do card
        res.push(
            RectShape::new(
                rect,
                CornerRadius::same(NODE_RADIUS),
                fill,
                Stroke::new(stroke_w, borda),
                StrokeKind::Inside,
            )
            .into(),
        );

        // Header colorido (apenas cantos superiores arredondados). Altura em
        // canvas → escala com o zoom (o nó inteiro cresce/encolhe junto).
        let header_h = ctx.meta.canvas_to_screen_size(node_component::CABECALHO_H);
        let header_rect =
            Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.min.y + header_h));
        let mut cr = CornerRadius::same(NODE_RADIUS);
        cr.sw = 0;
        cr.se = 0;
        res.push(
            RectShape::new(
                header_rect,
                cr,
                accent,
                Stroke::NONE,
                StrokeKind::Inside,
            )
            .into(),
        );

        // Rótulo (nome do nó) no cabeçalho colorido; a fonte escala com o zoom
        let fonte = ctx.meta.canvas_to_screen_size(node_component::FONTE_TITULO);
        let galley = ctx.painter.layout_no_wrap(
            self.label.clone(),
            FontId::new(fonte, FontFamily::Proportional),
            Color32::from_rgb(20, 20, 26),
        );
        let label_pos = Pos2::new(
            header_rect.center().x - galley.size().x / 2.0,
            header_rect.center().y - galley.size().y / 2.0,
        );
        res.push(
            TextShape::new(label_pos, galley, Color32::from_rgb(20, 20, 26)).into(),
        );

        // Portos de conexão: uma "bolinha" por parâmetro, alinhada ao corpo
        // do card. Entradas à esquerda (soquinhos ocos) e saídas à direita
        // (triângulos apontando p/ fora), na cor do tipo, sem borda branca.
        let zoom = ctx.meta.zoom;
        let port_r = (4.5 * zoom).clamp(2.5, 7.0);
        let (ins, outs) = port_offsets(self.tipo(), self.half);
        for off in &ins {
            let c = ctx.meta.canvas_to_screen_pos(self.pos + *off);
            // soquinho: círculo preenchido + furo na cor do corpo (aspecto de "entrada")
            res.push(
                Shape::Circle(CircleShape {
                    center: c,
                    radius: port_r,
                    fill: accent,
                    stroke: Stroke::NONE,
                })
                .into(),
            );
            res.push(
                Shape::Circle(CircleShape {
                    center: c,
                    radius: port_r * 0.45,
                    fill,
                    stroke: Stroke::NONE,
                })
                .into(),
            );
        }
        for (i, off) in outs.iter().enumerate() {
            let c = ctx.meta.canvas_to_screen_pos(self.pos + *off);
            // plug: bolinha preenchida (sem borda branca)
            res.push(
                Shape::Circle(CircleShape {
                    center: c,
                    radius: port_r,
                    fill: accent,
                    stroke: Stroke::NONE,
                })
                .into(),
            );
            // porto vetorial (ex.: Posição X/Y/Z): anel interno na cor do
            // corpo para indicar que, ao soltar o fio, abre o menu de
            // componentes (X / Y / Z).
            if portos(self.tipo()).saidas.get(i).map_or(false, |p| p.is_vetor()) {
                res.push(
                    Shape::Circle(CircleShape {
                        center: c,
                        radius: port_r * 0.45,
                        fill,
                        stroke: Stroke::NONE,
                    })
                    .into(),
                );
            }
        }

        res
    }

    fn update(&mut self, state: &NodeProps<()>) {
        self.pos = state.location();
        self.label = state.label.to_string();
        self.color = state.color();
        self.selected = state.selected;
        self.dragged = state.dragged;
        self.hovered = state.hovered;
        self.half = NoDisplay::tamanho(&self.label);
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        (pos.x - self.pos.x).abs() <= self.half.x
            && (pos.y - self.pos.y).abs() <= self.half.y
    }
}

/// Aresta sempre curva (S suave) entre dois portos específicos de nós.
#[derive(Clone)]
pub struct ArestaCurva {
    selected: bool,
    saida: usize,
    entrada: usize,
}

impl From<EdgeProps<ArestaInfo>> for ArestaCurva {
    fn from(e: EdgeProps<ArestaInfo>) -> Self {
        ArestaCurva {
            selected: e.selected,
            saida: e.payload.saida,
            entrada: e.payload.entrada,
        }
    }
}

impl DisplayEdge<(), ArestaInfo, Directed, DefaultIx, NoDisplay> for ArestaCurva {
    fn is_inside(
        &self,
        _start: &Node<(), ArestaInfo, Directed, DefaultIx, NoDisplay>,
        _end: &Node<(), ArestaInfo, Directed, DefaultIx, NoDisplay>,
        _pos: Pos2,
    ) -> bool {
        false
    }

    fn shapes(
        &mut self,
        start: &Node<(), ArestaInfo, Directed, DefaultIx, NoDisplay>,
        end: &Node<(), ArestaInfo, Directed, DefaultIx, NoDisplay>,
        ctx: &DrawContext,
    ) -> Vec<Shape> {
        let p0 = ctx
            .meta
            .canvas_to_screen_pos(start.display().port_out_pos(self.saida));
        let p3 = ctx
            .meta
            .canvas_to_screen_pos(end.display().port_in_pos(self.entrada));

        let dx = ((p3.x - p0.x).abs() * 0.5).max(30.0);
        let p1 = Pos2::new(p0.x + dx, p0.y);
        let p2 = Pos2::new(p3.x - dx, p3.y);

        let cor = if self.selected {
            Color32::from_rgb(120, 220, 140)
        } else {
            Color32::from_gray(160)
        };
        let stroke = Stroke::new(2.0, cor);

        let shape = eframe::egui::epaint::CubicBezierShape::from_points_stroke(
            [p0, p1, p2, p3],
            false,
            Color32::TRANSPARENT,
            stroke,
        );
        vec![shape.into()]
    }

    fn update(&mut self, state: &EdgeProps<ArestaInfo>) {
        self.selected = state.selected;
        self.saida = state.payload.saida;
        self.entrada = state.payload.entrada;
    }
}
