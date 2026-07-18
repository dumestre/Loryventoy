use eframe::egui;
use egui::CornerRadius;

use crate::projeto_arquivo::ProjetoArquivo;

pub mod versoes;
use versoes::{VERSOES, VERSAO_ATUAL};

// ── Colors ──
const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(18, 18, 28);
const SURFACE: egui::Color32 = egui::Color32::from_rgb(24, 24, 36);
const CARD: egui::Color32 = egui::Color32::from_rgb(32, 32, 48);
const CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(40, 40, 58);
const BORDER: egui::Color32 = egui::Color32::from_rgb(50, 50, 70);
const BRAND: egui::Color32 = egui::Color32::from_rgb(232, 120, 170);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(120, 170, 255);
const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(130, 130, 158);
const DANGER: egui::Color32 = egui::Color32::from_rgb(230, 120, 120);
const R10: CornerRadius = CornerRadius::same(10);
const R12: CornerRadius = CornerRadius::same(12);
const R6: CornerRadius = CornerRadius::same(6);
const R4: CornerRadius = CornerRadius::same(4);
const R8: CornerRadius = CornerRadius::same(8);
const LORY_SVG: &[u8] = include_bytes!("../icons/loryicon.svg");

pub struct HubPanel {
    pub pasta: String,
    arquivos: Vec<String>,
    aviso: Option<String>,
    nome_novo: String,
    projeto_atual: Option<String>,
    versao_selecionada: usize,
    pagina: PaginaHub,
}

enum PaginaHub {
    Projetos,
    Sobre,
}

impl HubPanel {
    pub fn new() -> Self {
        let pasta = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let mut h = Self {
            pasta,
            arquivos: Vec::new(),
            aviso: None,
            nome_novo: String::new(),
            projeto_atual: None,
            versao_selecionada: 0,
            pagina: PaginaHub::Projetos,
        };
        h.varrer();
        h
    }

