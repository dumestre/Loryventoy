//! Exportação do frame atual (e de uma sequência de frames) do preview para
//! arquivos PNG, sem depender da tela (off-screen).
//!
//! A rasterização é feita manualmente: as `Shape`s do egui são tessaladas em
//! `Mesh`es pelo `epaint::Tessellator`, e os triângulos são pintados num
//! `ColorImage` (buffer largura×altura do projeto) por rasterização de triângulos
//! com interpolação baricêntrica e alpha blending simples. O `ColorImage` é
//! então salvo em disco via a crate `image`.

#![allow(dead_code)] // TODO(render): reutilizadas pela futura tela de render

use std::path::Path;

use eframe::egui::epaint::{
    ClippedPrimitive, ClippedShape, Mesh, Primitive, TessellationOptions, Tessellator,
};
use eframe::egui::{Color32, ColorImage, Pos2, Rect, Shape, Vec2};

use crate::procedural::{CenaPreview, PenPath, PreviewData, TextoItem};
use crate::ui::preview::PreviewPanel;
use crate::ui::text_raster::TextRaster;

/// Deslocamento (em pixels) de coordenadas de projeto para coordenadas de
/// buffer de imagem. A origem (0,0) do projeto é o CANTO SUPERIOR-ESQUERDO
/// (igual ao preview), e o buffer tem exatamente a resolução do projeto —
/// logo o mapeamento é 1:1 sem deslocamento (o centro do canvas é
/// (largura/2, altura/2)).
fn deslocamento(_data: &PreviewData) -> Vec2 {
    Vec2::ZERO
}

/// Traduz uma `Shape` (coords de projeto) para coords de buffer de imagem.
fn traduzir(shape: Shape, off: Vec2) -> Shape {
    let f = |p: Pos2| Pos2::new(p.x + off.x, p.y + off.y);
    match shape {
        Shape::Rect(mut r) => {
            r.rect = Rect::from_min_max(f(r.rect.min), f(r.rect.max));
            Shape::Rect(r)
        }
        Shape::Ellipse(mut e) => {
            e.center = f(e.center);
            Shape::Ellipse(e)
        }
        Shape::Path(mut p) => {
            p.points = p.points.into_iter().map(f).collect();
            Shape::Path(p)
        }
        other => other,
    }
}

/// Tessela um conjunto de `Shape`s (em coords de buffer) numa lista de `Mesh`es.
fn tessalar(shapes: Vec<Shape>) -> Vec<Mesh> {
    let mut tess = Tessellator::new(
        1.0,
        TessellationOptions::default(),
        [0, 0], // sem textura de fonte (texto tratado à parte)
        Vec::new(),
    );
    let mut out: Vec<ClippedPrimitive> = Vec::new();
    for s in shapes {
        tess.tessellate_clipped_shape(
            ClippedShape {
                clip_rect: Rect::EVERYTHING,
                shape: s,
            },
            &mut out,
        );
    }
    out.into_iter()
        .filter_map(|cp| match cp.primitive {
            Primitive::Mesh(m) => Some(m),
            Primitive::Callback(_) => None,
        })
        .collect()
}

