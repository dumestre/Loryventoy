use eframe::egui::{
    Align2,
    Area,
    Color32,
    CursorIcon,
    FontId,
    Frame,
    Id,
    Key,
    Pos2,
    Rect,
    Sense,
    Stroke,
    StrokeKind,
    TextEdit,
    Ui,
    Vec2,
};

// Quadros por segundo usados na conversão real frame ↔ segundo.
// A régua, os marcadores, o loop e o conteúdo são expressos em SEGUNDOS;
// internamente a timeline trabalha em FRAMES (1 box = 1 frame).
//
// O FPS é configurável pelo nó Canvas do grafo; o valor corrente fica
// neste cell (lido por `fps_atual`), atualizado a cada frame pelo app.
thread_local! {
    pub static TAXA_CELL: std::cell::Cell<f32> = std::cell::Cell::new(30.0);
}

/// Define o fps_atual() corrente (chamado pelo app a partir do nó Canvas).
pub fn definir_fps(v: f32) {
    TAXA_CELL.with(|c| c.set(v));
}

/// Lê o fps_atual() corrente.
pub fn fps_atual() -> f32 {
    TAXA_CELL.with(|c| c.get())
}


#[derive(Clone, Copy, PartialEq, Eq)]
enum LoopDrag {
    Move,
    Start,
    End,
}


#[derive(Clone, Copy)]
struct LoopDragState {
    kind: LoopDrag,
    sec0: f32,
    start0: f32,
    end0: f32,
}


#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkerType {
    Point,
    Range,
}


#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkerColor {
    Red,
    Orange,
    Yellow,
    Lime,
    Green,
    Cyan,
    Teal,
    Blue,
    Purple,
    Pink,
    Brown,
    Coral,
    Indigo,
    Gold,
    Sage,
}

impl MarkerColor {
    fn rgb(self) -> (u8, u8, u8) {
        match self {
            MarkerColor::Red => (220, 80, 80),
            MarkerColor::Orange => (240, 160, 60),
            MarkerColor::Yellow => (230, 200, 90),
            MarkerColor::Lime => (160, 210, 80),
            MarkerColor::Green => (110, 200, 120),
            MarkerColor::Cyan => (90, 200, 220),
            MarkerColor::Teal => (60, 180, 180),
            MarkerColor::Blue => (90, 140, 255),
            MarkerColor::Purple => (180, 120, 230),
            MarkerColor::Pink => (230, 120, 180),
            MarkerColor::Brown => (160, 100, 60),
            MarkerColor::Coral => (240, 140, 120),
            MarkerColor::Indigo => (110, 90, 200),
            MarkerColor::Gold => (210, 180, 50),
            MarkerColor::Sage => (140, 190, 130),
        }
    }

    fn to_color(self) -> Color32 {
        let (r, g, b) = self.rgb();
        Color32::from_rgb(r, g, b)
    }

    fn to_fill(self) -> Color32 {
        let (r, g, b) = self.rgb();
        Color32::from_rgba_unmultiplied(r, g, b, 55)
    }

    const ALL: [MarkerColor; 15] = [
        MarkerColor::Red,
        MarkerColor::Orange,
        MarkerColor::Yellow,
        MarkerColor::Lime,
        MarkerColor::Green,
        MarkerColor::Cyan,
        MarkerColor::Teal,
        MarkerColor::Blue,
        MarkerColor::Purple,
        MarkerColor::Pink,
        MarkerColor::Brown,
        MarkerColor::Coral,
        MarkerColor::Indigo,
        MarkerColor::Gold,
        MarkerColor::Sage,
    ];
}


struct Marker {
    id: u32,
    kind: MarkerType,
    name: String,
    color: MarkerColor,
    start: f32,
    end: f32,
}


#[derive(Clone, Copy, PartialEq, Eq)]
enum MarkerDragKind {
    Move,
    Start,
    End,
}


#[derive(Clone, Copy)]
struct MarkerDrag {
    id: u32,
    kind: MarkerDragKind,
    sec0: f32,
    start0: f32,
    end0: f32,
}


pub struct TimelinePanel {

    pub current_frame: u32,

    // escala da timeline (largura de cada box em pixels = 40 * zoom)
    pub zoom: f32,

    // posição suavizada da agulha (interpola em direção a current_frame)
    display_frame: f32,

    // deslocamento horizontal da viewport, em segundos
    scroll: f32,

    // comprimento total do conteúdo (em segundos), p/ a scrollbar
    pub content_seconds: f32,

    // intervalo de loop, em segundos (só na régua dos segundos)
    pub loop_start: f32,
    pub loop_end: f32,

    // estado do arraste do loop (None = não arrastando o loop)
    loop_drag: Option<LoopDragState>,

    // arrastando a scrollbar?
    scrollbar_drag: bool,

    // arrastando um box de keyframe (muda current_frame)?
    dragging_frame: bool,

    // marcador sendo editado no popup (None = fechado)
    editing_marker: Option<u32>,

