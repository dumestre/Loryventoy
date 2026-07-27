#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoNo {
    Saida,
    Transform,
    Canvas,
    Cena,
    Layer,
    Shape,
    Texto,
    Pen,
    Ruido,
    Anim,
}

impl TipoNo {
    pub fn nome(&self) -> &'static str {
        match self {
            TipoNo::Saida => "Master",
            TipoNo::Transform => "Transform",
            TipoNo::Canvas => "Canvas",
            TipoNo::Cena => "Cena",
            TipoNo::Layer => "Layers",
            TipoNo::Shape => "Shape",
            TipoNo::Texto => "Texto",
            TipoNo::Pen => "Pen",
            TipoNo::Ruido => "Ruído",
            TipoNo::Anim => "Animação",
        }
    }

    pub fn from_label(label: &str) -> Option<TipoNo> {
        match label {
            "Master" => Some(TipoNo::Saida),
            "Transform" => Some(TipoNo::Transform),
            "Canvas" => Some(TipoNo::Canvas),
            "Cena" => Some(TipoNo::Cena),
            "Layers" => Some(TipoNo::Layer),
            "Shape" => Some(TipoNo::Shape),
            "Texto" => Some(TipoNo::Texto),
            "Pen" => Some(TipoNo::Pen),
            "Ruído" | "Ruido" => Some(TipoNo::Ruido),
            "Animação" | "Animacao" => Some(TipoNo::Anim),
            _ => None,
        }
    }

    pub fn instancia(&self) -> TipoNo {
        *self
    }

    pub fn pode_conectar(origem: TipoNo, destino: TipoNo) -> bool {
        match (origem, destino) {
            (TipoNo::Saida, _) => false,
            (_, TipoNo::Saida) => true,
            (TipoNo::Canvas, TipoNo::Cena) => true,
            (TipoNo::Cena, TipoNo::Cena) => true,
            (TipoNo::Layer, TipoNo::Shape | TipoNo::Texto | TipoNo::Pen) => true,
            (TipoNo::Shape, TipoNo::Cena) => true,
            (TipoNo::Texto, TipoNo::Cena) => true,
            (TipoNo::Pen, TipoNo::Cena) => true,
            (TipoNo::Ruido, TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen) => true,
            (TipoNo::Anim, TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen) => true,
            (
                o @ (TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen),
                TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen,
            ) if o != TipoNo::Saida => true,
            _ => false,
        }
    }
}
