use eframe::egui::{
    Color32,
    CornerRadius,
    CursorIcon,
    Key,
    Mesh,
    PointerButton,
    Pos2,
    Rect,
    Sense,
    Shape,
    Stroke,
    StrokeKind,
    Ui,
    Vec2,
};
use eframe::egui::epaint::{EllipseShape, PathShape, RectShape, Vertex};

use crate::procedural::{PreviewData, GVec2};
use crate::ui::scroll_delta;
use crate::ui::text_raster::TextRaster;


pub struct PreviewPanel {

    // deslocamento do pan (arrastar)
    pub offset: Vec2,

    // fator de zoom da cena
    zoom: f32,

    // tamanho da cena (canvas branco), na escala 1:1
    canvas_size: Vec2,

    // cor de fundo da cena (nó Canvas)
    cor_fundo: Color32,

    // dados completos do preview (cenas, formas e textos) vindos do grafo
    data: PreviewData,

    // instante de animação (segundos) para o sistema procedural
    tempo: f32,

    // rasterizador de texto (cosmic-text)
    raster: TextRaster,
    // erros de eval dos pens no frame atual (exibidos como overlay)
    pen_erros: Vec<String>,
}


impl PreviewPanel {

    pub fn new() -> Self {

        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
            canvas_size: Vec2::new(640.0, 360.0),
            cor_fundo: Color32::WHITE,
            data: PreviewData::default(),
            tempo: 0.0,
            raster: TextRaster::new(),
            pen_erros: Vec::new(),
        }
    }

    /// Define os dados completos do preview (vindos do grafo).
    pub fn set_preview(&mut self, data: PreviewData) {
        self.data = data;
    }

    /// Define o instante de animação (segundos) do sistema procedural.
    pub fn set_tempo(&mut self, tempo: f32) {
        self.tempo = tempo;
    }

    /// Retorna uma cópia dos dados completos do preview (cenas/formas/textos).
    /// Usado pela exportação off-screen (PNG) para rasterizar o frame atual.
    #[allow(dead_code)] // TODO(render): reutilizado pela tela de render
    pub fn preview_data(&self) -> PreviewData {
        self.data.clone()
    }

    /// Retorna o instante de animação (segundos) corrente do preview.
    #[allow(dead_code)] // TODO(render): reutilizado pela tela de render
    pub fn tempo_atual(&self) -> f32 {
        self.tempo
    }


    /// Ajusta o canvas da cena conforme a resolução do projeto (nó Canvas),
    /// preservando o aspect ratio. O maior lado é normalizado para uma base
    /// fixa, então o zoom continua controlando a escala visual.
    pub fn set_resolucao(&mut self, largura: u32, altura: u32, fundo: Color32) {
        let largura = largura.max(1) as f32;
        let altura = altura.max(1) as f32;
        const BASE: f32 = 640.0;
        let escala = BASE / largura.max(altura);
        self.canvas_size = Vec2::new(largura * escala, altura * escala);
        self.cor_fundo = fundo;
    }


    // Aplica um fator de zoom mantendo o ponto `anchor` (tela) fixo
    fn apply_zoom(
        &mut self,
        factor: f32,
        anchor: Pos2,
        screen_center: Pos2,
    ) {

        let new_zoom = (self.zoom * factor).clamp(0.1, 16.0);
        let factor = new_zoom / self.zoom;

        let d = anchor - screen_center;
        self.offset = d - (d - self.offset) * factor;
        self.zoom = new_zoom;
    }


    pub fn show(
        &mut self,
        ui: &mut Ui,
    ) {

        let size = Vec2::new(
            ui.available_width(),
            ui.available_height(),
        );


        // Aloca a área e detecta drag (para o botão do meio)
        let (rect, response) = ui.allocate_exact_size(
            size,
            Sense::drag(),
        );


        // Tecla F com o cursor sobre a área: recentraliza e reseta o zoom
        if response.hovered()
            && ui.ctx().input(|i| i.key_pressed(Key::F))
        {
            self.offset = Vec2::ZERO;
            self.zoom = 1.0;
        }


        let alt = ui.ctx().input(|i| i.modifiers.alt);


        // Gesto bruto independente da fonte (mouse, toque ou trackpad)
        let mut gesture = Vec2::ZERO;

        // Botão do meio do mouse (arrastar)
        if response.dragged_by(PointerButton::Middle) {
            gesture += response.drag_delta();
        }

        // Toque de 2 dedos (tela touch) — só importa sobre o canvas
        if ui.rect_contains_pointer(rect) {
            if let Some(touch) = ui.ctx().input(|i| i.multi_touch()) {
                if touch.num_touches >= 2 {
                    gesture += touch.translation_delta;
                }
            }

            // Gesto de 2 dedos no trackpad / scroll do mouse
            let scroll = scroll_delta(ui.ctx());
            if alt {
                // Alt + scroll = zoom no canvas (eixo vertical)
                gesture.y += scroll.y;
            } else {
                // scroll normal = pan
                gesture += scroll;
            }
        }


        if alt {
            // Alt + gesto = zoom seguindo o cursor (usa o componente vertical)
            let anchor = ui.ctx()
                .pointer_interact_pos()
                .unwrap_or_else(|| rect.center());
            let factor = (-gesture.y * 0.01).exp();
            self.apply_zoom(factor, anchor, rect.center());
            ui.ctx().request_repaint();
        } else if gesture != Vec2::ZERO {
            // Gesto normal = pan
            self.offset += gesture;
            ui.ctx().request_repaint();
        }


        // Cursor
        if response.dragged_by(PointerButton::Middle) {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        } else if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }


        let painter = ui.painter().with_clip_rect(rect);


        // Fundo escuro da janela preview
        painter.rect_filled(
            rect,
            CornerRadius::same(8),
            Color32::from_rgb(22, 22, 30),
        );


        // Canvas branco centralizado + offset do pan, na escala do zoom
        let scaled_size = self.canvas_size * self.zoom;
        let canvas_origin = rect.center() + self.offset - scaled_size / 2.0;
        let canvas_rect = Rect::from_min_size(
            canvas_origin,
            scaled_size,
        );


        // Sombra do canvas
        painter.rect_filled(
            canvas_rect.translate(Vec2::new(4.0, 4.0)),
            CornerRadius::same(2),
            Color32::from_rgba_unmultiplied(0, 0, 0, 80),
        );

        // Canvas branco (cena)
        painter.rect_filled(
            canvas_rect,
            CornerRadius::same(2),
            self.cor_fundo,
        );

        // Borda sutil do canvas
        painter.rect_stroke(
            canvas_rect,
            CornerRadius::same(2),
            Stroke::new(1.0, Color32::from_gray(200)),
            StrokeKind::Outside,
        );

        // ---- Cenas procedurais (sistema 100% procedural) ----
        // Mapeia coordenadas do projeto (0..largura, 0..altura) para o canvas
        // do preview. A origem (0,0) do projeto é o CANTO SUPERIOR-ESQUERDO
        // do canvas branco; o centro do canvas é (largura/2, altura/2). Um px
        // de projeto vira `proj_scale * zoom` px de tela, garantindo que os
        // objetos tenham o MESMO tamanho relativo ao canvas que terão no
        // export (sincronização preview ≡ export).
        let proj_w = self.data.largura.max(1.0);
        // fator px-de-projeto -> px-de-tela (canvas_size normaliza o maior
        // lado p/ a base; ×zoom aplica o zoom do usuário).
        let proj_scale = (self.canvas_size.x / proj_w).max(1e-6);
        let escala = proj_scale * self.zoom;
        // canto superior-esquerdo do canvas branco em tela = onde (0,0) cai.
        let canvas_min = canvas_rect.min;
        let para_tela = move |p: Pos2| -> Pos2 {
            canvas_min + p.to_vec2() * escala
        };
        let para_tela_v = move |v: Vec2| -> Vec2 { v * escala };

        // renovamos o cache de texturas a cada quadro
        let mut tex_idx = 0usize;
        // Clona a lista de cenas para soltar o empréstimo imutável de `self`
        // (necessário pois rasterizamos texto com `self.raster` mutavelmente
        // dentro do loop). Cenas são leves (referências/poucos itens).
        let cenas = self.data.cenas.clone();
        for cena in &cenas {
            if cena.opacidade <= 0.001 {
                continue;
            }
            let opac = cena.opacidade;
            for gen in &cena.formas {
                let shape = gen.generate(self.tempo);
                // opacidade da cena × opacidade animada do objeto.
                let op = opac * gen.opac_em(self.tempo);
                let shape = Self::aplicar_opacidade(shape, op);
                let tela = Self::translate_shape(shape, &para_tela, &para_tela_v);
                painter.add(tela);
            }
            for txt in &cena.textos {
                let (tx, ty) = txt.pos_em(self.tempo);
                let (esx, esy) = txt.escala_em(self.tempo);
                let r = match self.rasterizar_texto(
                    &txt.conteudo,
                    txt.tamanho,
                    txt.negrito,
                    txt.italico,
                    txt.cor,
                    escala,
                ) {
                    Some(r) => r,
                    None => continue,
                };
                let name = format!("preview_text_{}", tex_idx);
                tex_idx += 1;
                let handle = ui
                    .ctx()
                    .load_texture(name, r.imagem, eframe::egui::TextureOptions::LINEAR);
                let anchor = para_tela(Pos2::new(tx, ty));
                let size = Vec2::new(
                    r.tam_logico[0] * escala.max(0.05) * esx,
                    r.tam_logico[1] * escala.max(0.05) * esy,
                );
                let op = opac * txt.opac_em(self.tempo);
                let a = (op.clamp(0.0, 1.0) * 255.0) as u8;
                painter.image(
                    handle.id(),
                    Rect::from_min_size(anchor, size),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::from_white_alpha(a),
                );
            }
            // Ordena as canetas por `ordem` (z-order) antes de desenhar.
            let mut pen_ord: Vec<&crate::procedural::PenPath> = cena.pen.iter().collect();
            pen_ord.sort_by(|a, b| a.ordem.partial_cmp(&b.ordem).unwrap_or(std::cmp::Ordering::Equal));
            self.pen_erros.clear();
            for pen in pen_ord {
                let cmds = match pen.program.eval(self.tempo, pen.seed) {
                    Ok(c) => c,
                    Err(e) => {
                        self.pen_erros.push(e.to_string());
                        continue;
                    }
                };
                let shapes = Self::pen_cmds_para_shapes(
                    &cmds,
                    pen.pos_em(self.tempo),
                    pen.cor,
                    pen.cor_fill,
                    pen.espessura,
                    pen.preenchimento,
                    pen.cantos,
                    pen.escala_x,
                    pen.escala_y,
                    &para_tela,
                    &para_tela_v,
                    opac * pen.opac_em(self.tempo),
                );
                for s in shapes {
                    painter.add(s);
                }
                // Textos da caneta: rasterizados via cosmic-text (igual nó Texto),
                // reusando exatamente o mesmo caminho de desenho dos nós Texto.
                let penpos = pen.pos_em(self.tempo);
                let pen_escala = (pen.escala_x, pen.escala_y);
                let op_pen = opac * pen.opac_em(self.tempo);
                for pt in crate::dsl::extrair_textos(&cmds) {
                    let pos = (
                        penpos.x + pt.x * pen_escala.0,
                        penpos.y + pt.y * pen_escala.1,
                    );
                    let r = match self.rasterizar_texto(
                        &pt.conteudo,
                        pt.tamanho,
                        pt.negrito,
                        pt.italico,
                        pt.cor,
                        escala,
                    ) {
                        Some(r) => r,
                        None => continue,
                    };
                    let name = format!("preview_text_{}", tex_idx);
                    tex_idx += 1;
                    let handle = ui
                        .ctx()
                        .load_texture(name, r.imagem, eframe::egui::TextureOptions::LINEAR);
                    let a = (op_pen.clamp(0.0, 1.0) * 255.0) as u8;
                    // Alinhamento horizontal: desloca o canto de ancoragem.
                    let largura_log = r.tam_logico[0] * pen_escala.0;
                    let dx = match pt.alinhamento {
                        crate::dsl::TextoAlinhamento::Left => 0.0,
                        crate::dsl::TextoAlinhamento::Center => -largura_log / 2.0,
                        crate::dsl::TextoAlinhamento::Right => -largura_log,
                    };
                    let anchor_proj = Pos2::new(pos.0 + dx, pos.1);
                    let anchor = para_tela(anchor_proj);
                    let size = Vec2::new(
                        r.tam_logico[0] * escala.max(0.05) * pen_escala.0,
                        r.tam_logico[1] * escala.max(0.05) * pen_escala.1,
                    );
                    if pt.rotacao.abs() < 0.001 {
                        painter.image(
                            handle.id(),
                            Rect::from_min_size(anchor, size),
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::from_white_alpha(a),
                        );
                    } else {
                        // Texto rotacionado: desenha como um mesh de 2 triângulos
                        // com os 4 cantos girados em torno do canto superior-esq.
                        let rot = pt.rotacao.to_radians();
                        let (cs, sn) = (rot.cos(), rot.sin());
                        let giro = |v: Pos2| -> Pos2 {
                            anchor
                                + Vec2::new(
                                    (v.x - anchor.x) * cs - (v.y - anchor.y) * sn,
                                    (v.x - anchor.x) * sn + (v.y - anchor.y) * cs,
                                )
                        };
                        let p0 = giro(anchor);
                        let p1 = giro(anchor + Vec2::new(size.x, 0.0));
                        let p2 = giro(anchor + Vec2::new(size.x, size.y));
                        let p3 = giro(anchor + Vec2::new(0.0, size.y));
                        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
                        let tint = Color32::from_white_alpha(a);
                        let meshy = Mesh {
                            texture_id: handle.id(),
                            indices: vec![0, 1, 2, 0, 2, 3],
                            vertices: vec![
                                Vertex { pos: p0, uv: uv.min, color: tint },
                                Vertex { pos: p1, uv: Pos2::new(uv.max.x, uv.min.y), color: tint },
                                Vertex { pos: p2, uv: uv.max, color: tint },
                                Vertex { pos: p3, uv: Pos2::new(uv.min.x, uv.max.y), color: tint },
                            ],
                        };
                        painter.add(Shape::Mesh(meshy.into()));
                    }
                }
            }
        }

        // Overlay de erros de eval dos pens
        if !self.pen_erros.is_empty() {
            let mut y = rect.top() + 30.0;
            let x = rect.left() + 10.0;
            let painter = ui.painter();
            for e in &self.pen_erros {
                let galley = painter.layout_no_wrap(
                    format!("Pen error: {e}"),
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(255, 80, 80),
                );
                painter.galley(egui::pos2(x, y), galley, Color32::from_rgb(255, 80, 80));
                y += 18.0;
            }
        }

    }

    /// Aplica opacidade (0..1) à cor de uma forma antes de desenhá-la.
    fn aplicar_opacidade(shape: Shape, opac: f32) -> Shape {
        let a = (opac.clamp(0.0, 1.0) * 255.0) as u8;
        match shape {
            Shape::Rect(mut r) => {
                r.fill = Color32::from_rgba_unmultiplied(r.fill.r(), r.fill.g(), r.fill.b(), a);
                Shape::Rect(r)
            }
            Shape::Ellipse(mut e) => {
                e.fill = Color32::from_rgba_unmultiplied(e.fill.r(), e.fill.g(), e.fill.b(), a);
                Shape::Ellipse(e)
            }
            Shape::Path(mut p) => {
                p.fill = Color32::from_rgba_unmultiplied(p.fill.r(), p.fill.g(), p.fill.b(), a);
                Shape::Path(p)
            }
            other => other,
        }
    }

    /// Rasteriza um texto via cosmic-text (empréstimo mutável de `self.raster`),
    /// retornando a [`TextoRaster`] pronta para ser pintada pelo chamador (que
    /// faz `load_texture` + `painter.image`). Usado tanto pelos nós Texto quanto
    /// pelos textos da caneta (`PathCmd::Text`), garantindo o mesmo resultado.
    ///
    /// `pos` é o canto superior-esquerdo em coords de projeto; `escala_v` é a
    /// escala de eixo (1,1 na maioria dos casos).
    fn rasterizar_texto(
        &mut self,
        conteudo: &str,
        tamanho: f32,
        negrito: bool,
        italico: bool,
        cor: Color32,
        escala_total: f32,
    ) -> Option<crate::ui::text_raster::TextoRaster> {
        self.raster
            .raster(conteudo, tamanho, escala_total.max(0.05), negrito, italico, cor)
    }

    /// Traduz uma `Shape` (em coords de projeto) para as coords de tela via
    /// `f` (posição) e `fv` (vetor/tamanho), reproduzindo retângulos, elipses
    /// e polígonos. `fv` escala o tamanho junto com o zoom, para que as formas
    /// cresçam/encolham como o resto da cena.
    fn translate_shape(
        shape: Shape,
        f: &impl Fn(Pos2) -> Pos2,
        fv: &impl Fn(Vec2) -> Vec2,
    ) -> Shape {
        match shape {
            Shape::Rect(r) => {
                // escala o corner radius junto com a forma (senão o
                // arredondamento fica fixo e deforma no zoom).
                let escala = fv(Vec2::splat(1.0)).x.max(1e-3);
                let mut cr = r.corner_radius;
                cr.nw = ((cr.nw as f32 * escala).round().clamp(0.0, 255.0)) as u8;
                cr.ne = ((cr.ne as f32 * escala).round().clamp(0.0, 255.0)) as u8;
                cr.sw = ((cr.sw as f32 * escala).round().clamp(0.0, 255.0)) as u8;
                cr.se = ((cr.se as f32 * escala).round().clamp(0.0, 255.0)) as u8;
                Shape::Rect(RectShape {
                    rect: Rect::from_min_max(f(r.rect.min), f(r.rect.max)),
                    corner_radius: cr,
                    ..r
                })
            }
            Shape::Ellipse(e) => {
                // Usa o mesmo fator de escala das outras formas (fv aplicado a
                // um vetor unitário), para que a elipse acompanhe zoom/offset
                // igual a retângulos e paths (antes só usava o zoom, ficando
                // inconsistente).
                let escala = fv(Vec2::splat(1.0)).x.max(1e-3);
                Shape::Ellipse(EllipseShape {
                    center: f(e.center),
                    radius: e.radius * escala,
                    ..e
                })
            }
            Shape::Path(p) => Shape::Path(PathShape {
                points: p.points.into_iter().map(|pt| f(pt)).collect(),
                ..p
            }),
            other => other,
        }
    }

    /// Converte a lista de [`PathCmd`] avaliada do DSL em formas do egui.
    /// Usa lyon para tesselação de traço e preenchimento com suporte a
    /// join/cap arredondados, substituindo o flatten manual de bezier e o
    /// `PathShape` do egui.
    ///
    /// Pública para ser reutilizada pela exportação off-screen (PNG), que
    /// passa `para_tela`/`para_tela_v` mapeando coords de projeto direto para
    /// o buffer de imagem.
    pub fn pen_cmds_para_shapes<F, G>(
        cmds: &[crate::dsl::PathCmd],
        desloc: GVec2,
        cor_padrao: Color32,
        cor_fill_padrao: Color32,
        espessura_padrao: f32,
        preenche_padrao: bool,
        cantos: f32,
        escala_x: f32,
        escala_y: f32,
        para_tela: &F,
        para_tela_v: &G,
        opac: f32,
    ) -> Vec<Shape>
    where
        F: Fn(Pos2) -> Pos2,
        G: Fn(Vec2) -> Vec2,
    {
        use lyon::tessellation::{
            FillTessellator, FillOptions, StrokeTessellator, StrokeOptions,
            VertexBuffers, BuffersBuilder, FillVertexConstructor, StrokeVertexConstructor,
            FillVertex, StrokeVertex, LineJoin, LineCap,
        };
        use lyon::math::Point as LPoint;
        use crate::dsl::PathCmd;

        let sx = escala_x;
        let sy = escala_y;
        let a = (opac.clamp(0.0, 1.0) * 255.0) as u8;

        let mut out: Vec<Shape> = Vec::new();

        struct St {
            cor: Color32,
            cor_fill: Color32,
            esp: f32,
            preenche: bool,
        }

        let mut st = St { cor: cor_padrao, cor_fill: cor_fill_padrao, esp: espessura_padrao, preenche: preenche_padrao };

        let to_scr = |p: &GVec2| -> LPoint {
            let sp = para_tela(Pos2::new(desloc.x + p.x * sx, desloc.y + p.y * sy));
            LPoint::new(sp.x, sp.y)
        };

        struct SubPath {
            start: LPoint,
            ops: Vec<SubPathOp>,
            close: bool,
            cor: Color32,
            cor_fill: Color32,
            esp: f32,
            preenche: bool,
            single: bool,
        }

        enum SubPathOp {
            Line(LPoint),
            Cubic(LPoint, LPoint, LPoint),
        }

        let mut subs: Vec<SubPath> = Vec::new();
        let mut cur: Option<SubPath> = None;

        macro_rules! flush_sub {
            () => {
                if let Some(sp) = cur.take() {
                    subs.push(sp);
                }
            };
        }

        for c in cmds {
            match c {
                PathCmd::Move(p) => {
                    flush_sub!();
                    cur = Some(SubPath {
                        start: to_scr(p),
                        ops: Vec::new(),
                        close: false,
                        cor: st.cor,
                        cor_fill: st.cor_fill,
                        esp: st.esp,
                        preenche: st.preenche,
                        single: true,
                    });
                }
                PathCmd::Line(p) => {
                    if let Some(ref mut sp) = cur {
                        sp.ops.push(SubPathOp::Line(to_scr(p)));
                        sp.single = false;
                    } else {
                        let pt = to_scr(p);
                        cur = Some(SubPath {
                            start: pt,
                            ops: Vec::new(),
                            close: false,
                            cor: st.cor,
                            cor_fill: st.cor_fill,
                            esp: st.esp,
                            preenche: st.preenche,
                            single: false,
                        });
                    }
                }
                PathCmd::Bezier(c1, c2, p) => {
                    if let Some(ref mut sp) = cur {
                        sp.ops.push(SubPathOp::Cubic(to_scr(c1), to_scr(c2), to_scr(p)));
                        sp.single = false;
                    } else {
                        let pt = to_scr(p);
                        cur = Some(SubPath {
                            start: pt,
                            ops: Vec::new(),
                            close: false,
                            cor: st.cor,
                            cor_fill: st.cor_fill,
                            esp: st.esp,
                            preenche: st.preenche,
                            single: false,
                        });
                    }
                }
                PathCmd::Close => {
                    if let Some(ref mut sp) = cur {
                        sp.close = true;
                    }
                }
                PathCmd::Fill(b) => {
                    st.preenche = *b;
                    if *b && cur.is_some() {
                        cur.as_mut().unwrap().preenche = true;
                    }
                }
                PathCmd::Stroke(w) => st.esp = *w,
                PathCmd::Color(c) => { st.cor = *c; st.cor_fill = *c; }
                PathCmd::ColorStroke(c) => st.cor = *c,
                PathCmd::ColorFill(c) => st.cor_fill = *c,
                PathCmd::Text { .. } => {}
            }
        }
        flush_sub!();

        struct TessV { pos: [f32; 2] }
        struct FillC;
        impl FillVertexConstructor<TessV> for FillC {
            fn new_vertex(&mut self, v: FillVertex) -> TessV {
                let p = v.position();
                TessV { pos: [p.x, p.y] }
            }
        }
        struct StrokeC;
        impl StrokeVertexConstructor<TessV> for StrokeC {
            fn new_vertex(&mut self, v: StrokeVertex) -> TessV {
                let p = v.position();
                TessV { pos: [p.x, p.y] }
            }
        }

        for sp in subs {
            let co = Color32::from_rgba_unmultiplied(sp.cor.r(), sp.cor.g(), sp.cor.b(), a);
            let cfo = Color32::from_rgba_unmultiplied(sp.cor_fill.r(), sp.cor_fill.g(), sp.cor_fill.b(), a);

            if sp.single {
                let r = (para_tela_v(Vec2::splat(sp.esp)).x).max(1.0);
                out.push(Shape::Ellipse(EllipseShape::filled(
                    Pos2::new(sp.start.x, sp.start.y), Vec2::splat(r), co,
                )));
                continue;
            }

            let esp_tela = (para_tela_v(Vec2::splat(sp.esp)).x).max(0.5);

            let mut builder = lyon::path::Path::builder();
            builder.begin(sp.start);
            for op in &sp.ops {
                match op {
                    SubPathOp::Line(p) => { builder.line_to(*p); }
                    SubPathOp::Cubic(c1, c2, p) => { builder.cubic_bezier_to(*c1, *c2, *p); }
                }
            }
            if sp.close || sp.preenche {
                builder.close();
            }
            let path = builder.build();

            if sp.preenche {
                let mut fb: VertexBuffers<TessV, u32> = VertexBuffers::new();
                if FillTessellator::new()
                    .tessellate_path(&path, &FillOptions::default().with_tolerance(0.1),
                        &mut BuffersBuilder::new(&mut fb, FillC))
                    .is_ok()
                    && !fb.vertices.is_empty()
                {
                    out.push(Shape::from(egui::epaint::Mesh {
                        vertices: fb.vertices.iter().map(|v| Vertex {
                            pos: Pos2::new(v.pos[0], v.pos[1]),
                            uv: egui::epaint::WHITE_UV,
                            color: cfo,
                        }).collect(),
                        indices: fb.indices,
                        texture_id: Default::default(),
                    }));
                }
            }

            if esp_tela > 0.0 {
                let mut sb: VertexBuffers<TessV, u32> = VertexBuffers::new();
                let join = if cantos >= 0.5 { LineJoin::Round } else { LineJoin::Miter };
                if StrokeTessellator::new()
                    .tessellate_path(&path, &StrokeOptions::default()
                        .with_tolerance(0.1)
                        .with_line_join(join)
                        .with_line_cap(LineCap::Round)
                        .with_line_width(esp_tela),
                        &mut BuffersBuilder::new(&mut sb, StrokeC))
                    .is_ok()
                    && !sb.vertices.is_empty()
                {
                    out.push(Shape::from(egui::epaint::Mesh {
                        vertices: sb.vertices.iter().map(|v| Vertex {
                            pos: Pos2::new(v.pos[0], v.pos[1]),
                            uv: egui::epaint::WHITE_UV,
                            color: co,
                        }).collect(),
                        indices: sb.indices,
                        texture_id: Default::default(),
                    }));
                }
            }
        }

        out
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Program;

    #[test]
    fn pen_so_com_text_nao_gera_shapes_geometria() {
        // Regressão: um programa cujo único comando é `text` NÃO deve virar
        // forma geométrica alguma (os textos são desenhados à parte como
        // imagem, nunca como PathShape).
        let p = Program::parse("text \"OLA\" 100 200 64").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        let shapes = PreviewPanel::pen_cmds_para_shapes(
            &cmds,
            crate::procedural::GVec2::new(0.0, 0.0),
            Color32::WHITE,
            Color32::WHITE,
            2.0,
            false,
            0.0,
            1.0,
            1.0,
            &|p: Pos2| p,
            &|v: Vec2| v,
            1.0,
        );
        assert!(shapes.is_empty(), "esperado 0 shapes, veio {}", shapes.len());
        // e os textos devem ser extraíveis
        assert_eq!(crate::dsl::extrair_textos(&cmds).len(), 1);
    }
}
