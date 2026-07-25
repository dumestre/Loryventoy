#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::string::String;

use eframe::egui::{
    Align, Color32, ComboBox, DragValue, Grid, Image, Layout, Response,
    Sense, Stroke, TextStyle, Ui, Vec2,
};
use crate::graph_editor::NodeId;
use crate::nodes::{NodeParams, ProjetoConfig, TipoNo};

/// Ações que podem ser solicitadas pelo inspector de um nó.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum AcaoInspector {
    #[default]
    Nenhuma,
    FocarCena(NodeId),
    CriarLayerEntry,
    RemoverLayerEntry(usize),
    SubirLayerEntry(usize),
    DescerLayerEntry(usize),
    SelecionarLayer(usize),
    ToggleVisivelLayer(usize),
    RenomearLayerEntry(usize),
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
fn registrar_linha(nome: &str, resp: &Response) {
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
fn hex_de(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

/// Meia-extensão (w/2, h/2) do cartão do nó em coordenadas de canvas.
/// Usado pelo `egui-graph-edit` (hit-test/tamanho) e para posicionar o corpo.
/// O tamanho é responsivo: reflete a última medida do conteúdo do tipo.
/// (ver `registrar_medida`), com fallback enquanto não há medida.
/// Largura e altura vêm da medida real (ou fallback se ainda não mediu),
/// garantindo um tamanho mínimo igual ao fallback.
pub fn content_size(tipo: TipoNo) -> Vec2 {
    let fb = fallback_size(tipo);
    let measured = medidas().read().unwrap().get(&tipo).copied();
    Vec2::new(
        measured.map_or(fb.x, |m| m.x.max(fb.x)),
        measured.map_or(fb.y, |m| m.y.max(fb.y)),
    )
}

/// Desenha os parâmetros editáveis do nó DENTRO do corpo do cartão
/// (abaixo do cabeçalho, onde fica o nome), em layout de inspector:
/// rótulos alinhados em coluna à esquerda e campos à direita.
/// `cenas` é a lista de (nome, NodeId) de cena (para o combobox de Layers/Shape e listagem no Cena).
/// Retorna uma ação solicitada pelo inspector (ex.: focar em cena, criar layer).
pub fn show_content(
    ui: &mut Ui,
    tipo: TipoNo,
    params: Option<&mut NodeParams>,
    cenas: &[(String, NodeId)],
    topo_tela: f32,
    zoom: f32,
) -> AcaoInspector {
    let Some(params) = params else { return AcaoInspector::Nenhuma };
    // inicia a captura das posições Y das rows deste nó (para alinhar portos).
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
                grid_canvas(ui, cfg);
            }
        }
        TipoNo::Transform => {
            if let NodeParams::Transform {
                px, py, pz, rx, ry, rz, sx, sy, sz,
            } = params
            {
                grid_xyz(ui, "Posição", px, py, pz);
                grid_xyz(ui, "Rotação", rx, ry, rz);
                grid_xyz(ui, "Escala", sx, sy, sz);
            }
        }
        TipoNo::Cena => {
            if let NodeParams::Cena {
                nome_cena,
                ativa,
                zoom,
                angulo,
                opacidade,
            } = params
            {
                grid_texto(ui, "Cena", nome_cena);
                Grid::new("cena_ativa")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Ativa");
                        ui.checkbox(ativa, "");
                        ui.end_row();
                    });
                grid_2(ui, "Zoom", zoom, 0.01..=10.0, "", 2);
                grid_2(ui, "Ângulo", angulo, -360.0..=360.0, "°", 1);
                grid_2(ui, "Opacidade", opacidade, 0.0..=1.0, "", 2);

                ui.separator();
                ui.label(egui::RichText::new("Cenas disponíveis").strong());
                for (nome, idx) in cenas {
                ui.horizontal(|ui| {
                    ui.label(nome);
                    if ui.small_button("Focar").clicked() {
                        acao = AcaoInspector::FocarCena(*idx);
                    }
                });
                }
                if cenas.is_empty() {
                    ui.label("(crie marcadores na timeline)");
                }
            }
        }
        TipoNo::Layer => {
            if let NodeParams::Layer { cena, layers, selected } = params {
                grid_combo_cena(ui, "Cena", cena, cenas);
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(2.0);
                // Botão add — só o ícone, sem fundo
                ui.vertical_centered(|ui| {
                    let btn = ui.add(
                        Image::new(eframe::egui::include_image!("../ui/icons/add_clean.svg"))
                            .fit_to_exact_size(Vec2::splat(16.0))
                            .sense(Sense::click()),
                    ).on_hover_text("Adicionar Layer");
                    if btn.hovered() {
                        let r = btn.rect;
                        ui.painter().rect_filled(r.expand(2.0), 3.0, Color32::from_rgba_premultiplied(100, 200, 120, 20));
                    }
                    if btn.clicked() {
                        acao = AcaoInspector::CriarLayerEntry;
                    }
                });
                ui.add_space(4.0);
                // Lista de layers — ordem de baixo pra cima (última adicionada no topo)
                let mut acao_remover: Option<usize> = None;
                let mut acao_subir: Option<usize> = None;
                let mut acao_descer: Option<usize> = None;
                let count = layers.len();
                for rev_i in 0..count {
                    let i = count - 1 - rev_i;
                    let is_selected = *selected == i;
                    ui.horizontal(|ui| {
                        // Bolinha de cor
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
                        let cor = layers[i].cor;
                        let alpha = if layers[i].visivel { 1.0 } else { 0.35 };
                        ui.painter().circle_filled(rect.center(), 5.0, cor.gamma_multiply(alpha));
                        if !layers[i].visivel {
                            ui.painter().circle_stroke(rect.center(), 5.0, Stroke::new(1.0, Color32::from_rgb(80, 80, 90)));
                        }
                        // Toggle view
                        let vis_btn = if layers[i].visivel {
                            ui.add(
                                Image::new(eframe::egui::include_image!("../ui/icons/view_on.svg"))
                                    .fit_to_exact_size(Vec2::splat(14.0))
                                    .sense(Sense::click()),
                            ).on_hover_text("Ocultar layer")
                        } else {
                            ui.add(
                                Image::new(eframe::egui::include_image!("../ui/icons/view_off.svg"))
                                    .fit_to_exact_size(Vec2::splat(14.0))
                                    .sense(Sense::click()),
                            ).on_hover_text("Mostrar layer")
                        };
                        if vis_btn.hovered() {
                            let r = vis_btn.rect;
                            ui.painter().rect_filled(r.expand(1.5), 3.0, Color32::from_rgba_premultiplied(100, 200, 120, 20));
                        }
                        if vis_btn.clicked() {
                            layers[i].visivel = !layers[i].visivel;
                        }
                        // Nome
                        let nome_txt = if is_selected {
                            egui::RichText::new(&layers[i].nome).strong().color(Color32::from_rgb(220, 220, 240))
                        } else {
                            egui::RichText::new(&layers[i].nome)
                        };
                        let resp = ui.add(egui::Label::new(nome_txt).sense(Sense::click()));
                        if resp.clicked() {
                            acao = AcaoInspector::SelecionarLayer(i);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // Delete
                            let del_btn = ui.add(
                                Image::new(eframe::egui::include_image!("../ui/icons/delete_clean.svg"))
                                    .fit_to_exact_size(Vec2::splat(13.0))
                                    .sense(Sense::click()),
                            ).on_hover_text("Remover layer");
                            if del_btn.hovered() {
                                let r = del_btn.rect;
                                ui.painter().rect_filled(r.expand(1.5), 3.0, Color32::from_rgba_premultiplied(200, 80, 80, 20));
                            }
                            if del_btn.clicked() {
                                acao_remover = Some(i);
                            }
                            // Seta cima visual = mover p/ frente (índice +1)
                            let up_btn = ui.add(
                                Image::new(eframe::egui::include_image!("../ui/icons/arrow_up_clean.svg"))
                                    .fit_to_exact_size(Vec2::splat(13.0))
                                    .sense(Sense::click()),
                            ).on_hover_text("Mover para frente");
                            if up_btn.hovered() {
                                let r = up_btn.rect;
                                ui.painter().rect_filled(r.expand(1.5), 3.0, Color32::from_rgba_premultiplied(100, 200, 120, 20));
                            }
                            if up_btn.clicked() {
                                acao_subir = Some(i);
                            }
                            // Seta baixo visual = mover p/ trás (índice -1)
                            let down_btn = ui.add(
                                Image::new(eframe::egui::include_image!("../ui/icons/arrow_down_clean.svg"))
                                    .fit_to_exact_size(Vec2::splat(13.0))
                                    .sense(Sense::click()),
                            ).on_hover_text("Mover para trás");
                            if down_btn.hovered() {
                                let r = down_btn.rect;
                                ui.painter().rect_filled(r.expand(1.5), 3.0, Color32::from_rgba_premultiplied(100, 200, 120, 20));
                            }
                            if down_btn.clicked() {
                                acao_descer = Some(i);
                            }
                        });
                    });
                }
                // Processar ações estruturais (fora do loop)
                // Na ordem visual invertida: "cima" = aumentar índice, "baixo" = diminuir
                if let Some(i) = acao_remover {
                    acao = AcaoInspector::RemoverLayerEntry(i);
                } else if let Some(i) = acao_subir {
                    acao = AcaoInspector::DescerLayerEntry(i);
                } else if let Some(i) = acao_descer {
                    acao = AcaoInspector::SubirLayerEntry(i);
                }
            }
        }
        TipoNo::Shape => {
            if let NodeParams::Shape {
                cena,
                tipo,
                px,
                py,
                largura,
                altura,
                rotacao,
                cor,
                seed,
                noise_scale,
                amp,
                veloc,
                trim_inicio,
                trim_fim,
                ..
            } = params
            {
                grid_combo_cena(ui, "Cena", cena, cenas);
                grid_combo_tipo(ui, "Tipo", tipo);
                grid_xyz(ui, "Posição", px, py, &mut 0.0);
                grid_2(ui, "Largura", largura, 1.0..=8000.0, "", 1);
                grid_2(ui, "Altura", altura, 1.0..=8000.0, "", 1);
                grid_2(ui, "Rotação", rotacao, -360.0..=360.0, "°", 1);
                let rc = Grid::new("shape_cor")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Cor");
                        ui.horizontal(|ui| {
                            ui.color_edit_button_srgba(cor);
                            ui.label(hex_de(*cor));
                        });
                        ui.end_row();
                    });
                registrar_linha("Cor", &rc.response);
                // --- parâmetros procedurais ---
                grid_2(ui, "Seed", seed, 0.0..=9999.0, "", 0);
                grid_2(ui, "Ruído", noise_scale, 0.01..=5.0, "", 2);
                grid_2(ui, "Amplitude", amp, 0.0..=500.0, "", 1);
                grid_2(ui, "Velocidade", veloc, 0.0..=10.0, "x", 2);
                // --- trim (mostrado como 0-100%) ---
                {
                    let r = Grid::new("trim_inicio")
                        .num_columns(2).spacing([6.0, 2.0]).show(ui, |ui| {
                            ui.label("Trim início");
                            let mut v = *trim_inicio * 100.0;
                            if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                                *trim_inicio = (v / 100.0).clamp(0.0, 1.0);
                            }
                            ui.end_row();
                        });
                    registrar_linha("Trim início", &r.response);
                }
                {
                    let r = Grid::new("trim_fim")
                        .num_columns(2).spacing([6.0, 2.0]).show(ui, |ui| {
                            ui.label("Trim fim");
                            let mut v = *trim_fim * 100.0;
                            if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                                *trim_fim = (v / 100.0).clamp(0.0, 1.0);
                            }
                            ui.end_row();
                        });
                    registrar_linha("Trim fim", &r.response);
                }
            }
        }
        TipoNo::Texto => {
            if let NodeParams::Texto {
                cena,
                conteudo,
                tamanho,
                negrito,
                italico,
                px,
                py,
                cor,
                trim_inicio,
                trim_fim,
                ..
            } = params
            {
                grid_combo_cena(ui, "Cena", cena, cenas);
                Grid::new("texto_conteudo")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Conteúdo");
                        ui.text_edit_singleline(conteudo);
                        ui.end_row();
                    });
                grid_2(ui, "Tamanho", tamanho, 1.0..=2000.0, "", 1);
                Grid::new("texto_estilo")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Estilo");
                        ui.horizontal(|ui| {
                            ui.checkbox(negrito, "Negrito");
                            ui.checkbox(italico, "Itálico");
                        });
                        ui.end_row();
                    });
                grid_xyz(ui, "Posição", px, py, &mut 0.0);
                let rc = Grid::new("texto_cor")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Cor");
                        ui.horizontal(|ui| {
                            ui.color_edit_button_srgba(cor);
                            ui.label(hex_de(*cor));
                        });
                        ui.end_row();
                    });
                    registrar_linha("Cor", &rc.response);
                // --- trim texto 0-100% ---
                {
                    let r = Grid::new("texto_trim_inicio")
                        .num_columns(2).spacing([6.0, 2.0]).show(ui, |ui| {
                            ui.label("Trim início");
                            let mut v = *trim_inicio * 100.0;
                            if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                                *trim_inicio = (v / 100.0).clamp(0.0, 1.0);
                            }
                            ui.end_row();
                        });
                    registrar_linha("Trim início", &r.response);
                }
                {
                    let r = Grid::new("texto_trim_fim")
                        .num_columns(2).spacing([6.0, 2.0]).show(ui, |ui| {
                            ui.label("Trim fim");
                            let mut v = *trim_fim * 100.0;
                            if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                                *trim_fim = (v / 100.0).clamp(0.0, 1.0);
                            }
                            ui.end_row();
                        });
                    registrar_linha("Trim fim", &r.response);
                }
            }
        }
        TipoNo::Pen => {
            if let NodeParams::Pen {
                cena,
                codigo,
                erro,
                cor,
                cor_fill,
                pos_x,
                pos_y,
                espessura,
                preenchimento,
                seed,
                cantos,
                ordem,
                escala_x,
                escala_y,
                trim_inicio,
                trim_fim,
                ..
            } = params
            {
                grid_combo_cena(ui, "Cena", cena, cenas);
                grid_xyz(ui, "Posição", pos_x, pos_y, &mut 0.0);
                grid_2(ui, "Espessura", espessura, 0.0..=100.0, "px", 1);
                Grid::new("pen_preench")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Preencher");
                        ui.checkbox(preenchimento, "");
                        ui.end_row();
                    });
                grid_2(ui, "Cantos", cantos, 0.0..=1.0, "", 2);
                grid_2(ui, "Ordem", ordem, -100.0..=100.0, "", 1);
                grid_escala(ui, escala_x, escala_y);
                grid_2(ui, "Seed", seed, 0.0..=9999.0, "", 0);
                Grid::new("pen_cor")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Cor traço");
                        ui.horizontal(|ui| {
                            ui.color_edit_button_srgba(cor);
                            ui.label(hex_de(*cor));
                        });
                        ui.end_row();
                        ui.label("Cor preench.");
                        ui.horizontal(|ui| {
                            ui.color_edit_button_srgba(cor_fill);
                            ui.label(hex_de(*cor_fill));
                        });
                        ui.end_row();
                    });
                Grid::new("pen_codigo")
                    .num_columns(1)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Código DSL");
                        // Largura fixa (não INFINITA) e altura limitada por um
                        // ScrollArea, para que o textarea NÃO inflacione a
                        // altura do card do nó conforme o usuário digita.
                        let largura = (ui.available_width() * 1.6).max(280.0);
                        egui::ScrollArea::vertical()
                            .max_height(140.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(codigo)
                                        .code_editor()
                                        .font(egui::TextStyle::Monospace)
                                        .desired_rows(6)
                                        .desired_width(largura),
                                );
                            });
                        ui.end_row();
                    });
                // botões de autocompletar comandos DSL: inserem "\n<cmd> "
                // ao final do código.
                ui.horizontal_wrapped(|ui| {
                    let cmds = [
                        "move", "line", "rect", "circle", "bezier", "close", "fill on",
                        "stroke", "color", "stroke_color", "fill_color", "repeat", "if",
                    ];
                    for c in cmds {
                        if ui.small_button(c).clicked() {
                            let sep = if codigo.is_empty() || codigo.ends_with('\n') {
                                ""
                            } else {
                                "\n"
                            };
                            codigo.push_str(&format!("{sep}{c} "));
                        }
                    }
                });
                // re-parseia ao editar e reporta erro
                *erro = match crate::dsl::Program::parse(codigo) {
                    Ok(_) => None,
                    Err(e) => Some(e.to_string()),
                };

                ui.add_space(6.0);
                ui.separator();
                // Painel de log do Pen: status do parse + botão de copiar.
                let log_txt = match erro.as_deref() {
                    Some(e) => format!("ERRO: {e}"),
                    None => "OK: código válido.".to_string(),
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Log").strong());
                    if ui.small_button("Copiar log").clicked() {
                        ui.ctx().copy_text(log_txt.clone());
                    }
                });
                egui::ScrollArea::vertical()
                    .id_salt("pen_log")
                    .max_height(70.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let cor = if erro.is_some() {
                            Color32::from_rgb(230, 120, 120)
                        } else {
                            Color32::from_rgb(150, 200, 150)
                        };
                        ui.colored_label(cor, &log_txt);
                    });
                ui.add_space(6.0);
                // --- trim pen 0-100% ---
                {
                    let r = Grid::new("pen_trim_inicio")
                        .num_columns(2).spacing([6.0, 2.0]).show(ui, |ui| {
                            ui.label("Trim início");
                            let mut v = *trim_inicio * 100.0;
                            if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                                *trim_inicio = (v / 100.0).clamp(0.0, 1.0);
                            }
                            ui.end_row();
                        });
                    registrar_linha("Trim início", &r.response);
                }
                {
                    let r = Grid::new("pen_trim_fim")
                        .num_columns(2).spacing([6.0, 2.0]).show(ui, |ui| {
                            ui.label("Trim fim");
                            let mut v = *trim_fim * 100.0;
                            if draggable_value(ui, &mut v, 0.0..=100.0, 1.0, "%", 1).changed() {
                                *trim_fim = (v / 100.0).clamp(0.0, 1.0);
                            }
                            ui.end_row();
                        });
                    registrar_linha("Trim fim", &r.response);
                }
            }
        }
        TipoNo::Ruido => {
            if let NodeParams::Ruido {
                seed,
                freq,
                amp,
                veloc,
                alvo,
            } = params
            {
                grid_combo_alvo(ui, "Alvo", alvo);
                grid_2(ui, "Seed", seed, 0.0..=9999.0, "", 0);
                grid_2(ui, "Frequência", freq, 0.01..=5.0, "", 2);
                grid_2(ui, "Amplitude", amp, 0.0..=1000.0, "", 1);
                grid_2(ui, "Velocidade", veloc, 0.0..=10.0, "x", 2);
            }
        }
        TipoNo::Anim => {
            if let NodeParams::Anim {
                alvo,
                loop_mode,
                segmentos,
            } = params
            {
                grid_combo_anim_alvo(ui, "Alvo", alvo);
                grid_combo_loop(ui, "Loop", loop_mode);
                editor_segmentos(ui, segmentos);
            }
        }
        TipoNo::Saida => {
            if let NodeParams::Saida {
                brilho,
                contraste,
                saturacao,
            } = params
            {
                grid_2(ui, "Brilho", brilho, 0.0..=2.0, "", 2);
                grid_2(ui, "Contraste", contraste, 0.0..=2.0, "", 2);
                grid_2(ui, "Saturação", saturacao, 0.0..=2.0, "", 2);
            }
        }
    });
    // finaliza a captura: publica os Ys medidos no cache global.
    CAPTURA.with(|c| {
        if let Some(cap) = c.borrow_mut().take() {
            linhas_y().write().unwrap().insert(cap.tipo, cap.ys);
        }
    });
    acao
}

