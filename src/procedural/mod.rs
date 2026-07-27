//! Módulo procedural — fachada pública que expõe:
//! - `procedural::domain`: tipos e avaliação puros (sem egui)
//! - `procedural::render`: conversão para `egui::Shape`

pub mod domain;
pub mod render;

pub use domain::*;