    // criação por arrasto: Some(sec_inicial)
    creating_marker: Option<f32>,

    // ID do Point criado por click() no frame atual (para remover no double_click)
    created_by_click: Option<u32>,

    // marcadores (Point e Range)
    markers: Vec<Marker>,
    next_marker_id: u32,
    marker_drag: Option<MarkerDrag>,
    selected: Option<u32>,

    // duração total em frames (vem do nó Canvas)
    pub duracao_frames: u32,

}


impl TimelinePanel {

    pub fn new() -> Self {

        Self {
            current_frame: 0,
            zoom: 1.0,
            display_frame: 0.0,
            scroll: 0.0,
            content_seconds: 60.0,
            loop_start: 0.0,
            loop_end: 5.0,
            loop_drag: None,
            scrollbar_drag: false,
            dragging_frame: false,
            editing_marker: None,
            creating_marker: None,
            created_by_click: None,
            duracao_frames: 150,
            markers: vec![
                Marker {
                    id: 1,
                    kind: MarkerType::Range,
                    name: "intro".into(),
                    color: MarkerColor::Blue,
                    start: 0.0,
                    end: 3.0,
                },
            ],
            next_marker_id: 2,
            marker_drag: None,
            selected: None,
        }
    }


    // Classifica em qual região do loop o ponteiro está (só na régua)
    fn loop_region(
        &self,
        pos: Pos2,
        rect: Rect,
        box_width: f32,
        ruler_bottom: f32,
        loop_active: bool,
    ) -> Option<LoopDrag> {

        // loop inativo ou fora da faixa da régua: não é o loop
        if !loop_active {
            return None;
        }

        if pos.y < rect.top() || pos.y > ruler_bottom {
            return None;
        }

        // mesmos cálculos de x_of() em show()
        let lx0 = rect.left() + (self.loop_start * fps_atual() + 0.5) * box_width
            - self.scroll * box_width;
        let lx1 = rect.left() + (self.loop_end * fps_atual() + 0.5) * box_width
            - self.scroll * box_width;
        let near = 6.0;

        if (pos.x - lx0).abs() <= near {
            Some(LoopDrag::Start)
        } else if (pos.x - lx1).abs() <= near {
            Some(LoopDrag::End)
        } else if pos.x > lx0 && pos.x < lx1 {
            Some(LoopDrag::Move)
        } else {
            None
        }
    }


    // Classifica em qual região de marcador o ponteiro está (faixa de
    // marcadores). Retorna (índice, tipo de arraste).
    fn marker_region(
        &self,
        pos: Pos2,
        rect: Rect,
        box_width: f32,
        ruler_bottom: f32,
        marker_bottom: f32,
    ) -> Option<(usize, MarkerDragKind)> {

        if pos.y < ruler_bottom || pos.y > marker_bottom {
            return None;
        }

        let near = 5.0;

        for (i, m) in self.markers.iter().enumerate() {
            let xs = rect.left() + (m.start * fps_atual() + 0.5) * box_width
                - self.scroll * box_width;

            if m.kind == MarkerType::Range {
                let xe = rect.left() + (m.end * fps_atual() + 0.5) * box_width
                    - self.scroll * box_width;
                if (pos.x - xs).abs() <= near {
                    return Some((i, MarkerDragKind::Start));
                }
                if (pos.x - xe).abs() <= near {
                    return Some((i, MarkerDragKind::End));
                }
                if pos.x > xs && pos.x < xe {
                    return Some((i, MarkerDragKind::Move));
                }
            } else {
                if (pos.x - xs).abs() <= near + 3.0 {
                    return Some((i, MarkerDragKind::Move));
                }
            }
        }
        None
    }


    // Ajusta marcadores adjacentes quando um marcador muda de posição:
    // se dois marcadores se sobrepõem, empurra o seguinte para a direita
    // (ajuste em cascata para manter a ordem).
    fn resolve_overlaps(&mut self) {
        self.markers.sort_by(|a, b| {
            a.start.partial_cmp(&b.start).unwrap()
        });
        for i in 0..self.markers.len().saturating_sub(1) {
            let right = self.markers[i].end;
            let next = &mut self.markers[i + 1];
            if next.start < right {
                next.start = right;
                if next.end < next.start {
                    next.end = next.start;
                }
            }
        }
    }