/// Linha com rótulo + campo de texto (nome da cena).
fn grid_texto(ui: &mut Ui, label: &str, v: &mut String) {
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

/// Linha com rótulo + combobox de cena (vincula Layers/Shape a uma cena).
fn grid_combo_cena(ui: &mut Ui, label: &str, cena: &mut String, cenas: &[(String, NodeId)]) {
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

/// Linha com rótulo + combobox de tipo de forma.
fn grid_combo_tipo(ui: &mut Ui, label: &str, tipo: &mut u8) {
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

/// Linha com rótulo + combobox do parâmetro alvo do nó Ruído.
fn grid_combo_alvo(ui: &mut Ui, label: &str, alvo: &mut u8) {
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

/// Combo do alvo de um nó Animação (inclui Opacidade).
fn grid_combo_anim_alvo(ui: &mut Ui, label: &str, alvo: &mut u8) {
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

/// Combo do modo de repetição (loop) de um nó Animação.
fn grid_combo_loop(ui: &mut Ui, label: &str, modo: &mut u8) {
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

/// Editor da lista de segmentos (trechos) de animação: uma linha por trecho
/// com tempos, valores inicial/final e curva de easing, além de botões para
/// adicionar/remover trechos.
fn editor_segmentos(ui: &mut Ui, segs: &mut Vec<crate::procedural::AnimSeg>) {
    use crate::procedural::{AnimSeg, Easing};
    let easings = ["Linear", "Ease-in", "Ease-out", "Ease-in-out", "Step"];
    ui.label(egui::RichText::new("Trechos").strong());
    let mut remover: Option<usize> = None;
    for (i, s) in segs.iter_mut().enumerate() {
        Grid::new(("anim_seg", i))
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.label("t");
                ui.horizontal(|ui| {
                    draggable_value(ui, &mut s.t_ini, 0.0..=3600.0, 0.05, "s", 2);
                    ui.label("→");
                    draggable_value(ui, &mut s.t_fim, 0.0..=3600.0, 0.05, "s", 2);
                });
                ui.end_row();
                ui.label("de");
                ui.horizontal(|ui| {
                    draggable_value(ui, &mut s.v_ini[0], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                    draggable_value(ui, &mut s.v_ini[1], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                });
                ui.end_row();
                ui.label("para");
                ui.horizontal(|ui| {
                    draggable_value(ui, &mut s.v_fim[0], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                    draggable_value(ui, &mut s.v_fim[1], -f32::INFINITY..=f32::INFINITY, 0.5, "", 2);
                });
                ui.end_row();
                ui.label("curva");
                let mut e = s.easing.to_u8() as usize;
                let atual = easings.get(e).copied().unwrap_or(easings[0]);
                ComboBox::from_id_salt(("anim_ease_cb", i))
                    .selected_text(atual)
                    .show_ui(ui, |ui| {
                        for (k, n) in easings.iter().enumerate() {
                            if ui.selectable_label(e == k, *n).clicked() {
                                e = k;
                            }
                        }
                    });
                s.easing = Easing::from_u8(e as u8);
                ui.end_row();
            });
        if ui.small_button("Remover trecho").clicked() {
            remover = Some(i);
        }
        ui.separator();
    }
    if let Some(i) = remover {
        if segs.len() > 1 {
            segs.remove(i);
        }
    }
    if ui.small_button("+ Trecho").clicked() {
        let (t0, v) = segs
            .last()
            .map(|s| (s.t_fim, s.v_fim))
            .unwrap_or((0.0, [0.0, 0.0]));
        segs.push(AnimSeg {
            t_ini: t0,
            t_fim: t0 + 1.0,
            v_ini: v,
            v_fim: v,
            easing: Easing::EaseInOut,
        });
    }
}

/// DragValue que também responde ao scroll HORIZONTAL do trackpad (2 dedos
/// na horizontal): o `egui` nativo só usa o scroll vertical, então o gesto
/// horizontal vazava para o pan do canvas. Aqui consumimos ambos os eixos
/// para alterar o valor (horizontal com passo mais rápido), e o canvas, ao
/// detectar que o ponteiro está sobre o conteúdo do nó, não faz pan.
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
    // scroll horizontal do trackpad (2 dedos na horizontal) → também altera o
    // valor, MAS somente quando o cursor está sobre ESTE campo (igual ao que o
    // `egui` faz no eixo vertical). Sem esse teste, o scroll_delta global
    // seria aplicado em todos os campos desenhados no mesmo frame.
    let hscroll = crate::ui::scroll_delta(ui.ctx()).x;
    if hscroll != 0.0 && r.hovered() {
        let delta = hscroll * speed * 4.0;
        *v = (*v + delta).clamp(min, max);
        ui.ctx().request_repaint();
    }
    r
}

/// Linha com rótulo + 3 campos (X, Y, Z) alinhados em colunas.
fn grid_xyz(ui: &mut Ui, label: &str, x: &mut f32, y: &mut f32, z: &mut f32) {
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

/// Linha com rótulo + 1 campo (DragValue) alinhados em 2 colunas.
fn grid_2(
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

/// Linha de escala X/Y com o ícone SVG de escala como rótulo.
fn grid_escala(ui: &mut Ui, x: &mut f32, y: &mut f32) {
    Grid::new("pen_escala")
        .num_columns(3)
        .spacing([6.0, 2.0])
        .show(ui, |ui| {
            let img = Image::new(eframe::egui::include_image!("icons/escalapproporcional.svg"))
                .fit_to_exact_size(Vec2::splat(14.0));
            ui.add(img).on_hover_text("Escala X / Y");
            draggable_value(ui, x, -10.0..=10.0, 0.01, "", 2);
            draggable_value(ui, y, -10.0..=10.0, 0.01, "", 2);
            ui.end_row();
        });
}

/// Parâmetros do nó Canvas: Preset, Largura, Altura, FPS, Duração, Fundo.
fn grid_canvas(ui: &mut Ui, cfg: &mut ProjetoConfig) {
    // preset atual (bate largura+altura com algum da lista)
    let atual = PRESETS_RESOLUCAO
        .iter()
        .find(|(_, w, h)| *w == cfg.largura && *h == cfg.altura)
        .map(|(nome, _, _)| *nome)
        .unwrap_or("Personalizado");
    // largura do ComboBox = só o necessário para o maior rótulo
    let font = TextStyle::Button.resolve(ui.style());
    let largura_combo = std::iter::once("Personalizado")
        .chain(PRESETS_RESOLUCAO.iter().map(|(n, _, _)| *n))
        .map(|t| {
            ui.ctx().fonts_mut(|f| {
                f.layout_no_wrap(t.to_string(), font.clone(), Color32::WHITE)
                    .size()
                    .x
            })
        })
        .fold(0.0f32, f32::max)
        + 24.0;

    Grid::new("canvas_cfg")
        .num_columns(2)
        .spacing([8.0, 3.0])
        .show(ui, |ui| {
            ui.label("Preset");
            ComboBox::from_id_salt("preset_res")
                .selected_text(atual)
                .width(largura_combo)
                .show_ui(ui, |ui| {
                    for (nome, w, h) in PRESETS_RESOLUCAO {
                        let sel = *w == cfg.largura && *h == cfg.altura;
                        if ui.selectable_label(sel, *nome).clicked() {
                            cfg.largura = *w;
                            cfg.altura = *h;
                        }
                    }
                });
            ui.end_row();

            ui.label("Largura");
            ui.add(DragValue::new(&mut cfg.largura).range(16..=7680));
            ui.end_row();

            ui.label("Altura");
            ui.add(DragValue::new(&mut cfg.altura).range(16..=4320));
            ui.end_row();

            ui.label("FPS");
            draggable_value(ui, &mut cfg.fps, 1.0..=120.0, 0.1, "", 1);
            ui.end_row();

            ui.label("Duração");
            draggable_value(ui, &mut cfg.duracao_seg, 0.1..=3600.0, 0.1, " s", 1);

            ui.label("Fundo");
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut cfg.fundo);
                ui.label(hex_de(cfg.fundo));
            });
            ui.end_row();
        });
}
