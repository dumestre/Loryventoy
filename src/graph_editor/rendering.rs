#![allow(dead_code)]

use eframe::egui::{
    Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Shape, Stroke,
    StrokeKind, Vec2,
};
use eframe::egui::epaint::{CubicBezierShape, RectShape, TextShape};

use crate::nodes::TipoNo;
use crate::ui::node_component;

const NODE_RADIUS: u8 = 8;

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

    painter.add(Shape::Rect(RectShape::new(
        node_rect,
        CornerRadius::same(NODE_RADIUS),
        fill,
        Stroke::new(stroke_w, borda),
        StrokeKind::Inside,
    )));

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

pub fn desenhar_sombra(painter: &egui::Painter, node_rect: Rect) {
    let shadow = node_rect.translate(Vec2::new(3.0, 5.0));
    painter.add(RectShape::new(
        shadow,
        CornerRadius::same(NODE_RADIUS),
        Color32::from_rgba_unmultiplied(0, 0, 0, 70),
        Stroke::NONE,
        StrokeKind::Inside,
    ));
}

pub fn bezier_entre(
    p0: Pos2,
    p3: Pos2,
    selected: bool,
) -> Shape {
    let dx = ((p3.x - p0.x).abs() * 0.5).max(30.0);
    let p1 = Pos2::new(p0.x + dx, p0.y);
    let p2 = Pos2::new(p3.x - dx, p3.y);

    let cor = if selected {
        Color32::from_rgb(120, 220, 140)
    } else {
        Color32::from_gray(160)
    };
    let stroke = Stroke::new(2.0, cor);

    CubicBezierShape::from_points_stroke(
        [p0, p1, p2, p3],
        false,
        Color32::TRANSPARENT,
        stroke,
    )
    .into()
}

pub fn desenhar_grade(painter: &egui::Painter, rect: Rect, pan: Vec2, zoom: f32) {
    let step = 40.0 * zoom;
    if step < 8.0 {
        return;
    }
    let stroke = Stroke::new(0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 15));
    let origin_screen = rect.min.to_vec2() + pan;

    let mut x = ((origin_screen.x - rect.left()) % step + step) % step;
    while x < rect.width() {
        let sx = rect.left() + x;
        painter.line_segment([Pos2::new(sx, rect.top()), Pos2::new(sx, rect.bottom())], stroke);
        x += step;
    }
    let mut y = ((origin_screen.y - rect.top()) % step + step) % step;
    while y < rect.height() {
        let sy = rect.top() + y;
        painter.line_segment([Pos2::new(rect.left(), sy), Pos2::new(rect.right(), sy)], stroke);
        y += step;
    }
}