    pub fn show(
        &mut self,
        ui: &mut Ui,
        loop_active: bool,
        playing: bool,
    ) {

        // (created_by_click NÃO é limpo aqui — persiste entre frames
        //  para o duplo‑clique em dois cliques separados)

        let size =
            Vec2::new(
                ui.available_width(),
                ui.available_height()
            );

        let (rect, response) =
            ui.allocate_exact_size(
                size,
                Sense::click_and_drag()
            );

        let painter = ui.painter().with_clip_rect(rect);


        let ruler_h = 18.0;
        let marker_h = 16.0;
        let scrollbar_h = 14.0;
        let ruler_bottom = rect.top() + ruler_h;
        let marker_bottom = ruler_bottom + marker_h;
        let keyframes_top = marker_bottom;
        let keyframes_bottom = rect.bottom() - scrollbar_h;


        // ---- SCROLL HORIZONTAL e ZOOM (Alt) ----
        // Só atua quando o ponteiro está sobre a própria timeline, para
        // não afetar o canvas (e vice-versa)
        let alt = ui.ctx().input(|i| i.modifiers.alt);
        // Ignora o ponteiro quando ele está sobre uma camada ACIMA da
        // timeline (ex.: popup do ComboBox de resolução do nó Canvas), para
        // não vazar cursor/edição por baixo do conteúdo.
        let pointer = ui
            .ctx()
            .pointer_interact_pos()
            .filter(|p| ui.ctx().layer_id_at(*p) == Some(ui.layer_id()));
        let over = pointer.map_or(false, |p| rect.contains(p));
        let anchor = pointer.unwrap_or_else(|| rect.center());

        let bw0 = (40.0 * self.zoom).max(8.0);

        if over {
            let scroll_delta =
                ui.ctx().input(|i| i.smooth_scroll_delta);
            let hdelta =
                if scroll_delta.x.abs() > scroll_delta.y.abs() {
                    scroll_delta.x
                } else {
                    scroll_delta.y
                };

            if alt && hdelta != 0.0 {
                // Alt + scroll = zoom-to-cursor na timeline
                let factor = (-hdelta * 0.01).exp();
                let new_zoom = (self.zoom * factor).clamp(0.25, 8.0);
                let f_cursor =
                    (anchor.x - rect.left()) / bw0 + self.scroll;
                self.zoom = new_zoom;
                let bw1 = (40.0 * self.zoom).max(8.0);
                self.scroll =
                    (f_cursor - (anchor.x - rect.left()) / bw1).max(0.0);
            } else if hdelta != 0.0 {
                // scroll normal = rolagem horizontal; sentido "arrastar o
                // conteúdo" (arrastar p/ esquerda revela o que está à direita)
                self.scroll = (self.scroll - hdelta / bw0).max(0.0);
            }
        }


        // Atualiza a agulha suavizada (antes dos desenhos)
        self.display_frame +=
            (self.current_frame as f32 - self.display_frame) * 0.25;
        if (self.display_frame - self.current_frame as f32).abs() < 0.01 {
            self.display_frame = self.current_frame as f32;
        } else {
            ui.ctx().request_repaint();
        }


        let box_width = (40.0 * self.zoom).max(8.0);


        // Tecla F: rola a viewport até a agulha (como o canvas centraliza)
        if response.hovered()
            && ui.ctx().input(|i| i.key_pressed(Key::F))
        {
            self.scroll = (self.display_frame + 0.5
                - rect.width() / (2.0 * box_width))
                .max(0.0);
        }


        // Mantém a agulha sempre visível (só durante a reprodução), com
        // margem, salvo enquanto se arrasta a scrollbar. Em pause a tela
        // NÃO rola automaticamente de volta para a agulha.
        if playing && !self.scrollbar_drag {
            let pad = 1.5;
            let vis_min = self.scroll + pad;
            let vis_max =
                self.scroll + rect.width() / box_width - pad;
            if self.display_frame < vis_min {
                self.scroll = (self.display_frame - pad).max(0.0);
            } else if self.display_frame > vis_max {
                self.scroll = (self.display_frame + pad
                    - rect.width() / box_width)
                    .max(0.0);
            }
        }


        // Frames visíveis considerando o scroll
        let first_visible =
            ((self.scroll - 1.5).floor()).max(0.0) as i32;
        let last_visible =
            (self.scroll + rect.width() / box_width + 0.5)
                .ceil() as i32;

        // Atalho para converter um frame em x de tela (com scroll)
        // Captura apenas a cópia local de scroll, não self.
        let scroll = self.scroll;
        let x_of = |f: f32| -> f32 {
            rect.left() + (f + 0.5) * box_width - scroll * box_width
        };


        // ---- ÁREA DE LOOP (somente na régua dos segundos, e só se ativo) ----
        if loop_active && self.loop_end > self.loop_start {
            let lx0 = x_of(self.loop_start * fps_atual());
            let lx1 = x_of(self.loop_end * fps_atual());
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(lx0, rect.top()),
                    Pos2::new(lx1, ruler_bottom),
                ),
                0.0,
                Color32::from_rgba_unmultiplied(90, 140, 255, 35),
            );
        }


        // ---- RÉGUA (segundos reais a partir dos frames) ----
        for f in first_visible..=last_visible {

            let x = x_of(f as f32);

            // marcação maior a cada segundo (múltiplo de fps_atual())
            let is_sec = (f as i32).rem_euclid(fps_atual() as i32) == 0;
            let cor = if is_sec {
                Color32::from_gray(70)
            } else {
                Color32::from_gray(40)
            };
            let y_bot = if is_sec {
                ruler_bottom
            } else {
                ruler_bottom - 6.0
            };

            painter.line_segment(
                [
                    Pos2::new(x, rect.top()),
                    Pos2::new(x, y_bot),
                ],
                Stroke::new(1.0, cor),
            );

            // número do segundo (conversão real frame -> segundo)
            if is_sec {
                let s = (f as f32 / fps_atual()) as u32;
                painter.text(
                    Pos2::new(x, rect.top() + ruler_h / 2.0),
                    Align2::CENTER_CENTER,
                    format!("{}s", s),
                    FontId::proportional(11.0),
                    Color32::from_gray(150),
                );
            }
        }

        // bordas do loop sobre a régua dos segundos
        if loop_active && self.loop_end > self.loop_start {
            for &lx in &[x_of(self.loop_start * fps_atual()), x_of(self.loop_end * fps_atual())] {
                painter.line_segment(
                    [
                        Pos2::new(lx, rect.top()),
                        Pos2::new(lx, ruler_bottom),
                    ],
                    Stroke::new(1.5, Color32::from_rgb(90, 140, 255)),
                );
            }
        }

        // separador régua / marcadores
        painter.line_segment(
            [
                Pos2::new(rect.left(), ruler_bottom),
                Pos2::new(rect.right(), ruler_bottom),
            ],
            Stroke::new(1.0, Color32::from_gray(60)),
        );


        // ---- FAIXA DE MARCADORES (Point e Range) ----
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.left(), ruler_bottom),
                Pos2::new(rect.right(), marker_bottom),
            ),
            0.0,
            Color32::from_rgb(28, 28, 38),
        );
        painter.text(
            Pos2::new(rect.left() + 6.0, (ruler_bottom + marker_bottom) / 2.0),
            Align2::LEFT_CENTER,
            "marcadores (duplo-clique)",
            FontId::proportional(9.0),
            Color32::from_gray(100),
        );

        for m in &self.markers {
            let xs = x_of(m.start * fps_atual());
            let c = m.color.to_color();
            let selected = self
                .selected
                .map_or(false, |id| id == m.id);

            if m.kind == MarkerType::Range {
                let xe = x_of(m.end * fps_atual());
                let r = Rect::from_min_max(
                    Pos2::new(xs, ruler_bottom + 1.0),
                    Pos2::new(xe, marker_bottom - 1.0),
                );
                painter.rect_filled(r, 3.0, m.color.to_fill());
                painter.rect_stroke(
                    r,
                    3.0,
                    Stroke::new(
                        if selected { 2.0 } else { 1.0 },
                        c,
                    ),
                    StrokeKind::Inside,
                );
                painter.text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    &m.name,
                    FontId::proportional(10.0),
                    c,
                );
            } else {
                // Point: marcador vertical + nome
                painter.line_segment(
                    [
                        Pos2::new(xs, ruler_bottom),
                        Pos2::new(xs, marker_bottom),
                    ],
                    Stroke::new(if selected { 3.0 } else { 2.0 }, c),
                );
                painter.text(
                    Pos2::new(xs, (ruler_bottom + marker_bottom) / 2.0),
                    Align2::CENTER_CENTER,
                    &m.name,
                    FontId::proportional(10.0),
                    c,
                );
            }
        }

        // Preview de criação por arrasto
        if let Some(start_sec) = self.creating_marker {
            if let Some(p) = pointer {
                let snap = |s: f32| (s * 2.0).round() / 2.0;
                let end_sec = snap(
                    ((p.x - rect.left()) / box_width + self.scroll - 0.5)
                        / fps_atual(),
                );
                if (end_sec - start_sec).abs() >= 0.5 {
                    let (s, e) = if start_sec < end_sec {
                        (start_sec, end_sec)
                    } else {
                        (end_sec, start_sec)
                    };
                    let xs = x_of(s * fps_atual());
                    let xe = x_of(e * fps_atual());
                    painter.rect_filled(
                        Rect::from_min_max(
                            Pos2::new(xs, ruler_bottom + 1.0),
                            Pos2::new(xe, marker_bottom - 1.0),
                        ),
                        3.0,
                        Color32::from_rgba_unmultiplied(
                            90, 140, 255, 80,
                        ),
                    );
                    painter.rect_stroke(
                        Rect::from_min_max(
                            Pos2::new(xs, ruler_bottom + 1.0),
                            Pos2::new(xe, marker_bottom - 1.0),
                        ),
                        3.0,
                        Stroke::new(1.0, Color32::from_rgb(90, 140, 255)),
                        StrokeKind::Inside,
                    );
                }
            }
        }

        // separador marcadores / keyframes
        painter.line_segment(
            [
                Pos2::new(rect.left(), marker_bottom),
                Pos2::new(rect.right(), marker_bottom),
            ],
            Stroke::new(1.0, Color32::from_gray(60)),
        );


        // ---- BOXES (keyframes) ----
        let box_fill_light = Color32::from_rgb(34, 34, 44);
        let box_fill_dark = Color32::from_rgb(24, 24, 31);
        for f in first_visible..=last_visible {

            let x = x_of(f as f32);

            let box_rect = Rect::from_min_max(
                Pos2::new(x - box_width / 2.0, keyframes_top),
                Pos2::new(x + box_width / 2.0, keyframes_bottom),
            );

            // Faixas de 10 frames: cor clara e escura intercaladas
            let band = (f as i32).div_euclid(10);
            let fill = if band.rem_euclid(2) == 0 {
                box_fill_light
            } else {
                box_fill_dark
            };
            painter.rect_filled(box_rect, 0.0, fill);

            painter.rect_stroke(
                box_rect,
                0.0,
                Stroke::new(1.0, Color32::from_gray(45)),
                StrokeKind::Inside,
            );

            // número do frame sempre no centro do box
            painter.text(
                box_rect.center(),
                Align2::CENTER_CENTER,
                format!("{}", f),
                FontId::proportional(12.0),
                Color32::from_gray(200),
            );
        }


        // ---- AGULHA (bolinha verde correndo sobre os boxes) ----
        let needle_x = x_of(self.display_frame);
        let needle_y = (keyframes_top + keyframes_bottom) / 2.0;

        // guia vertical do frame atual (só na faixa de keyframes)
        painter.line_segment(
            [
                Pos2::new(needle_x, keyframes_top),
                Pos2::new(needle_x, keyframes_bottom),
            ],
            Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(80, 220, 120, 70),
            ),
        );

        painter.circle_filled(
            Pos2::new(needle_x, needle_y),
            7.0,
            Color32::from_rgb(80, 220, 120),
        );


        // ---- SCROLLBAR HORIZONTAL (base da timeline) ----
        let sb_top = rect.bottom() - scrollbar_h;
        let scrollbar_rect = Rect::from_min_max(
            Pos2::new(rect.left(), sb_top),
            Pos2::new(rect.right(), rect.bottom()),
        );
        let vis = rect.width() / box_width;
        let content_frames = self.content_seconds * fps_atual();
        let thumb_w = (rect.width() * (vis / content_frames)
            .clamp(0.05, 1.0))
            .max(20.0);
        let max_scroll = (content_frames - vis).max(0.0);
        let sb_frac = if max_scroll > 0.0 {
            (self.scroll / max_scroll).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let thumb_x =
            rect.left() + sb_frac * (rect.width() - thumb_w);

        // trilho
        painter.rect_filled(
            scrollbar_rect,
            0.0,
            Color32::from_rgb(30, 30, 38),
        );
        // thumb
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(thumb_x, sb_top + 3.0),
                Pos2::new(thumb_x + thumb_w, rect.bottom() - 3.0),
            ),
            7.0,
            Color32::from_gray(120),
        );


        // ---- CURSOR sobre a timeline ----
        let cursor = if let Some(state) = self.loop_drag {
            match state.kind {
                LoopDrag::Start | LoopDrag::End => {
                    Some(CursorIcon::ResizeHorizontal)
                }
                LoopDrag::Move => Some(CursorIcon::Grab),
            }
        } else if let Some(pos) = pointer {
            if scrollbar_rect.contains(pos) {
                Some(CursorIcon::ResizeHorizontal)
            } else if pos.y >= ruler_bottom && pos.y <= marker_bottom {
                // faixa de marcadores
                if let Some((_, kind)) = self.marker_region(
                    pos, rect, box_width, ruler_bottom, marker_bottom,
                ) {
                    match kind {
                        MarkerDragKind::Start | MarkerDragKind::End => {
                            Some(CursorIcon::ResizeHorizontal)
                        }
                        MarkerDragKind::Move => Some(CursorIcon::Grab),
                    }
                } else {
                    // vazio → esconde cursor e desenha o lapis.svg
                    Some(CursorIcon::None)
                }
            } else if rect.contains(pos) {
                match self.loop_region(
                    pos, rect, box_width, ruler_bottom, loop_active,
                ) {
                    Some(LoopDrag::Start) | Some(LoopDrag::End) => {
                        Some(CursorIcon::ResizeHorizontal)
                    }
                    Some(LoopDrag::Move) => Some(CursorIcon::Grab),
                    None => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(icon) = cursor {
            ui.ctx().set_cursor_icon(icon);
        }

        // Desenha o lapis.svg no lugar do cursor quando está sobre a
        // área vazia dos marcadores (usando paint_at para não interferir
        // no clique — sem Area)
        if let Some(pos) = pointer {
            if pos.y >= ruler_bottom
                && pos.y <= marker_bottom
                && rect.contains(pos)
                && self.marker_region(
                    pos, rect, box_width, ruler_bottom, marker_bottom,
                ).is_none()
            {
                let sz = Vec2::splat(18.0);
                // ponta do lápis (canto inferior‑esquerdo do SVG, 0,18) exatamente no cursor
                let r = Rect::from_min_max(
                    pos - Vec2::new(0.0, sz.y),
                    pos + Vec2::new(sz.x, 0.0),
                );
                egui::Image::new(egui::include_image!("icons/lapis.svg"))
                    .paint_at(ui, r);
            }
        }


        // Clicar em um box de keyframe leva a agulha para ele
        if response.clicked() {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                if pos.y >= keyframes_top
                    && pos.y <= keyframes_bottom
                {
                    let f = ((pos.x - rect.left()) / box_width
                        + self.scroll - 0.5)
                        .round()
                        .max(0.0) as u32;
                    self.current_frame = f;
                    ui.ctx().request_repaint();
                }
            }
        }

        // Clique na faixa de marcadores:
        //   - sobre marcador existente → seleciona
        //   - em área vazia → cria um marcador Point
        if response.clicked() {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                if pos.y >= ruler_bottom
                    && pos.y <= marker_bottom
                {
                    if let Some((i, _)) = self.marker_region(
                        pos,
                        rect,
                        box_width,
                        ruler_bottom,
                        marker_bottom,
                    ) {
                        self.selected = Some(self.markers[i].id);
                        self.created_by_click = None;
                    } else if !response.double_clicked() {
                        // clique simples sem arrasto → Point
                        // (double_clicked() evita criar Point duplicado
                        //  na segunda chamada — o first click ainda
                        //  cria Point, mas é aceitável)
                        let snap = |s: f32| (s * 2.0).round() / 2.0;
                        let sec = snap(
                            (((pos.x - rect.left()) / box_width
                                + self.scroll - 0.5)
                                .max(0.0))
                                / fps_atual()
                        );
                        let m = Marker {
                            id: self.next_marker_id,
                            kind: MarkerType::Point,
                            name: "Ponto".into(),
                            color: MarkerColor::Yellow,
                            start: sec,
                            end: sec,
                        };
                        let mid = m.id;
                        self.next_marker_id += 1;
                        self.markers.push(m);
                        self.selected = Some(mid);
                        self.created_by_click = Some(mid);
                    }
                }
            }
        }

        // Duplo-clique na faixa de marcadores:
        //   1) se o clique anterior criou um Point → estende‑o até o
        //      duplo‑clique (vira Range)
        //   2) sobre um marcador existente → abre popup de edição
        //   3) em área vazia → cria Range centrado
        if response.double_clicked() {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                if pos.y >= ruler_bottom
                    && pos.y <= marker_bottom
                {
                    // 1) Extende o Point se ele foi criado pelo click()
                    //    anterior e a posição do duplo‑clique é diferente
                    let extend = self.created_by_click.and_then(|id| {
                        self.markers.iter().find(|m| m.id == id).and_then(|m| {
                            if m.kind == MarkerType::Point {
                                Some((id, m.start))
                            } else {
                                None
                            }
                        })
                    });
                    if let Some((id, start_sec)) = extend {
                        let snap = |s: f32| (s * 2.0).round() / 2.0;
                        let sec = snap(
                            (((pos.x - rect.left()) / box_width
                                + self.scroll - 0.5)
                                .max(0.0))
                                / fps_atual()
                        );
                        if (sec - start_sec).abs() >= 0.5 {
                            if let Some(m) = self.markers
                                .iter_mut()
                                .find(|m| m.id == id)
                            {
                                let s = m.start.min(sec);
                                let e = m.start.max(sec);
                                m.start = s;
                                m.end = e.max(s + 0.5);
                                m.kind = MarkerType::Range;
                                m.name = "Marcador".into();
                                m.color = MarkerColor::Blue;
                                self.selected = Some(m.id);
                                self.created_by_click = None;
                                return;
                            }
                        }
                    }

                    // 2) marcador existente → popup
                    if let Some((i, _)) = self.marker_region(
                        pos,
                        rect,
                        box_width,
                        ruler_bottom,
                        marker_bottom,
                    ) {
                        let id = self.markers[i].id;
                        self.selected = Some(id);
                        self.editing_marker = Some(id);
                        self.created_by_click = None;
                    } else {
                        // 3) vazio → Range centrado
                        let snap = |s: f32| (s * 2.0).round() / 2.0;
                        let sec = snap(
                            (((pos.x - rect.left()) / box_width
                                + self.scroll - 0.5)
                                .max(0.0))
                                / fps_atual()
                        );
                        let m = Marker {
                            id: self.next_marker_id,
                            kind: MarkerType::Range,
                            name: "Marcador".into(),
                            color: MarkerColor::Blue,
                            start: sec,
                            end: (sec + 2.0).max(1.0),
                        };
                        let mid = m.id;
                        self.next_marker_id += 1;
                        self.markers.push(m);
                        self.selected = Some(mid);
                    }
                }
            }
        }


        // ---- INTERAÇÃO ----
        if response.drag_started() {
            if let Some(origin) =
                ui.ctx().input(|i| i.pointer.press_origin())
            {
                if scrollbar_rect.contains(origin) {
                    self.scrollbar_drag = true;
                    self.created_by_click = None;
                } else if let Some((i, kind)) = self.marker_region(
                    origin,
                    rect,
                    box_width,
                    ruler_bottom,
                    marker_bottom,
                ) {
                    self.created_by_click = None;
                    let m = &self.markers[i];
                    self.marker_drag = Some(MarkerDrag {
                        id: m.id,
                        kind,
                        sec0: ((origin.x - rect.left()) / box_width
                            + self.scroll)
                            / fps_atual(),
                        start0: m.start,
                        end0: m.end,
                    });
                    self.selected = Some(m.id);
                } else if origin.y >= ruler_bottom
                    && origin.y <= marker_bottom
                {
                    self.created_by_click = None;
                    // Arrasto em área vazia da faixa de marcadores:
                    // inicia a criação de um novo Range
                    let snap = |s: f32| (s * 2.0).round() / 2.0;
                    let start_sec = snap(
                        (((origin.x - rect.left()) / box_width
                            + self.scroll
                            - 0.5)
                            / fps_atual())
                            .max(0.0)
                    );
                    self.creating_marker = Some(start_sec);
                } else if origin.y >= keyframes_top
                    && origin.y <= keyframes_bottom
                {
                    self.dragging_frame = true;
                } else if self.loop_drag.is_none() {
                    if let Some(kind) = self.loop_region(
                        origin,
                        rect,
                        box_width,
                        ruler_bottom,
                        loop_active,
                    ) {
                        self.loop_drag = Some(LoopDragState {
                            kind,
                            sec0: ((origin.x - rect.left()) / box_width
                                + self.scroll)
                                / fps_atual(),
                            start0: self.loop_start,
                            end0: self.loop_end,
                        });
                    }
                }
            }
        }

        if response.dragged() {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {

                if self.scrollbar_drag {
                    // Arrasta a scrollbar: move a viewport
                    let track_w = (rect.width() - thumb_w).max(1.0);
                    let frac = ((pos.x - rect.left() - thumb_w / 2.0)
                        / track_w)
                        .clamp(0.0, 1.0);
                    self.scroll = frac * max_scroll;
                } else if let Some(drag) = self.marker_drag {
                    // Arrasta marcador
                    if let Some(m) = self.markers
                        .iter_mut()
                        .find(|m| m.id == drag.id)
                    {
                        let sec = ((pos.x - rect.left()) / box_width
                            + self.scroll)
                            / fps_atual();
                        let delta = sec - drag.sec0;
                        let min_w = 1.0;

                        let snap = |s: f32| (s * 2.0).round() / 2.0;
                        match drag.kind {
                            MarkerDragKind::Move => {
                                let s = (drag.start0 + delta).max(0.0);
                                let e = (drag.end0 + delta).max(0.0);
                                m.start = snap(s);
                                m.end = snap(e.max(s + min_w));
                            }
                            MarkerDragKind::Start => {
                                m.start = snap(
                                    (drag.start0 + delta)
                                        .clamp(0.0, m.end - min_w)
                                );
                            }
                            MarkerDragKind::End => {
                                m.end = snap(
                                    (drag.end0 + delta)
                                        .max(m.start + min_w)
                                );
                            }
                        }
                        self.resolve_overlaps();
                        // re‑seleciona depois da ordenação
                        self.selected = Some(drag.id);
                    }
                } else if self.creating_marker.is_some() {
                    // Criação por arrasto: apenas move a viewport para
                    // acompanhar, o marcador será criado no drag_stopped
                    // (não precisa fazer nada aqui)
                } else if self.dragging_frame {
                    // Arrastar nos frames: muda o frame atual
                    let f = ((pos.x - rect.left()) / box_width
                        + self.scroll - 0.5)
                        .round()
                        .max(0.0) as u32;
                    self.current_frame = f;
                } else {
                    // Manipulando o loop (move ou redimensiona as bordas)
                    let drag = self.loop_drag
                        .map(|s| (s.kind, s.sec0, s.start0, s.end0));
                    if let Some((kind, sec0, start0, end0)) = drag {
                        let sec = ((pos.x - rect.left()) / box_width
                            + self.scroll)
                            / fps_atual();
                        let delta = sec - sec0;
                        let min_w = 1.0;

                        match kind {
                            LoopDrag::Move => {
                                let start = (start0 + delta).max(0.0);
                                self.loop_start = start.round();
                                self.loop_end = (end0 + delta).round();
                            }
                            LoopDrag::Start => {
                                self.loop_start = (start0 + delta)
                                    .clamp(0.0, end0 - min_w)
                                    .round();
                            }
                            LoopDrag::End => {
                                self.loop_end = (end0 + delta)
                                    .max(start0 + min_w)
                                    .round();
                            }
                        }
                    }
                }
            }
        }

        if response.drag_stopped() {
            // Finaliza criação de marcador por arrasto
            if let Some(start) = self.creating_marker {
                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    let snap = |s: f32| (s * 2.0).round() / 2.0;
                    let end = snap(
                        (((pos.x - rect.left()) / box_width + self.scroll)
                            / fps_atual())
                            .max(0.0)
                    );
                    if (end - start).abs() >= 0.5 {
                        let (s, e) = if start < end {
                            (start, end)
                        } else {
                            (end, start)
                        };
                        let m = Marker {
                            id: self.next_marker_id,
                            kind: MarkerType::Range,
                            name: "Marcador".into(),
                            color: MarkerColor::Blue,
                            start: s,
                            end: e.max(s + 0.5),
                        };
                        let mid = m.id;
                        self.next_marker_id += 1;
                        self.markers.push(m);
                        self.selected = Some(mid);
                        self.resolve_overlaps();
                    }
                }
                self.creating_marker = None;
            }
            self.loop_drag = None;
            self.scrollbar_drag = false;
            self.marker_drag = None;
            self.dragging_frame = false;
        }


        // ---- TECLADO: Delete / Backspace apaga marcador selecionado ----
        let over_timeline = pointer.map_or(false, |p| rect.contains(p));
        if (over_timeline || self.editing_marker.is_some())
            && (ui.ctx().input(|i| i.key_pressed(Key::Delete))
                || ui.ctx().input(|i| i.key_pressed(Key::Backspace)))
        {
            if let Some(id) = self.selected {
                self.markers.retain(|m| m.id != id);
                self.selected = None;
                self.editing_marker = None;
            }
        }


        // ---- POPUP DE EDIÇÃO DO MARCADOR (renomear + cor) ----
        if let Some(edit_id) = self.editing_marker {
            // posiciona o popup logo abaixo do marcador
            // (usa cálculo direto para não capturar self.scroll via x_of)
            let popup_pos = if let Some(m) = self.markers
                .iter()
                .find(|m| m.id == edit_id)
            {
                let mid = (m.start + m.end) / 2.0;
                let cx = rect.left() + (mid + 0.5) * box_width
                    - self.scroll * box_width;
                Pos2::new(cx, marker_bottom + 4.0)
            } else {
                // marcador foi deletado enquanto o popup estava aberto
                self.editing_marker = None;
                Pos2::new(rect.left() + 100.0, marker_bottom + 4.0)
            };

            let popup_id = Id::new("marker_edit_popup");
            let popup_open = Area::new(popup_id)
                .fixed_pos(popup_pos)
                .show(ui.ctx(), |ui| {

                Frame::popup(ui.style())
                    .corner_radius(8)
                    .show(ui, |ui| {
                    ui.set_min_width(160.0);

                    if let Some(m) = self.markers
                        .iter_mut()
                        .find(|m| m.id == edit_id)
                    {
                        // rename
                        ui.horizontal(|ui| {
                            ui.label("Nome:");
                            ui.add(
                                TextEdit::singleline(&mut m.name)
                                    .desired_width(100.0)
                            );
                        });

                        ui.separator();

                        // cor: bolinhas preenchidas (6 por linha)
                        ui.label("Cor:");
                        let cw = 20.0;    // diâmetro da bolinha
                        let gap = 4.0;
                        // max_width para 6 colunas
                        ui.set_max_width(gap + (cw + gap) * 6.0);
                        ui.horizontal_wrapped(|ui| {
                            for (_ic, &c) in MarkerColor::ALL.iter().enumerate() {
                                let fill = c.to_color();
                                let sel = m.color == c;
                                let s = if sel {
                                    Stroke::new(2.5, Color32::WHITE)
                                } else {
                                    Stroke::new(1.0, Color32::from_gray(80))
                                };
                                let rect = Frame::NONE
                                    .fill(fill)
                                    .corner_radius(10)
                                    .stroke(s)
                                    .show(ui, |ui| {
                                        ui.set_min_size(Vec2::splat(cw));
                                    })
                                    .response
                                    .rect;
                                if ui.interact(rect, ui.next_auto_id(), Sense::click()).clicked() {
                                    m.color = c;
                                }
                            }
                        });

                        ui.separator();
                        if ui.button("Fechar").clicked() {
                            self.editing_marker = None;
                        }
                    } else {
                        ui.label("(removido)");
                        self.editing_marker = None;
                    }
                });
            });

            // fecha popup se clicar fora dele
            // (ignora clique que abriu o popup via double_click neste frame)
            if !response.double_clicked()
                && ui.ctx().input(|i| i.pointer.any_click())
                && !popup_open.response.rect.contains(
                    ui.ctx().pointer_interact_pos()
                        .unwrap_or(Pos2::ZERO)
                )
            {
                self.editing_marker = None;
            }
            if ui.ctx().input(|i| i.key_pressed(Key::Escape)) {
                self.editing_marker = None;
            }
        }


        // (foco não é requisitado para evitar que espaço ative click())

    }

}
