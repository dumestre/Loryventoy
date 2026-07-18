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
            for pen in pen_ord {
                let cmds = match pen.program.eval(self.tempo, pen.seed) {
                    Ok(c) => c,
                    Err(_) => continue,
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
    /// Cada sub-path (entre `Move`) vira um `PathShape` com traço e, se
    /// solicitado, preenchimento. As coordenadas de projeto são levadas para
    /// a tela via `para_tela` (posição) e `para_tela_v` (tamanho do traço).
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
        use crate::dsl::PathCmd;

        // aplica a escala de eixo aos pontos (antes de somar o deslocamento)
        let sx = escala_x;
        let sy = escala_y;

        let mut out = Vec::new();
        let mut cor = cor_padrao;
        let mut cor_fill = cor_fill_padrao;
        let mut esp = espessura_padrao;
        let mut preenche = preenche_padrao;
        let mut atual: Vec<Pos2> = Vec::new();
        let mut fechado = false;

        for c in cmds {
            match c {
                PathCmd::Move(p) => {
                    Self::pen_flush(
                        &mut out,
                        &atual,
                        cor,
                        cor_fill,
                        esp,
                        preenche,
                        fechado,
                        cantos,
                        para_tela,
                        para_tela_v,
                        opac,
                    );
                    atual.clear();
                    fechado = false;
                    atual.push(Pos2::new(desloc.x + p.x * sx, desloc.y + p.y * sy));
                }
                PathCmd::Line(p) => {
                    atual.push(Pos2::new(desloc.x + p.x * sx, desloc.y + p.y * sy));
                }
                PathCmd::Bezier(c1, c2, p) => {
                    // amostra a curva de Bézier cúbica em segmentos de linha.
                    // O ponto inicial é o último acumulado (já com `desloc`); se
                    // não houver `move` prévio, parte da própria origem do nó.
                    let base = *atual.last().unwrap_or(&Pos2::ZERO);
                    let sx0 = if atual.is_empty() { desloc.x } else { base.x };
                    let sy0 = if atual.is_empty() { desloc.y } else { base.y };
                    let start = Pos2::new(sx0, sy0);
                    let steps = 24u32;
                    for s in 1..=steps {
                        let u = s as f32 / steps as f32;
                        let iu = 1.0 - u;
                        let bx = iu * iu * iu * start.x
                            + 3.0 * iu * iu * u * (desloc.x + c1.x * sx)
                            + 3.0 * iu * u * u * (desloc.x + c2.x * sx)
                            + u * u * u * (desloc.x + p.x * sx);
                        let by = iu * iu * iu * start.y
                            + 3.0 * iu * iu * u * (desloc.y + c1.y * sy)
                            + 3.0 * iu * u * u * (desloc.y + c2.y * sy)
                            + u * u * u * (desloc.y + p.y * sy);
                        atual.push(Pos2::new(bx, by));
                    }
                }
                PathCmd::Close => {
                    // Apenas marca o sub-path como fechado: o tessellator liga
                    // o último ponto ao primeiro. NÃO duplicamos o 1º ponto,
                    // pois uma aresta de comprimento zero vira artefato
                    // (aba/spike tremulante) no anti-alias.
                    fechado = true;
                    if let (Some(first), Some(last)) = (atual.first(), atual.last()) {
                        if (first.x - last.x).abs() < 0.01 && (first.y - last.y).abs() < 0.01 {
                            atual.pop();
                        }
                    }
                }
                PathCmd::Fill(b) => preenche = *b,
                PathCmd::Stroke(w) => esp = *w,
                PathCmd::Color(c) => {
                    cor = *c;
                    cor_fill = *c;
                }
                PathCmd::ColorStroke(c) => cor = *c,
                PathCmd::ColorFill(c) => cor_fill = *c,
                // Texto é desenhado à parte (ver loop do pen no `show`),
                // fora da conversão para `Shape`s. Aqui apenas ignoramos.
                PathCmd::Text { .. } => {}
            }
        }
        Self::pen_flush(
            &mut out,
            &atual,
            cor,
            cor_fill,
            esp,
            preenche,
            fechado,
            cantos,
            para_tela,
            para_tela_v,
            opac,
        );
        out
    }

    /// Empurra o sub-path acumulado (`pts`) como um `PathShape` no egui.
    /// `preenche` controla o preenchimento; `fechado` controla se o contorno é
    /// fechado (aresta de volta ao início). Assim um path aberto com
    /// `fill on` é preenchido sem ganhar uma aresta fechada automática.
    fn pen_flush<F, G>(
        out: &mut Vec<Shape>,
        pts: &[Pos2],
        cor: Color32,
        cor_fill: Color32,
        esp: f32,
        preenche: bool,
        fechado: bool,
        cantos: f32,
        para_tela: &F,
        para_tela_v: &G,
        opac: f32,
    ) where
        F: Fn(Pos2) -> Pos2,
        G: Fn(Vec2) -> Vec2,
    {
        if pts.is_empty() {
            return;
        }
        let a = (opac.clamp(0.0, 1.0) * 255.0) as u8;
        // Ponto isolado (1 vértice): desenha como um pequeno disco para não
        // sumir silenciosamente.
        if pts.len() == 1 {
            let c = para_tela(pts[0]);
            let r = (para_tela_v(Vec2::splat(esp)).x).max(1.0);
            out.push(Shape::Ellipse(EllipseShape::filled(
                c,
                Vec2::splat(r),
                Color32::from_rgba_unmultiplied(cor.r(), cor.g(), cor.b(), a),
            )));
            return;
        }
        // Remove vértices CONSECUTIVOS coincidentes: arestas de comprimento
        // zero (ex.: um `line` que repete o ponto do `move`, ou um `close`
        // sobre o 1º ponto) viram "abas"/spikes tremulantes no anti-alias do
        // tessellator. Também descarta a duplicata do 1º ponto no fim quando o
        // path é fechado.
        let closed_pre = fechado || preenche;
        let mut limpo: Vec<Pos2> = Vec::with_capacity(pts.len());
        for p in pts {
            if let Some(u) = limpo.last() {
                if (u.x - p.x).abs() < 0.001 && (u.y - p.y).abs() < 0.001 {
                    continue;
                }
            }
            limpo.push(*p);
        }
        if closed_pre && limpo.len() > 1 {
            if let (Some(f), Some(l)) = (limpo.first().copied(), limpo.last().copied()) {
                if (f.x - l.x).abs() < 0.001 && (f.y - l.y).abs() < 0.001 {
                    limpo.pop();
                }
            }
        }
        if limpo.len() < 2 {
            return;
        }

        let cor_op = Color32::from_rgba_unmultiplied(cor.r(), cor.g(), cor.b(), a);
        let cor_fill_op =
            Color32::from_rgba_unmultiplied(cor_fill.r(), cor_fill.g(), cor_fill.b(), a);
        let stroke = Stroke::new((para_tela_v(Vec2::splat(esp)).x).max(0.5), cor_op);
        let path_stroke: eframe::egui::epaint::PathStroke = stroke.into();
        let fill = if preenche { cor_fill_op } else { Color32::TRANSPARENT };
        // cantos>=0.5 => arredondado: a versão atual do egui (0.35) não expõe
        // join/cap no `Stroke`, então arredondamos os vértices interiores
        // geometricamente (insere pontos ao longo dos cantos).
        let pontos = if cantos >= 0.5 {
            Self::arredondar_cantos(&limpo, fechado, esp)
        } else {
            limpo
        };
        let points: Vec<Pos2> = pontos.iter().map(|p| para_tela(*p)).collect();
        // O epaint/tessellator só aceita preenchimento em paths FECHADOS:
        // um `PathShape` com `fill` opaco e `closed == false` gera panic
        // ("You asked to fill a path that is not closed"). Portanto, se há
        // preenchimento, o path é tratado como fechado.
        let closed = fechado || preenche;
        out.push(Shape::Path(PathShape {
            points,
            closed,
            fill,
            stroke: path_stroke,
        }));
    }

    /// Insere pontos ao longo dos cantos do path para suavizar as quinas
    /// (quando `cantos>=0.5`). A egui 0.35 não expõe `StrokeJoin`, então o
    /// arredondamento é feito amostrando cada vértice interior com dois pontos
    /// deslocados ao longo das arestas adjacentes.
    fn arredondar_cantos(pts: &[Pos2], fechado: bool, _esp: f32) -> Vec<Pos2> {
        if pts.len() < 3 {
            return pts.to_vec();
        }
        let n = pts.len();
        let mut out = Vec::with_capacity(n * 2);
        let inicio = if fechado { 0 } else { 1 };
        let fim = if fechado { n } else { n - 1 };
        if !fechado {
            out.push(pts[0]);
        }
        for i in inicio..fim {
            let prev = pts[(i + n - 1) % n];
            let cur = pts[i];
            let next = pts[(i + 1) % n];
            let t = 0.25;
            let p1 = Pos2::new(
                cur.x + (prev.x - cur.x) * t,
                cur.y + (prev.y - cur.y) * t,
            );
            let p2 = Pos2::new(
                cur.x + (next.x - cur.x) * t,
                cur.y + (next.y - cur.y) * t,
            );
            out.push(p1);
            out.push(cur);
            out.push(p2);
        }
        if !fechado {
            out.push(pts[n - 1]);
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
