#![allow(dead_code)]

use std::collections::HashMap;
use std::string::String;
use std::sync::{OnceLock, RwLock};

use crate::domain::Color as DomainColor;
use crate::graph_editor::NodeId;
use crate::nodes::{NodeParams, TipoNo};
use eframe::egui::{
    Color32, ComboBox, DragValue, Grid, Image, Response, Sense, Stroke,
    Ui, Vec2,
};

mod animation;
mod canvas;
mod layer;
mod noise;
mod pen;
mod scene;
mod shape;
mod text;
mod transform;

/// Ações que podem ser solicitadas pelo inspector de um nó.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum AcaoInspector {
    #[default]
    Nenhuma,
    FocarCena(NodeId),
    CriarLayerEntry,
    RemoverLayerEntry(NodeId, usize),
    SubirLayerEntry(NodeId, usize),
    DescerLayerEntry(NodeId, usize),
    SelecionarLayer(NodeId, usize),
    ToggleVisivelLayer(NodeId, usize),
    RenomearLayerEntry(NodeId, usize),
}

/// Margens internas do corpo do card e altura do cabeçalho, em unidades de
/// CANVAS (zoom 1). O nó inteiro escala com o zoom (estilo Blender): tudo
/// — card, cabeçalho, margens e conteúdo — cresce/encolhe junto.
pub const MARGEM_X: f32 = 8.0;
pub const MARGEM_Y: f32 = 4.0;
pub const CABECALHO_H: f32 = 16.0;
/// Fonte base do nome do nó (unidades de canvas, escala com o zoom).
pub const FONTE_TITULO: f32 = 11.0;

/// Meia-extensão mínima do card (canvas), para tipos ainda não medidos.
fn fallback_size(tipo: TipoNo) -> Vec2 {
    match tipo {
        TipoNo::Saida => Vec2::new(80.0, 55.0),
        TipoNo::Transform => Vec2::new(100.0, 62.0),
        TipoNo::Canvas => Vec2::new(105.0, 72.0),
        TipoNo::Cena => Vec2::new(110.0, 70.0),
        TipoNo::Layer => Vec2::new(130.0, 75.0),
        TipoNo::Shape => Vec2::new(120.0, 80.0),
        TipoNo::Texto => Vec2::new(150.0, 90.0),
        TipoNo::Pen => Vec2::new(150.0, 120.0),
        TipoNo::Ruido => Vec2::new(110.0, 70.0),
        TipoNo::Anim => Vec2::new(170.0, 130.0),
    }
}

/// Cache das medidas de conteúdo por tipo de nó (meia-extensão do card,
/// em unidades de canvas). Preenchido a cada frame ao desenhar o conteúdo.
fn medidas() -> &'static RwLock<HashMap<TipoNo, Vec2>> {
    static M: OnceLock<RwLock<HashMap<TipoNo, Vec2>>> = OnceLock::new();
    M.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Cache do Y central de cada ROW de parâmetro (por nome), relativo ao TOPO
/// do corpo do card, em unidades de canvas. Usado para alinhar a "bolinha"
/// (porto) de um parâmetro exatamente na altura da sua row no inspector.
fn linhas_y() -> &'static RwLock<HashMap<TipoNo, HashMap<String, f32>>> {
    static L: OnceLock<RwLock<HashMap<TipoNo, HashMap<String, f32>>>> = OnceLock::new();
    L.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Contexto de captura das posições Y das rows durante `show_content`.
/// `topo` é o Y (tela) do início do corpo; `zoom` converte tela→canvas.
struct CapturaRows {
    tipo: TipoNo,
    topo_tela: f32,
    zoom: f32,
    ys: HashMap<String, f32>,
}

thread_local! {
    static CAPTURA: std::cell::RefCell<Option<CapturaRows>> =
        const { std::cell::RefCell::new(None) };
}

/// Registra o Y central (em canvas, relativo ao topo do body) de uma row com
/// o `nome` de parâmetro dado, a partir do `Response` da row desenhada.
pub(crate) fn registrar_linha(nome: &str, resp: &Response) {
    CAPTURA.with(|c| {
        if let Some(cap) = c.borrow_mut().as_mut() {
            let centro_tela = resp.rect.center().y;
            let y_canvas = (centro_tela - cap.topo_tela) / cap.zoom.max(0.01);
            cap.ys.insert(nome.to_string(), y_canvas);
        }
    });
}

/// Y central (canvas, relativo ao topo do corpo) da row de parâmetro `nome`
/// para o `tipo`, se já foi medida neste/último frame.
pub fn linha_y(tipo: TipoNo, nome: &str) -> Option<f32> {
    linhas_y()
        .read()
        .unwrap()
        .get(&tipo)
        .and_then(|m| m.get(nome))
        .copied()
}

