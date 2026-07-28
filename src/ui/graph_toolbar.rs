use crate::nodes::TipoNo;
use eframe::egui::{self, Color32, Image, Popup, Rect, Stroke, Ui, Vec2};

/// Ação solicitada pela toolbar ao painel do grafo.
pub enum AcaoToolbar {
    Adicionar(TipoNo),
    Undo,
    Redo,
    /// Aplica um modelo de animação pronto a um novo nó de Texto (cria o
    /// Texto + o nó Animação já conectado). `id` identifica o preset.
    ModeloAnimTexto(u8),
}

pub struct GraphToolbar {
    pub search_query: String,
    pub focus_search: bool,
    pub acao: Option<AcaoToolbar>,
}

impl GraphToolbar {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            focus_search: false,
            acao: None,
        }
    }

    /// Desenha um botão de ação (texto) habilitado/desabilitado conforme
    /// `ativo`. Retorna `Some(acao)` ao clicar.
    fn botao_acao(
        ui: &mut Ui,
        rotulo: &str,
        ativo: bool,
        acao: AcaoToolbar,
    ) -> Option<AcaoToolbar> {
        if ui
            .add_enabled(
                ativo,
                egui::Button::new(rotulo).min_size(egui::Vec2::new(30.0, 22.0)),
            )
            .clicked()
        {
            Some(acao)
        } else {
            None
        }
    }

    pub fn show(&mut self, ui: &mut Ui, graph_rect: Rect, pode_undo: bool, pode_redo: bool) {
        let toolbar_w = 360.0;
        let padding = 8.0;
        let pos = egui::pos2(
            graph_rect.min.x + (graph_rect.width() - toolbar_w) / 2.0,
            graph_rect.min.y + padding,
        );

        egui::Area::new(egui::Id::new("graph_toolbar"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(toolbar_w - padding * 2.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    // Botões de desfazer/refazer
                    if let Some(a) = Self::botao_acao(ui, "↶", pode_undo, AcaoToolbar::Undo) {
                        self.acao = Some(a);
                    }
                    if let Some(a) = Self::botao_acao(ui, "↷", pode_redo, AcaoToolbar::Redo) {
                        self.acao = Some(a);
                    }

                    ui.separator();

                    // Campo de busca
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .desired_width(150.0)
                            .hint_text("buscar…"),
                    );
                    if resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.focus_search = true;
                    }

                    ui.separator();

                    // Botão de adicionar (ícone +) que abre o menu de tipos de nó
                    let img = Image::new(egui::include_image!("icons/add.svg"));
                    let opener = egui::Button::image(img)
                        .corner_radius(8.0)
                        .fill(Color32::from_rgb(45, 45, 58))
                        .stroke(Stroke::new(1.0, Color32::from_gray(80)))
                        .min_size(Vec2::new(26.0, 22.0));

                    let r = ui.add(opener);

                    let acao = &mut self.acao;
                    Popup::menu(&r).show(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("Master")
                                    .color(Color32::from_rgb(120, 220, 140)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Saida));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Transform")
                                    .color(Color32::from_rgb(235, 185, 95)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Transform));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Canvas")
                                    .color(Color32::from_rgb(170, 120, 235)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Canvas));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Cena").color(Color32::from_rgb(90, 190, 190)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Cena));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Layers")
                                    .color(Color32::from_rgb(120, 170, 235)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Layer));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Shape")
                                    .color(Color32::from_rgb(235, 150, 120)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Shape));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Texto")
                                    .color(Color32::from_rgb(150, 200, 120)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Texto.instancia()));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Pen").color(Color32::from_rgb(200, 120, 220)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Pen.instancia()));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Ruído")
                                    .color(Color32::from_rgb(120, 200, 220)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Ruido.instancia()));
                        }
                        if ui
                            .button(
                                egui::RichText::new("Animação")
                                    .color(Color32::from_rgb(230, 130, 170)),
                            )
                            .clicked()
                        {
                            *acao = Some(AcaoToolbar::Adicionar(TipoNo::Anim.instancia()));
                        }
                        ui.separator();
                        ui.label(egui::RichText::new("Modelos de animação (Texto)").strong());
                        let modelos = [
                            (0u8, "Fade In", Color32::from_rgb(150, 200, 120)),
                            (1u8, "Slide Esquerda", Color32::from_rgb(150, 200, 120)),
                            (2u8, "Slide Direita", Color32::from_rgb(150, 200, 120)),
                            (3u8, "Subir", Color32::from_rgb(150, 200, 120)),
                            (4u8, "Bounce", Color32::from_rgb(150, 200, 120)),
                            (5u8, "Zoom In", Color32::from_rgb(150, 200, 120)),
                        ];
                        for (id, nome, cor) in modelos {
                            if ui.button(egui::RichText::new(nome).color(cor)).clicked() {
                                *acao = Some(AcaoToolbar::ModeloAnimTexto(id));
                            }
                        }
                    });
                });
            });
    }
}
