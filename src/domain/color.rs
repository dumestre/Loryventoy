/// Cor RGBA do domínio. Não depende de `egui` ou de outro renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self::from_rgba(255, 255, 255, 255);

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgba_unmultiplied(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_rgba_premultiplied(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn to_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    // Métodos compatíveis com Color32
    pub fn r(self) -> u8 {
        self.r
    }
    pub fn g(self) -> u8 {
        self.g
    }
    pub fn b(self) -> u8 {
        self.b
    }
    pub fn a(self) -> u8 {
        self.a
    }
}

/// Re-export glam::Vec2 para uso no domínio (biblioteca matemática pura).
pub use glam::Vec2;

/// Alias para clareza semântica.
pub type Pos2 = Vec2;