/// Registra o tamanho REAL do conteúdo (em pixels de tela, já escalado pelo
/// zoom) medido ao desenhar a `Area`. Convertemos de volta para unidades de
/// canvas (dividindo pelo zoom) e somamos as margens/cabeçalho em canvas,
/// para o card escalar junto com o resto.
/// Atualiza largura E altura com base no conteúdo real.
pub fn registrar_medida(tipo: TipoNo, conteudo_tela: Vec2, zoom: f32) {
    let z = zoom.max(0.01);
    let cw = conteudo_tela.x / z;
    let ch = conteudo_tela.y / z;
    let w = (cw + MARGEM_X * 2.0) / 2.0;
    let h = (ch + CABECALHO_H + MARGEM_Y * 2.0) / 2.0;
    if w.is_finite() && w > 1.0 && h.is_finite() && h > 1.0 {
        let fb = fallback_size(tipo);
        let mut m = medidas().write().unwrap();
        m.insert(tipo, Vec2::new(w.max(fb.x), h.max(fb.y)));
    }
}

/// Escala o estilo do `Ui` pelo zoom, para que os widgets do conteúdo do nó
/// (fontes, espaçamentos, alturas) cresçam/encolham junto com o card.
pub fn escalar_estilo(ui: &mut Ui, zoom: f32) {
    let z = zoom.clamp(0.05, 16.0);
    let style = ui.style_mut();
    for (_ts, font_id) in style.text_styles.iter_mut() {
        font_id.size *= z;
    }
    style.spacing.item_spacing *= z;
    style.spacing.button_padding *= z;
    style.spacing.interact_size *= z;
    style.spacing.icon_width *= z;
    style.spacing.icon_width_inner *= z;
    style.spacing.combo_width *= z;
    style.spacing.indent *= z;
}

/// Presets de resolução prontos (rótulo curto, largura, altura).
pub const PRESETS_RESOLUCAO: &[(&str, u32, u32)] = &[
    ("4K UHD", 3840, 2160),
    ("QHD", 2560, 1440),
    ("Full HD", 1920, 1080),
    ("HD", 1280, 720),
    ("SD", 854, 480),
    ("Quadrado", 1080, 1080),
    ("Vertical FHD", 1080, 1920),
    ("Vertical HD", 720, 1280),
    ("Cinema 2K", 2048, 1080),
    ("Instagram", 1080, 1350),
];