    pub fn varrer(&mut self) {
        self.arquivos.clear();
        let dir = std::path::Path::new(&self.pasta);
        if !dir.exists() {
            self.aviso = Some(format!("pasta nao existe: {}", self.pasta));
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                self.aviso = Some(format!("nao foi possivel ler a pasta: {e}"));
                return;
            }
        };
        let mut nomes: Vec<String> = Vec::new();
        for ent in entries.flatten() {
            let path = ent.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| f.ends_with(".movimento.json"))
                    .unwrap_or(false)
            {
                if let Some(nome) = path.file_name().and_then(|f| f.to_str()) {
                    nomes.push(nome.to_string());
                }
            }
        }
        nomes.sort();
        self.arquivos = nomes;
        self.aviso = None;
    }

    fn caminho(&self, nome: &str) -> std::path::PathBuf {
        std::path::Path::new(&self.pasta).join(nome)
    }

    fn abrir(&self, nome: &str) -> Result<ProjetoArquivo, String> {
        let texto = std::fs::read_to_string(self.caminho(nome))
            .map_err(|e| format!("nao foi possivel ler {nome}: {e}"))?;
        let arquivo: ProjetoArquivo = serde_json::from_str(&texto)
            .map_err(|e| format!("arquivo invalido {nome}: {e}"))?;
        Ok(arquivo)
    }

    fn salvar_em(&self, nome: &str, arquivo: &ProjetoArquivo) -> Result<(), String> {
        let json = serde_json::to_string_pretty(arquivo)
            .map_err(|e| format!("falha ao serializar: {e}"))?;
        std::fs::write(self.caminho(nome), json)
            .map_err(|e| format!("nao foi possivel salvar {nome}: {e}"))
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        criar_projeto: impl Fn() -> ProjetoArquivo + 'static,
    ) -> Option<ProjetoArquivo> {
        let mut resultado: Option<ProjetoArquivo> = None;

        // ── TOP BAR ──
        egui::Frame::new()
            .fill(SIDEBAR_BG)
            .corner_radius(R8)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::from_bytes("bytes://loryicon.svg", LORY_SVG)
                            .max_size(egui::vec2(28.0, 28.0))
                    );
                    ui.add_space(8.0);
                    ui.strong(egui::RichText::new("Loryventoy").size(16.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("v{VERSAO_ATUAL}"))
                                .color(BRAND)
                                .size(12.0)
                                .strong(),
                        );
                        ui.add_space(8.0);
                        if ui.add(egui::Button::new("Varrer").corner_radius(R6)).clicked() {
                            self.varrer();
                        }
                        ui.add_space(4.0);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.pasta)
                                .desired_width(180.0)
                                .font(egui::TextStyle::Monospace),
                        );
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.varrer();
                        }
                    });
                });
            });
        ui.add_space(6.0);

        // ── SIDEBAR + CONTENT ──
        ui.horizontal(|ui| {
            // ── SIDEBAR ──
            ui.allocate_ui_with_layout(
                egui::vec2(180.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::Frame::new()
                        .fill(SIDEBAR_BG)
                        .corner_radius(R10)
                        .show(ui, |ui| {
                            ui.add_space(12.0);
                            let projetos_sel = matches!(self.pagina, PaginaHub::Projetos);
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("\u{25cf}  Projetos")
                                        .size(14.0)
                                        .color(if projetos_sel { BRAND } else { egui::Color32::WHITE })
                                        .strong(),
                                )
                                .fill(if projetos_sel {
                                    egui::Color32::from_rgba_premultiplied(232, 120, 170, 20)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .corner_radius(R6)
                                .min_size(egui::vec2(156.0, 36.0)),
                            ).clicked() {
                                self.pagina = PaginaHub::Projetos;
                            }

                            ui.add_space(4.0);

                            let sobre_sel = matches!(self.pagina, PaginaHub::Sobre);
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("\u{25cf}  Sobre")
                                        .size(14.0)
                                        .color(if sobre_sel { BRAND } else { egui::Color32::WHITE })
                                        .strong(),
                                )
                                .fill(if sobre_sel {
                                    egui::Color32::from_rgba_premultiplied(232, 120, 170, 20)
                                } else {
                                    egui::Color32::TRANSPARENT
                                })
                                .corner_radius(R6)
                                .min_size(egui::vec2(156.0, 36.0)),
                            ).clicked() {
                                self.pagina = PaginaHub::Sobre;
                            }

                            ui.add_space(16.0);
                            if ui.add(
                                egui::Button::new(
                                    egui::RichText::new("+  Novo Projeto")
                                        .size(13.0)
                                        .color(BRAND),
                                )
                                .fill(egui::Color32::from_rgba_premultiplied(232, 120, 170, 15))
                                .corner_radius(R6)
                                .min_size(egui::vec2(156.0, 36.0)),
                            ).clicked() {
                                self.pagina = PaginaHub::Projetos;
                                egui::Window::new("Novo Projeto")
                                    .id(egui::Id::new("novo_projeto_modal"))
                                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                                    .show(ui.ctx(), |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Nome:");
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.nome_novo)
                                                    .hint_text("nome do projeto")
                                                    .desired_width(200.0),
                                            );
                                        });
                                        ui.add_space(8.0);
                                        if ui.add(
                                            egui::Button::new("Criar Projeto")
                                                .fill(BRAND)
                                                .corner_radius(R6)
                                                .min_size(egui::vec2(140.0, 32.0)),
                                        ).clicked() {
                                            let base = if self.nome_novo.trim().is_empty() {
                                                "projeto"
                                            } else {
                                                self.nome_novo.trim()
                                            };
                                            let nome = format!("{base}.movimento.json");
                                            if self.caminho(&nome).exists() {
                                                self.aviso = Some(format!("ja existe: {nome}"));
                                            } else {
                                                let mut arquivo = criar_projeto();
                                                arquivo.script_text = format!(
                                                    "project \"{base}\" {{ width 1920 height 1080 fps 30 duration 8 background #1e1e26 }}\n"
                                                );
                                                match self.salvar_em(&nome, &arquivo) {
                                                    Ok(()) => {
                                                        self.projeto_atual = Some(nome);
                                                        self.varrer();
                                                        resultado = Some(arquivo);
                                                    }
                                                    Err(e) => self.aviso = Some(e),
                                                }
                                            }
                                        }
                                    });
                            }
                        });
                },
            );

            // ── CONTENT ──
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    match self.pagina {
                        PaginaHub::Projetos => self.pagina_projetos(ui, &mut resultado, &criar_projeto),
                        PaginaHub::Sobre => self.pagina_sobre(ui),
                    }
                },
            );
        });

        // ── Warnings ──
        if let Some(aviso) = &self.aviso {
            ui.colored_label(DANGER, aviso);
        }

        resultado
    }

    fn pagina_projetos(
        &mut self,
        ui: &mut egui::Ui,
        resultado: &mut Option<ProjetoArquivo>,
        _criar_projeto: &dyn Fn() -> ProjetoArquivo,
    ) {
        ui.vertical(|ui| {
            // ── Section header ──
            ui.horizontal(|ui| {
                ui.strong(egui::RichText::new("Projetos").size(20.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} projeto(s)", self.arquivos.len()))
                            .color(TEXT_MUTED)
                            .size(12.0),
                    );
                });
            });
            ui.add_space(8.0);

            if let Some(aviso) = &self.aviso {
                ui.colored_label(DANGER, aviso);
                ui.add_space(4.0);
            }

            // ── Grid ──
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if self.arquivos.is_empty() {
                        egui::Frame::new()
                            .fill(SURFACE)
                            .corner_radius(R12)
                            .show(ui, |ui| {
                                ui.add_space(48.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("Nenhum projeto ainda")
                                            .size(18.0)
                                            .color(egui::Color32::from_rgb(180, 180, 200)),
                                    );
                                    ui.add_space(6.0);
                                    ui.label(
                                        egui::RichText::new("Use o botao +Novo Projeto na barra lateral para criar um.")
                                            .color(TEXT_MUTED),
                                    );
                                });
                                ui.add_space(48.0);
                            });
                        return;
                    }

                    let card_w = 220.0;
                    let card_h = 210.0;
                    let gap = 12.0;
                    let avail = ui.available_width();
                    let cols = ((avail + gap) / (card_w + gap)).floor().max(1.0) as usize;

                    let arquivos: Vec<String> = self.arquivos.clone();
                    for chunk in arquivos.chunks(cols) {
                        ui.horizontal(|ui| {
                            for nome in chunk {
                                if let Some(arq) = self.card_projeto(ui, nome, card_w, card_h) {
                                    *resultado = Some(arq);
                                }
                            }
                            let used = chunk.len() as f32 * (card_w + gap);
                            let remaining = ui.available_width() - used + gap;
                            if remaining > 0.0 {
                                ui.add_space(remaining);
                            }
                        });
                        ui.add_space(gap);
                    }
                });
        });
    }

    fn card_projeto(
        &mut self,
        ui: &mut egui::Ui,
        nome: &str,
        card_w: f32,
        card_h: f32,
    ) -> Option<ProjetoArquivo> {
        let mut aberto: Option<ProjetoArquivo> = None;

        let inner = ui.allocate_ui(egui::vec2(card_w, card_h), |ui| {
            let hovered = ui.rect_contains_pointer(ui.max_rect());
            let fill = if hovered { CARD_HOVER } else { CARD };
            let stroke = if hovered {
                egui::Stroke::new(1.5, BRAND)
            } else {
                egui::Stroke::new(1.0, BORDER)
            };

            egui::Frame::new()
                .fill(fill)
                .stroke(stroke)
                .corner_radius(R10)
                .show(ui, |ui| {
                    // ── Thumbnail band ──
                    let band_h = 90.0;
                    let p = ui.painter_at(egui::Rect::from_min_size(
                        ui.max_rect().min,
                        egui::vec2(card_w, band_h),
                    ));
                    for y in (0..band_h as usize).step_by(4) {
                        let t = y as f32 / band_h;
                        let cr = (232.0 * (1.0 - t) + 100.0 * t) as u8;
                        let cg = (120.0 * (1.0 - t) + 140.0 * t) as u8;
                        let cb = (170.0 * (1.0 - t) + 255.0 * t) as u8;
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(ui.max_rect().left(), ui.max_rect().top() + y as f32),
                                egui::vec2(card_w, 4.0),
                            ),
                            CornerRadius::ZERO,
                            egui::Color32::from_rgb(cr, cg, cb),
                        );
                    }

                    // ── Icon overlay ──
                    let icon_c = egui::pos2(
                        ui.max_rect().left() + card_w / 2.0,
                        ui.max_rect().top() + band_h / 2.0,
                    );
                    let s = 8.0;
                    p.add(egui::Shape::convex_polygon(
                        vec![
                            icon_c + egui::vec2(0.0, -s),
                            icon_c + egui::vec2(s, 0.0),
                            icon_c + egui::vec2(0.0, s),
                            icon_c + egui::vec2(-s, 0.0),
                        ],
                        egui::Color32::from_rgba_premultiplied(255, 255, 255, 50),
                        egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(255, 255, 255, 120)),
                    ));

                    // ── Info ──
                    ui.add_space(band_h + 10.0);
                    ui.strong(egui::RichText::new(nome).size(13.0));

                    let meta = std::fs::metadata(self.caminho(nome)).ok();
                    let modified = meta
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .map(Self::format_system_time);
                    let size = meta.map(|m| m.len()).unwrap_or(0);
                    ui.label(
                        egui::RichText::new(match modified {
                            Some(m) => format!("{m}  {}", Self::format_file_size(size)),
                            None => Self::format_file_size(size),
                        })
                        .color(TEXT_MUTED)
                        .size(11.0),
                    );

                    ui.add_space(8.0);

                    // ── Actions ──
                    if ui.add(
                        egui::Button::new("Abrir")
                            .fill(BRAND)
                            .corner_radius(R6)
                            .min_size(egui::vec2(card_w - 24.0, 28.0)),
                    ).clicked() {
                        match self.abrir(nome) {
                            Ok(arq) => {
                                self.projeto_atual = Some(nome.to_string());
                                aberto = Some(arq);
                            }
                            Err(e) => self.aviso = Some(e),
                        }
                    }

                    ui.horizontal(|ui| {
                        if ui.small_button("Duplicar").clicked() {
                            let copia = format!("{nome}.copia.movimento.json");
                            match std::fs::copy(self.caminho(nome), self.caminho(&copia)) {
                                Ok(_) => {
                                    self.varrer();
                                    self.aviso = Some(format!("duplicado: {copia}"));
                                }
                                Err(e) => self.aviso = Some(format!("falha ao duplicar: {e}")),
                            }
                        }
                        ui.add_space(4.0);
                        if ui.small_button("Excluir").clicked() {
                            if let Err(e) = std::fs::remove_file(self.caminho(nome)) {
                                self.aviso = Some(format!("falha ao excluir: {e}"));
                            } else {
                                self.varrer();
                                self.aviso = Some(format!("excluido: {nome}"));
                            }
                        }
                    });
                });
        });

        if inner.response.clicked_by(egui::PointerButton::Secondary) {
            // could show context menu
        }

        aberto
    }

    fn pagina_sobre(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.strong(egui::RichText::new("Sobre o Loryventoy").size(20.0));
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(SURFACE)
                .corner_radius(R12)
                .show(ui, |ui| {
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        ui.add(
                            egui::Image::from_bytes("bytes://loryicon.svg", LORY_SVG)
                                .max_size(egui::vec2(64.0, 64.0))
                        );
                        ui.add_space(8.0);
                        ui.strong(egui::RichText::new("Loryventoy").size(24.0));
                        ui.label(
                            egui::RichText::new(format!("v{VERSAO_ATUAL}"))
                                .color(BRAND)
                                .size(14.0)
                                .strong(),
                        );
                    });
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(
                            "Editor de animacao procedural baseado em nos.\n\
                             Crie animacoes complexas com facilidade usando\n\
                             uma DSL poderosa e grafo visual."
                        )
                        .color(egui::Color32::from_rgb(170, 170, 196)),
                    );
                    ui.add_space(16.0);
                });

            ui.add_space(16.0);
            ui.strong(egui::RichText::new("Historico de Versoes").size(16.0));
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (i, v) in VERSOES.iter().enumerate() {
                        let atual = v.numero == VERSAO_ATUAL;
                        let selecionado = self.versao_selecionada == i;
                        let stroke = if atual {
                            egui::Stroke::new(1.5, BRAND)
                        } else if selecionado {
                            egui::Stroke::new(1.0, ACCENT)
                        } else {
                            egui::Stroke::new(1.0, BORDER)
                        };
                        let fill = if atual {
                            egui::Color32::from_rgb(42, 36, 50)
                        } else {
                            SURFACE
                        };

                        let resp = egui::Frame::new()
                            .fill(fill)
                            .stroke(stroke)
                            .corner_radius(R10)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(egui::RichText::new(format!("v{}", v.numero)).size(14.0));
                                        if atual {
                                            egui::Frame::new()
                                                .fill(egui::Color32::from_rgba_premultiplied(232, 120, 170, 30))
                                                .corner_radius(R4)
                                                .show(ui, |ui| {
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        egui::RichText::new("atual")
                                                            .color(BRAND)
                                                            .size(10.0)
                                                            .strong(),
                                                    );
                                                    ui.add_space(4.0);
                                                });
                                        }
                                    });
                                    ui.label(
                                        egui::RichText::new(v.titulo)
                                            .color(egui::Color32::from_rgb(170, 170, 196))
                                            .size(13.0),
                                    );
                                    ui.add_space(4.0);
                                    for item in v.itens {
                                        ui.horizontal(|ui| {
                                            ui.add_space(12.0);
                                            ui.label(egui::RichText::new(".").color(ACCENT).size(10.0));
                                            ui.label(
                                                egui::RichText::new(*item)
                                                    .color(TEXT_MUTED)
                                                    .size(12.0),
                                            );
                                        });
                                    }
                                });
                            })
                            .response;

                        if resp.clicked() {
                            self.versao_selecionada = i;
                        }
                        ui.add_space(8.0);
                    }
                });
        });
    }

    pub fn salvar_atual(&self, arquivo: &ProjetoArquivo) -> Result<(), String> {
        match &self.projeto_atual {
            Some(nome) => self.salvar_em(nome, arquivo),
            None => self.salvar_em("projeto.movimento.json", arquivo),
        }
    }

    fn format_system_time(t: std::time::SystemTime) -> String {
        let dur = match t.duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d,
            Err(_) => return String::new(),
        };
        let secs = dur.as_secs();
        let dias = secs / 86400;
        let mut y = 1970i32;
        let mut resto = dias as i64;
        loop {
            let bissexto = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            let dias_ano = if bissexto { 366 } else { 365 };
            if resto < dias_ano {
                break;
            }
            resto -= dias_ano;
            y += 1;
        }
        let dias_por_mes = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut mes = 0;
        let mut d = resto;
        let bissexto = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        loop {
            let dim = dias_por_mes[mes] + if mes == 1 && bissexto { 1 } else { 0 };
            if d < dim {
                break;
            }
            d -= dim;
            mes += 1;
        }
        let hora = (secs % 86400) / 3600;
        let min = (secs % 3600) / 60;
        format!("{:02}/{:02}/{} {:02}:{:02}", d + 1, mes + 1, y, hora, min)
    }

    fn format_file_size(size: u64) -> String {
        if size < 1024 {
            format!("{size} B")
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        }
    }
}