/// Pinta os triângulos de uma `Mesh` no `ColorImage`, com alpha blending:
/// `dst = src*src_a + dst*(1 - src_a)` (cores em straight alpha). `opac` é a
/// opacidade da cena (0..1), multiplicada pelo alpha de cada vértice.
fn rasterizar_mesh(img: &mut ColorImage, mesh: &Mesh, opac: f32) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    let vs = &mesh.vertices;
    if vs.is_empty() {
        return;
    }
    for tri in mesh.indices.chunks(3) {
        if tri.len() != 3 {
            continue;
        }
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= vs.len() || i1 >= vs.len() || i2 >= vs.len() {
            continue;
        }
        let v0 = vs[i0].pos;
        let v1 = vs[i1].pos;
        let v2 = vs[i2].pos;

        let minx = (v0.x.min(v1.x).min(v2.x).floor()).max(0.0) as i32;
        let maxx = (v0.x.max(v1.x).max(v2.x).ceil()).min((w - 1) as f32) as i32;
        let miny = (v0.y.min(v1.y).min(v2.y).floor()).max(0.0) as i32;
        let maxy = (v0.y.max(v1.y).max(v2.y).ceil()).min((h - 1) as f32) as i32;
        if maxx < minx || maxy < miny {
            continue;
        }

        let denom = (v1.y - v2.y) * (v0.x - v2.x) + (v2.x - v1.x) * (v0.y - v2.y);
        if denom.abs() < 1e-6 {
            continue;
        }

        for py in miny..=maxy {
            for px in minx..=maxx {
                let x = px as f32 + 0.5;
                let y = py as f32 + 0.5;
                let w0 = ((v1.y - v2.y) * (x - v2.x) + (v2.x - v1.x) * (y - v2.y)) / denom;
                let w1 = ((v2.y - v0.y) * (x - v2.x) + (v0.x - v2.x) * (y - v2.y)) / denom;
                let w2 = 1.0 - w0 - w1;
                if w0 < -0.001 || w1 < -0.001 || w2 < -0.001 {
                    continue;
                }
                let c0 = vs[i0].color;
                let c1 = vs[i1].color;
                let c2 = vs[i2].color;

                let r = c0.r() as f32 * w0 + c1.r() as f32 * w1 + c2.r() as f32 * w2;
                let g = c0.g() as f32 * w0 + c1.g() as f32 * w1 + c2.g() as f32 * w2;
                let b = c0.b() as f32 * w0 + c1.b() as f32 * w1 + c2.b() as f32 * w2;
                let a = (c0.a() as f32 * w0 + c1.a() as f32 * w1 + c2.a() as f32 * w2) * opac;
                let sa = (a / 255.0).clamp(0.0, 1.0);

                let dst = img[(px as usize, py as usize)];
                let na = (a + dst.a() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
                let nr = (r * sa + dst.r() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
                let ng = (g * sa + dst.g() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
                let nb = (b * sa + dst.b() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
                img[(px as usize, py as usize)] = Color32::from_rgba_unmultiplied(nr, ng, nb, na);
            }
        }
    }
}

/// Desenha uma imagem de texto (RGBA straight) no buffer, na posição
/// `(bx, by)` de coords de buffer (canto superior-esquerdo), aplicando a
/// opacidade da cena.
fn blitar_texto(img: &mut ColorImage, txt_img: &ColorImage, bx: f32, by: f32, opac: f32) {
    let tw = txt_img.width();
    let th = txt_img.height();
    for ty in 0..th {
        let by = (by as i32) + ty as i32;
        if by < 0 || by >= img.height() as i32 {
            continue;
        }
        for tx in 0..tw {
            let bx = (bx as i32) + tx as i32;
            if bx < 0 || bx >= img.width() as i32 {
                continue;
            }
            let src = txt_img[(tx, ty)];
            let sa = (src.a() as f32 / 255.0 * opac).clamp(0.0, 1.0);
            if sa <= 0.001 {
                continue;
            }
            let dst = img[(bx as usize, by as usize)];
            let na = (src.a() as f32 + dst.a() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            let nr = (src.r() as f32 * sa + dst.r() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            let ng = (src.g() as f32 * sa + dst.g() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            let nb = (src.b() as f32 * sa + dst.b() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            img[(bx as usize, by as usize)] = Color32::from_rgba_unmultiplied(nr, ng, nb, na);
        }
    }
}

/// Renderiza uma cena (formas + pen + textos) no instante `t` num `ColorImage`.
fn renderizar_cena(img: &mut ColorImage, cena: &CenaPreview, t: f32, off: Vec2) {
    if cena.opacidade <= 0.001 {
        return;
    }
    let opac = cena.opacidade;

    // ---- Formas procedurais ----
    // Cada forma pode ter opacidade animada própria (alvo=Opacidade), então
    // rasterizamos uma a uma com a opacidade (cena × objeto).
    for gen in &cena.formas {
        let s = traduzir(gen.generate(t), off);
        let op = opac * gen.opac_em(t);
        for m in &tessalar(vec![s]) {
            rasterizar_mesh(img, m, op);
        }
    }

    // ---- Pen (caminhos DSL avaliados no tempo `t`) ----
    // Cada caneta pode ter opacidade animada própria, então rasterizamos
    // uma a uma com a opacidade (cena × objeto).
    let mut raster_pen = TextRaster::new();
    for pen in &cena.pen {
        let cmds = match pen.program.eval(t, pen.seed) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let shapes_pen = pen_para_shapes(pen, t, off);
        let op = opac * pen.opac_em(t);
        for m in &tessalar(shapes_pen) {
            rasterizar_mesh(img, m, op);
        }
        // Textos da caneta (comando `text`), rasterizados via cosmic-text.
        let penpos = pen.pos_em(t);
        let penx = penpos.x;
        let peny = penpos.y;
        for pt in crate::dsl::extrair_textos(&cmds) {
            desenhar_texto_pen(
                img,
                &mut raster_pen,
                penx + pt.x,
                peny + pt.y,
                &pt,
                op,
            );
        }
    }

    // ---- Textos (rasterizados via cosmic-text) ----
    let mut raster = TextRaster::new();
    for txt in &cena.textos {
        desenhar_texto(img, &mut raster, txt, opac * txt.opac_em(t), t);
    }
}

/// Converte um [`PenPath`] avaliado no tempo `t` nas `Shape`s do egui, já
/// traduzidas para coords de buffer.
fn pen_para_shapes(pen: &PenPath, t: f32, off: Vec2) -> Vec<Shape> {
    let program = &pen.program;
    let cmds = match program.eval(t, pen.seed) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let to_buf = |p: Pos2| Pos2::new(p.x + off.x, p.y + off.y);
    let id = |v: Vec2| v;
    // opacidade é aplicada depois, na rasterização, então passamos 1.0 aqui.
    PreviewPanel::pen_cmds_para_shapes(
        &cmds,
        pen.pos_em(t),
        pen.cor,
        pen.cor_fill,
        pen.espessura,
        pen.preenchimento,
        pen.cantos,
        pen.escala_x,
        pen.escala_y,
        &to_buf,
        &id,
        1.0,
    )
}

/// Rasteriza `txt` e o desenha no buffer.
fn desenhar_texto(img: &mut ColorImage, raster: &mut TextRaster, txt: &TextoItem, opac: f32, t: f32) {
    // Export é 1:1 (px de projeto), então escala = 1.0.
    let txt_r = match raster.raster(
        &txt.conteudo,
        txt.tamanho,
        1.0,
        txt.negrito,
        txt.italico,
        txt.cor,
    ) {
        Some(i) => i,
        None => return,
    };
    // (px, py) é o canto superior-esquerdo do texto em coords de projeto
    // (origem no canto sup-esq, igual ao preview) — com Ruído no tempo `t`.
    let (px, py) = txt.pos_em(t);
    let bx = px;
    let by = py;
    blitar_texto(img, &txt_r.imagem, bx, by, opac);
}

/// Desenha um texto da caneta no buffer, respeitando `alinhamento` (horizontal)
/// e `rotacao` (graus, em torno do canto superior-esquerdo). Export é 1:1.
fn desenhar_texto_pen(
    img: &mut ColorImage,
    raster: &mut TextRaster,
    px: f32,
    py: f32,
    pt: &crate::dsl::PenText,
    opac: f32,
) {
    let txt_r = match raster.raster(
        &pt.conteudo,
        pt.tamanho,
        1.0,
        pt.negrito,
        pt.italico,
        pt.cor,
    ) {
        Some(i) => i,
        None => return,
    };
    // Alinhamento horizontal: desloca o canto de referência.
    let largura = txt_r.tam_logico[0];
    let dx = match pt.alinhamento {
        crate::dsl::TextoAlinhamento::Left => 0.0,
        crate::dsl::TextoAlinhamento::Center => -largura / 2.0,
        crate::dsl::TextoAlinhamento::Right => -largura,
    };
    let bx = px + dx;
    let by = py;
    if pt.rotacao.abs() < 0.001 {
        blitar_texto(img, &txt_r.imagem, bx, by, opac);
    } else {
        blitar_texto_rot(img, &txt_r.imagem, bx, by, pt.rotacao, opac);
    }
}

/// Blita uma imagem de texto rotacionada (em graus) em torno do canto
/// superior-esquerdo (bx, by), aplicando alpha composto.
fn blitar_texto_rot(
    img: &mut ColorImage,
    txt_img: &ColorImage,
    bx: f32,
    by: f32,
    rot_graus: f32,
    opac: f32,
) {
    let tw = txt_img.width() as f32;
    let th = txt_img.height() as f32;
    let rot = rot_graus.to_radians();
    let (cs, sn) = (rot.cos(), rot.sin());
    // centro da imagem
    let cx = tw / 2.0;
    let cy = th / 2.0;
    for ty in 0..txt_img.height() {
        for tx in 0..txt_img.width() {
            let src = txt_img[(tx, ty)];
            let sa = (src.a() as f32 / 255.0 * opac).clamp(0.0, 1.0);
            if sa <= 0.001 {
                continue;
            }
            // posição relativa ao centro, rotacionada
            let rx = tx as f32 - cx;
            let ry = ty as f32 - cy;
            let wx = rx * cs - ry * sn + cx + bx;
            let wy = rx * sn + ry * cs + cy + by;
            let px = wx.round() as i32;
            let py = wy.round() as i32;
            if px < 0 || px >= img.width() as i32 || py < 0 || py >= img.height() as i32 {
                continue;
            }
            let dst = img[(px as usize, py as usize)];
            let na = (src.a() as f32 + dst.a() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            let nr = (src.r() as f32 * sa + dst.r() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            let ng = (src.g() as f32 * sa + dst.g() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            let nb = (src.b() as f32 * sa + dst.b() as f32 * (1.0 - sa)).clamp(0.0, 255.0) as u8;
            img[(px as usize, py as usize)] = Color32::from_rgba_unmultiplied(nr, ng, nb, na);
        }
    }
}

/// Converte um `ColorImage` num `Vec<u8>` RGBA (8 bits por canal).
fn color_image_para_rgba(img: &ColorImage) -> Vec<u8> {
    let mut out = Vec::with_capacity(img.pixels.len() * 4);
    for c in &img.pixels {
        out.push(c.r());
        out.push(c.g());
        out.push(c.b());
        out.push(c.a());
    }
    out
}

/// Exporta o frame atual do preview (em `data`, no instante `t`) para um PNG.
pub fn exportar_png(data: &PreviewData, t: f32, caminho: &Path) -> Result<(), String> {
    let w = data.largura.max(1.0) as u32;
    let h = data.altura.max(1.0) as u32;
    let mut img = ColorImage::new(
        [w as usize, h as usize],
        vec![data.fundo; (w as usize) * (h as usize)],
    );
    let off = deslocamento(data);

    for cena in &data.cenas {
        renderizar_cena(&mut img, cena, t, off);
    }

    let rgba = color_image_para_rgba(&img);
    image::save_buffer(
        caminho,
        &rgba,
        w,
        h,
        image::ColorType::Rgba8,
    )
    .map_err(|e| format!("falha ao salvar PNG {}: {e}", caminho.display()))?;
    Ok(())
}

/// Exporta uma sequência de frames (vídeo) como PNGs em `dir`, variando o tempo
/// de 0 até `duracao_seg` em passos de `1/fps`.
pub fn exportar_frames(
    data: &PreviewData,
    fps: f32,
    duracao_seg: f32,
    dir: &Path,
) -> Result<usize, String> {
    let fps = fps.max(1.0);
    let total = (duracao_seg * fps).ceil().max(1.0) as u32;
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("falha ao criar pasta {}: {e}", dir.display()))?;

    let mut contagem = 0usize;
    for f in 0..total {
        let t = f as f32 / fps;
        let nome = format!("frame_{:04}.png", f + 1);
        let caminho = dir.join(nome);
        exportar_png(data, t, &caminho)?;
        contagem += 1;
    }
    Ok(contagem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::procedural::{CenaPreview, GVec2, PenPath, PreviewData};
    use crate::dsl::Program;
    use eframe::egui::Color32;

    #[test]
    fn pen_com_text_desenha_pixels_no_buffer() {
        // Monta um PreviewData mínimo com um pen cujo programa tem `text`
        // e verifica que o PNG resultante contém pixels coloridos (texto)
        // onde deveria (perto de 100,200 em coords de projeto 1:1).
        let mut data = PreviewData::default();
        data.largura = 400.0;
        data.altura = 400.0;
        data.fundo = Color32::BLACK;

        let mut cena = CenaPreview::default();
        cena.opacidade = 1.0;
        let program = Program::parse("color 1 1 0\ntext \"OI\" 100 200 64").unwrap();
        cena.pen.push(PenPath {
            program,
            pos: GVec2::new(0.0, 0.0),
            cor: Color32::YELLOW,
            cor_fill: Color32::YELLOW,
            espessura: 2.0,
            preenchimento: false,
            seed: 1,
            cantos: 0.0,
            ordem: 0.0,
            escala_x: 1.0,
            escala_y: 1.0,
            ruido: None,
            anim: None,
        });
        data.cenas.push(cena);

        let tmp = std::env::temp_dir().join("teste_pen_text.png");
        exportar_png(&data, 0.0, &tmp).expect("exporta");
        let img = image::open(&tmp).expect("abre png").to_rgba8();
        std::fs::remove_file(&tmp).ok();
        // conta pixels não-preto (texto amarelo deve existir)
        let nao_preto = img
            .pixels()
            .filter(|p| (p[0] as u32 + p[1] as u32 + p[2] as u32) > 30)
            .count();
        assert!(nao_preto > 50, "esperava pixels de texto, veio {nao_preto}");
    }
}