/// String hexadecimal "#RRGGBB" de uma cor.
pub(crate) fn hex_de(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

pub(crate) fn cor_egui(cor: DomainColor) -> Color32 {
    Color32::from_rgba_unmultiplied(cor.r, cor.g, cor.b, cor.a)
}

pub(crate) fn editar_cor(ui: &mut Ui, cor: &mut DomainColor) {
    let mut visual = cor_egui(*cor);
    if ui.color_edit_button_srgba(&mut visual).changed() {
        *cor = DomainColor::from_rgba(visual.r(), visual.g(), visual.b(), visual.a());
    }
    ui.label(hex_de(visual));
}

/// Meia-extensão (w/2, h/2) do cartão do nó em coordenadas de canvas.
pub fn content_size(tipo: TipoNo) -> Vec2 {
    let fb = fallback_size(tipo);
    let measured = medidas().read().unwrap().get(&tipo).copied();
    Vec2::new(
        measured.map_or(fb.x, |m| m.x.max(fb.x)),
        measured.map_or(fb.y, |m| m.y.max(fb.y)),
    )
}

/// Image botão sem hover/active fill do egui.
pub(crate) fn icon_button(ui: &mut Ui, image: Image<'_>, size: f32) -> Response {
    let mut v = ui.visuals().clone();
    v.widgets.hovered.bg_fill = Color32::TRANSPARENT;
    v.widgets.active.bg_fill = Color32::TRANSPARENT;
    v.widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.active.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.widgets.active.bg_stroke = Stroke::NONE;
    ui.scope(|ui| {
        *ui.visuals_mut() = v;
        ui.add(
            image
                .fit_to_exact_size(Vec2::splat(size))
                .sense(Sense::click()),
        )
    })
    .inner
}

pub(crate) fn hover_bg(ui: &Ui, resp: &Response, color: Color32) {
    if resp.hovered() {
        let r = resp.rect;
        ui.painter().rect_filled(r.expand(1.0), 2.0, color);
    }
}

pub(crate) const HOVER_VERDE: Color32 = Color32::from_rgba_premultiplied(16, 31, 19, 40);
pub(crate) const HOVER_VERMELHO: Color32 = Color32::from_rgba_premultiplied(39, 16, 16, 50);

/// Renderiza o cabeçalho do Layer (combo de cena + separador + botão add).
pub use layer::render_layer_header;

/// Renderiza uma única row de layer (cor, toggle, nome, delete, setas).
pub use layer::render_layer_row;

/// Conteúdo do nó (abaixo do cabeçalho), em layout de inspector.
pub fn show_content(
    ui: &mut Ui,
    tipo: TipoNo,
    params: Option<&mut NodeParams>,
    cenas: &[(String, NodeId)],
    node_id: NodeId,
    renaming_layer: &mut Option<(NodeId, usize)>,
    topo_tela: f32,
    zoom: f32,
) -> AcaoInspector {
    let Some(params) = params else {
        return AcaoInspector::Nenhuma;
    };
    CAPTURA.with(|c| {
        *c.borrow_mut() = Some(CapturaRows {
            tipo,
            topo_tela,
            zoom,
            ys: HashMap::new(),
        });
    });
    let mut acao = AcaoInspector::Nenhuma;
    ui.vertical(|ui| match tipo {
        TipoNo::Canvas => {
            if let NodeParams::Canvas(cfg) = params {
                canvas::show(ui, cfg);
            }
        }
        TipoNo::Transform => {
            if let NodeParams::Transform(t) = params {
                transform::show_transform(ui, t);
            }
        }
        TipoNo::Cena => {
            if let NodeParams::Cena(cena) = params {
                let r = scene::show(ui, cena, cenas);
                if r != AcaoInspector::Nenhuma {
                    acao = r;
                }
            }
        }
        TipoNo::Layer => {
            if let NodeParams::Layer(layer) = params {
                let r = layer::show(ui, layer, cenas, node_id, renaming_layer);
                if r != AcaoInspector::Nenhuma {
                    acao = r;
                }
            }
        }
        TipoNo::Shape => {
            if let NodeParams::Shape(shape) = params {
                shape::show(ui, shape, cenas);
            }
        }
        TipoNo::Texto => {
            if let NodeParams::Texto(texto) = params {
                text::show(ui, texto, cenas);
            }
        }
        TipoNo::Pen => {
            if let NodeParams::Pen(pen) = params {
                pen::show(ui, pen, cenas);
            }
        }
        TipoNo::Ruido => {
            if let NodeParams::Ruido(ruido) = params {
                noise::show(ui, ruido);
            }
        }
        TipoNo::Anim => {
            if let NodeParams::Anim(anim) = params {
                animation::show(ui, anim);
            }
        }
        TipoNo::Saida => {
            if let NodeParams::Saida(saida) = params {
                transform::show_output(ui, saida);
            }
        }
    });
    CAPTURA.with(|c| {
        if let Some(cap) = c.borrow_mut().take() {
            linhas_y().write().unwrap().insert(cap.tipo, cap.ys);
        }
    });
    acao
}

// ── Grid helpers (shared across submodules) ──────────────────────

pub(crate) fn grid_texto(ui: &mut Ui, label: &str, v: &mut String) {
    let r = Grid::new(label)
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label(label);
            ui.text_edit_singleline(v);
            ui.end_row();
        });
    registrar_linha(label, &r.response);
}

pub(crate) fn grid_combo_cena(ui: &mut Ui, label: &str, cena: &mut String, cenas: &[(String, NodeId)]) {
    let r = Grid::new(("cena", label))
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label(label);
            let atual = if cena.is_empty() {
                cenas.first().map(|(n, _)| n.clone()).unwrap_or_default()
            } else {
                cena.clone()
            };
            ComboBox::from_id_salt(("cena_cb", label))
                .selected_text(&atual)
                .show_ui(ui, |ui| {
                    for (c, _) in cenas {
                        if ui.selectable_label(*c == atual, c.clone()).clicked() {
                            *cena = c.clone();
                        }
                    }
                    if cenas.is_empty() {
                        ui.label("(crie um nó Cena)");
                    }
                });
            ui.end_row();
        });
    registrar_linha(label, &r.response);
}

pub(crate) fn grid_combo_tipo(ui: &mut Ui, label: &str, tipo: &mut u8) {
    let nomes = [
        "Retângulo",
        "Elipse",
        "Triângulo",
        "Estrela",
        "Losango",
        "Polígono",
        "Seta",
    ];
    let atual = nomes.get(*tipo as usize).copied().unwrap_or(nomes[0]);
    let r = Grid::new(("tipo", label))
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label(label);
            ComboBox::from_id_salt(("tipo_cb", label))
                .selected_text(atual)
                .show_ui(ui, |ui| {
                    for (i, n) in nomes.iter().enumerate() {
                        if ui.selectable_label(*tipo as usize == i, *n).clicked() {
                            *tipo = i as u8;
                        }
                    }
                });
            ui.end_row();
        });
    registrar_linha(label, &r.response);
}

