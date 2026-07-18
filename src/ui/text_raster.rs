use std::collections::HashMap;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, SwashCache, Weight};
use eframe::egui::{Color32, ColorImage};

/// Chave de cache de uma rasterização de texto. O tamanho é quantizado em
/// pixels inteiros (tamanho final já com zoom) para reaproveitar texturas
/// entre quadros com o mesmo zoom.
#[derive(Clone, PartialEq, Eq, Hash)]
struct ChaveTexto {
    texto: String,
    px: u32,
    negrito: bool,
    italico: bool,
    cor: [u8; 4],
}

/// Resultado de uma rasterização: a imagem e o tamanho lógico (em px de
/// PROJETO 1:1) que ela representa, para o preview posicionar corretamente
/// independentemente da resolução em que foi rasterizada.
#[derive(Clone)]
pub struct TextoRaster {
    pub imagem: ColorImage,
    /// tamanho lógico (largura, altura) em px de projeto 1:1
    pub tam_logico: [f32; 2],
}

/// Rasterizador de texto para o preview procedural. Encapsula o
/// `FontSystem`/`SwashCache` do `cosmic-text` (com fontes do sistema) e
/// converte o texto em uma `ColorImage` (RGBA), com cache por
/// conteúdo/estilo/tamanho para não rasterizar o mesmo texto todo quadro.
pub struct TextRaster {
    font_system: FontSystem,
    swash: SwashCache,
    cache: HashMap<ChaveTexto, Option<TextoRaster>>,
}

impl TextRaster {
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash: SwashCache::new(),
            cache: HashMap::new(),
        }
    }

    /// Rasteriza `texto` na resolução FINAL desejada (`tamanho` já é o tamanho
    /// em pixels de tela, ou seja tamanho-de-projeto × escala × zoom). Assim a
    /// textura sai nítida em qualquer zoom, sem upscale borrado.
    ///
    /// A cor é embutida na textura; o chamador deve desenhar a imagem com uma
    /// tinta BRANCA (× opacidade) para não multiplicar a cor duas vezes.
    ///
    /// `tam_logico` no resultado é em px de projeto 1:1 (independe da escala),
    /// para o posicionamento no canvas ser estável.
    pub fn raster(
        &mut self,
        texto: &str,
        tamanho_px: f32,
        escala: f32,
        negrito: bool,
        italic: bool,
        cor: Color32,
    ) -> Option<TextoRaster> {
        if texto.trim().is_empty() {
            return None;
        }
        let escala = escala.max(0.05);
        let px = (tamanho_px * escala).round().clamp(1.0, 4096.0);
        let chave = ChaveTexto {
            texto: texto.to_string(),
            px: px as u32,
            negrito,
            italico: italic,
            cor: [cor.r(), cor.g(), cor.b(), cor.a()],
        };
        if let Some(hit) = self.cache.get(&chave) {
            return hit.clone();
        }
        let resultado = self.rasterizar(texto, px, escala, negrito, italic, cor);
        self.cache.insert(chave, resultado.clone());
        // Evita crescimento indefinido do cache (ex.: animando o tamanho).
        if self.cache.len() > 512 {
            self.cache.clear();
        }
        resultado
    }

    fn rasterizar(
        &mut self,
        texto: &str,
        px: f32,
        escala: f32,
        negrito: bool,
        italic: bool,
        cor: Color32,
    ) -> Option<TextoRaster> {
        let family = Family::SansSerif;
        let weight = if negrito { Weight::BOLD } else { Weight::NORMAL };
        let style = if italic {
            cosmic_text::Style::Italic
        } else {
            cosmic_text::Style::Normal
        };
        let attrs = Attrs::new().family(family).weight(weight).style(style);

        let largura = (texto.chars().count() as f32 * px * 0.7 + 40.0).max(40.0);
        let altura = (px * 1.6 + 20.0).max(20.0);
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(px, px * 1.2));
        buffer.set_size(&mut self.font_system, Some(largura), Some(altura));
        buffer.set_text(
            &mut self.font_system,
            texto,
            attrs,
            cosmic_text::Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut pixels: Vec<(i32, i32, Color32)> = Vec::new();
        buffer.draw(
            &mut self.font_system,
            &mut self.swash,
            cosmic_text::Color::rgba(cor.r(), cor.g(), cor.b(), cor.a()),
            |x, y, _w, _h, c| {
                if c.a() == 0 {
                    return;
                }
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                pixels.push((x, y, Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), c.a())));
            },
        );

        if pixels.is_empty() {
            return None;
        }
        let w = (max_x - min_x + 1).max(1) as usize;
        let h = (max_y - min_y + 1).max(1) as usize;
        let mut rgba = vec![0u8; w * h * 4];
        for (x, y, c) in pixels {
            let px_ = (x - min_x) as usize;
            let py_ = (y - min_y) as usize;
            if px_ < w && py_ < h {
                let o = (py_ * w + px_) * 4;
                rgba[o] = c.r();
                rgba[o + 1] = c.g();
                rgba[o + 2] = c.b();
                rgba[o + 3] = c.a();
            }
        }
        Some(TextoRaster {
            imagem: ColorImage::from_rgba_unmultiplied([w, h], &rgba),
            // tamanho lógico em px de PROJETO: divide pela escala usada.
            tam_logico: [w as f32 / escala, h as f32 / escala],
        })
    }
}
