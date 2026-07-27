//! Conversão de formas do domínio para `egui::Shape`.
//! Mantém o conhecimento de rendering isolado aqui.

use crate::procedural::domain::Shape;
use eframe::egui::{Color32, Pos2, Rect, Shape as EguiShape, Stroke, Vec2};
use eframe::egui::epaint::{EllipseShape, PathShape, RectShape};
use crate::domain::Color;

/// Converte uma `Color` do domínio para `egui::Color32`.
pub fn color_to_color32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// Converte uma `Shape` do domínio para `egui::Shape`.
pub fn shape_to_egui(shape: Shape) -> EguiShape {
    match shape {
        Shape::Rect { c, tam, corner_radius, cor } => {
            let rect = Rect::from_center_size(Pos2::new(c.x, c.y), Vec2::new(tam.x, tam.y));
            if corner_radius > 0 {
                EguiShape::Rect(RectShape::filled(rect, corner_radius, color_to_color32(cor)))
            } else {
                EguiShape::Rect(RectShape::filled(rect, 0, color_to_color32(cor)))
            }
        }
        Shape::Ellipse { c, rx, ry, cor } => {
            EguiShape::Ellipse(EllipseShape::filled(Pos2::new(c.x, c.y), Vec2::new(rx, ry), color_to_color32(cor)))
        }
        Shape::Path { pts, cor } => {
            if pts.len() < 2 {
                return EguiShape::Noop;
            }
            let egui_pts: Vec<Pos2> = pts.iter().map(|p| Pos2::new(p.x, p.y)).collect();
            let mut path = PathShape::closed_line(egui_pts, Stroke::NONE);
            path.fill = color_to_color32(cor);
            EguiShape::Path(path)
        }
    }
}

/// Converte um `ShapeGenerator` do domínio para `egui::Shape` no tempo `t`.
pub fn generate_shape_egui(gen: &crate::procedural::domain::ShapeGenerator, t: f32) -> EguiShape {
    shape_to_egui(gen.generate(t))
}