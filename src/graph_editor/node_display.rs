use eframe::egui::{
    Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Shape, Stroke,
    StrokeKind, Vec2,
};
use eframe::egui::epaint::{CircleShape, RectShape, TextShape};
use egui_graphs::{DisplayEdge, DisplayNode, DrawContext, EdgeProps, MetadataFrame, Node, NodeProps};
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
        // Apenas sombra: o card, header, label e portos são desenhados em
        // camadas separadas (card na Area de conteúdo, portos em passada
        // própria) para resolver sobreposição corretamente.
        let center = ctx.meta.canvas_to_screen_pos(self.pos);
        let sx = ctx.meta.canvas_to_screen_size(self.half.x);
        let sy = ctx.meta.canvas_to_screen_size(self.half.y);
        let rect = Rect::from_center_size(center, Vec2::new(sx * 2.0, sy * 2.0));

        let shadow = rect.translate(Vec2::new(3.0, 5.0));
        vec![RectShape::new(
            shadow,
            CornerRadius::same(NODE_RADIUS),
            Color32::from_rgba_unmultiplied(0, 0, 0, 70),
            Stroke::NONE,
            StrokeKind::Inside,
        )
        .into()]
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

/// Desenha o card visual (fundo, borda, header colorido e rótulo) dentro
/// de uma `Area` de conteúdo. Chamado antes dos widgets do inspector para
/// que o card fique NA MESMA CAMADA que o conteúdo — resolve sobreposição.
pub fn desenhar_card(
    painter: &egui::Painter,
    node_rect: Rect,
    tipo: TipoNo,
    selected: bool,
    hovered: bool,
    label: &str,
    zoom: f32,
) {
    let fill = Color32::from_rgb(30, 30, 40);
    let accent = tipo.cor();
    let stroke_w = if selected {
        2.5
    } else if hovered {
        2.0
    } else {
        1.5
    };
    let borda = if selected {
        Color32::from_rgb(235, 70, 70)
    } else {
        accent
    };

    // Corpo do card
    painter.add(Shape::Rect(RectShape::new(
        node_rect,
        CornerRadius::same(NODE_RADIUS),
        fill,
        Stroke::new(stroke_w, borda),
        StrokeKind::Inside,
    )));

    // Header colorido (cantos superiores arredondados)
    let header_h = zoom * node_component::CABECALHO_H;
    let header_rect =
        Rect::from_min_max(node_rect.min, Pos2::new(node_rect.max.x, node_rect.min.y + header_h));
    let mut cr = CornerRadius::same(NODE_RADIUS);
    cr.sw = 0;
    cr.se = 0;
    painter.add(Shape::Rect(RectShape::new(
        header_rect,
        cr,
        accent,
        Stroke::NONE,
        StrokeKind::Inside,
    )));

    // Rótulo (nome do nó) centrado no header
    let fonte = zoom * node_component::FONTE_TITULO;
    let galley = painter.layout_no_wrap(
        label.to_string(),
        FontId::new(fonte, FontFamily::Proportional),
        Color32::from_rgb(20, 20, 26),
    );
    let label_pos = Pos2::new(
        header_rect.center().x - galley.size().x / 2.0,
        header_rect.center().y - galley.size().y / 2.0,
    );
    painter.add(TextShape::new(label_pos, galley, Color32::from_rgb(20, 20, 26)));
}

/// Desenha apenas os portos de conexão de um nó num `Painter` externo.
/// Usado para pintar portos NA FRENTE do conteúdo do nó ( Areas com
/// `Order::Foreground`), evitando que fiquem cobertos pelos widgets.
pub fn desenhar_portos(
    painter: &egui::Painter,
    meta: &MetadataFrame,
    pos_canvas: Pos2,
    half: Vec2,
    tipo: TipoNo,
) {
    let fill = Color32::from_rgb(30, 30, 40);
    let accent = tipo.cor();
    let zoom = meta.zoom;
    let port_r = (4.5 * zoom).clamp(2.5, 7.0);
    let (ins, outs) = port_offsets(tipo, half);

    for off in &ins {
        let c = meta.canvas_to_screen_pos(pos_canvas + *off);
        painter.add(Shape::Circle(CircleShape {
            center: c,
            radius: port_r,
            fill: accent,
            stroke: Stroke::NONE,
        }));
        painter.add(Shape::Circle(CircleShape {
            center: c,
            radius: port_r * 0.45,
            fill,
            stroke: Stroke::NONE,
        }));
    }

    for (i, off) in outs.iter().enumerate() {
        let c = meta.canvas_to_screen_pos(pos_canvas + *off);
        painter.add(Shape::Circle(CircleShape {
            center: c,
            radius: port_r,
            fill: accent,
            stroke: Stroke::NONE,
        }));
        if portos(tipo).saidas.get(i).map_or(false, |p| p.is_vetor()) {
            painter.add(Shape::Circle(CircleShape {
                center: c,
                radius: port_r * 0.45,
                fill,
                stroke: Stroke::NONE,
            }));
        }
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