pub(crate) fn grid_combo_alvo(ui: &mut Ui, label: &str, alvo: &mut u8) {
    let nomes = ["Posição", "Rotação", "Escala"];
    let atual = nomes.get(*alvo as usize).copied().unwrap_or(nomes[0]);
    let r = Grid::new(("alvo", label))
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label(label);
            ComboBox::from_id_salt(("alvo_cb", label))
                .selected_text(atual)
                .show_ui(ui, |ui| {
                    for (i, n) in nomes.iter().enumerate() {
                        if ui.selectable_label(*alvo as usize == i, *n).clicked() {
                            *alvo = i as u8;
                        }
                    }
                });
            ui.end_row();
        });
    registrar_linha(label, &r.response);
}

pub(crate) fn grid_combo_anim_alvo(ui: &mut Ui, label: &str, alvo: &mut u8) {
    let nomes = ["Posição", "Rotação", "Escala", "Opacidade"];
    let atual = nomes.get(*alvo as usize).copied().unwrap_or(nomes[0]);
    let r = Grid::new(("anim_alvo", label))
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label(label);
            ComboBox::from_id_salt(("anim_alvo_cb", label))
                .selected_text(atual)
                .show_ui(ui, |ui| {
                    for (i, n) in nomes.iter().enumerate() {
                        if ui.selectable_label(*alvo as usize == i, *n).clicked() {
                            *alvo = i as u8;
                        }
                    }
                });
            ui.end_row();
        });
    registrar_linha(label, &r.response);
}

pub(crate) fn grid_combo_loop(ui: &mut Ui, label: &str, modo: &mut u8) {
    let nomes = ["Nenhum", "Repetir", "Ping-pong"];
    let atual = nomes.get(*modo as usize).copied().unwrap_or(nomes[0]);
    Grid::new(("anim_loop", label))
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label(label);
            ComboBox::from_id_salt(("anim_loop_cb", label))
                .selected_text(atual)
                .show_ui(ui, |ui| {
                    for (i, n) in nomes.iter().enumerate() {
                        if ui.selectable_label(*modo as usize == i, *n).clicked() {
                            *modo = i as u8;
                        }
                    }
                });
            ui.end_row();
        });
}

pub fn draggable_value(
    ui: &mut Ui,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f32,
    suffix: &str,
    precision: usize,
) -> Response {
    let (min, max) = (*range.start(), *range.end());
    let r = ui.add(
        DragValue::new(v)
            .range(range)
            .speed(speed)
            .suffix(suffix)
            .fixed_decimals(precision),
    );
    let hscroll = crate::ui::scroll_delta(ui.ctx()).x;
    if hscroll != 0.0 && r.hovered() {
        let delta = hscroll * speed * 4.0;
        *v = (*v + delta).clamp(min, max);
        ui.ctx().request_repaint();
    }
    r
}

pub(crate) fn grid_xyz(ui: &mut Ui, label: &str, x: &mut f32, y: &mut f32, z: &mut f32) {
    let r = Grid::new(label)
        .num_columns(4)
        .spacing([4.0, 2.0])
        .show(ui, |ui| {
            ui.label(label);
            draggable_value(ui, x, -f32::INFINITY..=f32::INFINITY, 0.5, "", 1);
            draggable_value(ui, y, -f32::INFINITY..=f32::INFINITY, 0.5, "", 1);
            draggable_value(ui, z, -f32::INFINITY..=f32::INFINITY, 0.5, "", 1);
            ui.end_row();
        });
    registrar_linha(label, &r.response);
}

pub(crate) fn grid_2(
    ui: &mut Ui,
    label: &str,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    precision: usize,
) {
    let r = Grid::new(label)
        .num_columns(2)
        .spacing([6.0, 2.0])
        .show(ui, |ui| {
            ui.label(label);
            draggable_value(ui, v, range, 0.01, suffix, precision);
            ui.end_row();
        });
    registrar_linha(label, &r.response);
}

pub(crate) fn grid_escala(ui: &mut Ui, x: &mut f32, y: &mut f32) {
    Grid::new("pen_escala")
        .num_columns(3)
        .spacing([6.0, 2.0])
        .show(ui, |ui| {
            let img = Image::new(eframe::egui::include_image!(
                "../icons/escalapproporcional.svg"
            ))
            .fit_to_exact_size(Vec2::splat(14.0));
            ui.add(img).on_hover_text("Escala X / Y");
            draggable_value(ui, x, -10.0..=10.0, 0.01, "", 2);
            draggable_value(ui, y, -10.0..=10.0, 0.01, "", 2);
            ui.end_row();
        });
}
