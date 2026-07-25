use eframe::egui;
use egui::{Color32, CornerRadius, FontId, RichText, Stroke, Vec2};

use crate::projeto_arquivo::ProjetoArquivo;

pub mod versoes;
use versoes::{VERSOES, VERSAO_ATUAL};

// ─── Paleta ───
const BG: Color32 = Color32::from_rgb(18, 16, 20);
const SURFACE: Color32 = Color32::from_rgb(28, 25, 30);
const CARD: Color32 = Color32::from_rgb(36, 32, 38);
const CARD_HOVER: Color32 = Color32::from_rgb(44, 40, 46);
const BORDER: Color32 = Color32::from_rgb(56, 50, 58);
const BORDER_FOCUS: Color32 = Color32::from_rgb(80, 70, 88);
const ACCENT: Color32 = Color32::from_rgb(241, 60, 119);
const ACCENT_DIM: Color32 = Color32::from_rgba_premultiplied(241, 60, 119, 40);
const GREEN: Color32 = Color32::from_rgb(80, 200, 120);
const TEXT: Color32 = Color32::from_rgb(235, 228, 238);
const TEXT_MUTED: Color32 = Color32::from_rgb(148, 132, 144);
const DANGER: Color32 = Color32::from_rgb(255, 110, 120);

// ─── Constantes ───
const SIDEBAR_W: f32 = 220.0;
const R6: CornerRadius = CornerRadius::same(6);
const R8: CornerRadius = CornerRadius::same(8);
const R10: CornerRadius = CornerRadius::same(10);
const R12: CornerRadius = CornerRadius::same(12);

#[derive(Clone, Copy, PartialEq)]
enum Pagina {
    Projetos,
    Instalacoes,
    Sobre,
}

#[derive(Clone, Copy, PartialEq)]
enum SortBy {
    Nome,
    Data,
    Tamanho,
}

struct InstallTask {
    version: String,
    progress: f32,
}

pub struct HubPanel {
    pub pasta: String,
    pub current_project: Option<String>,
    pagina: Pagina,
    arquivos: Vec<String>,
    aviso: Option<String>,
    query: String,
    sort: SortBy,
    nome_novo: String,
    show_new: bool,
    delete_target: Option<String>,
    version_idx: usize,
    install: Option<InstallTask>,
    installed: Vec<String>,
    anim_time: f32,
}

impl HubPanel {
    pub fn new() -> Self {
        let pasta = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let mut this = Self {
            pasta,
            current_project: None,
            pagina: Pagina::Projetos,
            arquivos: Vec::new(),
            aviso: None,
            query: String::new(),
            sort: SortBy::Data,
            nome_novo: String::new(),
            show_new: false,
            delete_target: None,
            version_idx: 0,
            install: None,
            installed: vec![VERSAO_ATUAL.to_string()],
            anim_time: 0.0,
        };
        this.varrer();
        this
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
                self.aviso = Some(format!("nao foi possivel ler: {e}"));
                return;
            }
        };
        let mut nomes: Vec<String> = Vec::new();
        for ent in entries.flatten() {
            let path = ent.path();
            if path.extension().and_then(|e| e.to_str()) == Some("lory") {
                if let Some(nome) = path.file_name().and_then(|f| f.to_str()) {
                    nomes.push(nome.to_string());
                }
            }
        }
        match self.sort {
            SortBy::Nome => nomes.sort(),
            SortBy::Data => {
                nomes.sort_by(|a, b| {
                    let ma = std::fs::metadata(self.caminho(a))
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    let mb = std::fs::metadata(self.caminho(b))
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    mb.cmp(&ma)
                });
            }
            SortBy::Tamanho => {
                nomes.sort_by(|a, b| {
                    let sa = std::fs::metadata(self.caminho(a)).map(|m| m.len()).unwrap_or(0);
                    let sb = std::fs::metadata(self.caminho(b)).map(|m| m.len()).unwrap_or(0);
                    sb.cmp(&sa)
                });
            }
        }
        self.arquivos = nomes;
        self.aviso = None;
    }

    fn caminho(&self, nome: &str) -> std::path::PathBuf {
        std::path::Path::new(&self.pasta).join(nome)
    }

    fn ler(&self, nome: &str) -> Result<ProjetoArquivo, String> {
        let raw = std::fs::read_to_string(self.caminho(nome))
            .map_err(|e| format!("nao foi possivel ler {nome}: {e}"))?;
        serde_json::from_str(&raw).map_err(|e| format!("arquivo invalido {nome}: {e}"))
    }

    fn salvar(&self, nome: &str, data: &ProjetoArquivo) -> Result<(), String> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("falha ao serializar: {e}"))?;
        std::fs::write(self.caminho(nome), json)
            .map_err(|e| format!("nao foi possivel salvar {nome}: {e}"))
    }

    pub fn salvar_atual(&self, data: &ProjetoArquivo) -> Result<(), String> {
        match &self.current_project {
            Some(nome) => self.salvar(nome, data),
            None => self.salvar("projeto.lory", data),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, create: impl Fn() -> ProjetoArquivo + 'static) -> Option<ProjetoArquivo> {
        let mut aberto: Option<ProjetoArquivo> = None;
        self.anim_time += ui.ctx().input(|i| i.stable_dt).min(0.02);

        if let Some(ref mut task) = self.install {
            task.progress = (task.progress + 0.015).min(1.0);
            if task.progress >= 1.0 {
                let ver = task.version.clone();
                if !self.installed.contains(&ver) {
                    self.installed.push(ver);
                }
                self.install = None;
            }
            ui.ctx().request_repaint();
        }

        let full_rect = ui.max_rect();

        ui.painter().rect_filled(full_rect, 0.0, BG);

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(Vec2::new(SIDEBAR_W, full_rect.height()), egui::Layout::top_down(egui::Align::Min), |ui| {
                self.sidebar(ui);
            });

            let content_w = ui.available_width().max(400.0);
            ui.allocate_ui_with_layout(Vec2::new(content_w, full_rect.height()), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    ui.allocate_ui_with_layout(Vec2::new(content_w - 48.0, ui.available_height()), egui::Layout::top_down(egui::Align::Min), |ui| {
                        self.content_area(ui, &mut aberto, &create);
                    });
                });
            });
        });

        if self.show_new {
            self.modal_novo_projeto(ui.ctx(), &create, &mut aberto);
        }
        if let Some(t) = self.delete_target.clone() {
            self.modal_excluir(ui.ctx(), &t);
        }

        if let Some(msg) = &self.aviso {
            self.show_toast(ui.ctx(), msg);
        }

        aberto
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(28.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("Loryventoy").font(FontId::proportional(22.0)).color(TEXT).strong());
                ui.add_space(2.0);
                ui.label(RichText::new(format!("v{VERSAO_ATUAL}")).font(FontId::monospace(11.0)).color(ACCENT).strong());
            });
        });
        ui.add_space(36.0);

        let itens: &[(&str, Pagina, &str)] = &[
            ("Projetos", Pagina::Projetos, "📁"),
            ("Instalações", Pagina::Instalacoes, "⬇"),
            ("Sobre", Pagina::Sobre, "ℹ"),
        ];

        for &(rotulo, pag, icon) in itens {
            let ativo = self.pagina == pag;
            let resp = ui.add_sized(
                [SIDEBAR_W - 40.0, 44.0],
                egui::Button::new(
                    RichText::new(format!("{}  {}", icon, rotulo))
                        .font(FontId::proportional(14.0))
                        .color(if ativo { ACCENT } else { TEXT_MUTED })
                        .strong()
                )
                .fill(if ativo { ACCENT_DIM } else { Color32::TRANSPARENT })
                .corner_radius(R8)
                .stroke(if ativo { Stroke::new(1.5, ACCENT) } else { Stroke::NONE })
            );

            if ativo {
                let r = resp.rect;
                let bar = egui::Rect::from_min_max(
                    egui::pos2(r.left(), r.top() + 6.0),
                    egui::pos2(r.left() + 4.0, r.bottom() - 6.0),
                );
                let p = ui.painter();
                for y in (bar.top() as usize)..(bar.bottom() as usize) {
                    let t = (y as f32 - bar.top()) / bar.height();
                    let cr = (255.0 * (1.0 - t) + 241.0 * t) as u8;
                    let cg = (214.0 * (1.0 - t) + 60.0 * t) as u8;
                    let cb = (107.0 * (1.0 - t) + 119.0 * t) as u8;
                    p.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(bar.left(), y as f32), egui::vec2(bar.width(), 1.0)),
                        CornerRadius::ZERO,
                        Color32::from_rgb(cr, cg, cb),
                    );
                }
            }

            if resp.hovered() && !ativo {
                ui.painter().rect_stroke(resp.rect, R8, Stroke::new(1.0, BORDER_FOCUS), egui::StrokeKind::Middle);
            }

            if resp.clicked() {
                self.pagina = pag;
            }
            ui.add_space(6.0);
        }

        ui.add_space(24.0);
        ui.add_space(24.0);

        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.label(RichText::new("Pasta dos projetos").font(FontId::proportional(11.0)).color(TEXT_MUTED));
                ui.add_space(4.0);
                let path_text = if self.pasta.len() > 30 {
                    format!("...{}", &self.pasta[self.pasta.len()-28..])
                } else {
                    self.pasta.clone()
                };
                ui.add(
                    egui::Label::new(RichText::new(path_text).font(FontId::monospace(10.0)).color(TEXT))
                        .wrap()
                        .truncate()
                );
            });
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            if ui.add(egui::Button::new(RichText::new("🔄 Atualizar").font(FontId::proportional(11.0)).color(TEXT_MUTED))
                .fill(Color32::TRANSPARENT)
                .corner_radius(R6)
                .min_size(Vec2::new(SIDEBAR_W - 40.0, 28.0))
            ).clicked() {
                self.varrer();
            }
        });
    }

    fn content_area(&mut self, ui: &mut egui::Ui, aberto: &mut Option<ProjetoArquivo>, create: &dyn Fn() -> ProjetoArquivo) {
        match self.pagina {
            Pagina::Projetos => self.pag_projetos(ui, aberto, create),
            Pagina::Instalacoes => self.pag_instalacoes(ui),
            Pagina::Sobre => self.pag_sobre(ui),
        }
    }

    fn pag_projetos(&mut self, ui: &mut egui::Ui, aberto: &mut Option<ProjetoArquivo>, create: &dyn Fn() -> ProjetoArquivo) {
        ui.label(RichText::new("Projetos").font(FontId::proportional(20.0)).color(TEXT).strong());
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            let search_resp = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("Buscar...")
                    .desired_width(180.0)
                    .font(FontId::monospace(12.0))
            );
            if search_resp.has_focus() {
                ui.painter().rect_stroke(search_resp.rect, R6, Stroke::new(2.0, ACCENT), egui::StrokeKind::Middle);
            }

            ui.add_space(6.0);
            if ui.add(egui::Button::new(RichText::new("↻").font(FontId::proportional(14.0))).corner_radius(R6).min_size(Vec2::new(28.0, 26.0))).clicked() {
                self.varrer();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for &(l, v) in &[("Nome", SortBy::Nome), ("Data", SortBy::Data), ("Tam", SortBy::Tamanho)] {
                    let sel = self.sort == v;
                    if ui.add(egui::Button::new(RichText::new(l).font(FontId::proportional(11.0)).color(if sel { ACCENT } else { TEXT_MUTED }).strong())
                        .fill(if sel { Color32::from_rgba_premultiplied(241, 60, 119, 18) } else { Color32::TRANSPARENT })
                        .corner_radius(R6)
                        .min_size(Vec2::new(48.0, 24.0))
                    ).clicked() {
                        self.sort = v;
                        self.varrer();
                    }
                }
            });
        });
        ui.add_space(6.0);

        let filtrados: Vec<String> = self.arquivos.iter()
            .filter(|n| n.to_lowercase().contains(&self.query.to_lowercase()))
            .cloned()
            .collect();

        if filtrados.is_empty() {
            self.empty_state(ui, create);
            return;
        }

        let cols = 4;
        let gap = 12.0;
        let cw = (ui.available_width() - (cols as f32 - 1.0) * gap) / cols as f32;
        let ch = 150.0;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for chunk in filtrados.chunks(cols) {
                ui.horizontal(|ui| {
                    for nome in chunk {
                        if let Some(a) = self.project_card(ui, nome, cw, ch) { *aberto = Some(a); }
                    }
                });
                ui.add_space(gap);
            }

            ui.add_space(gap * 2.0);
            ui.vertical_centered(|ui| {
                if self.primary_button(ui, "+  Novo Projeto", Vec2::new(180.0, 36.0)).clicked() { self.show_new = true; }
            });
            ui.add_space(12.0);
        });
    }

    fn empty_state(&mut self, ui: &mut egui::Ui, _create: &dyn Fn() -> ProjetoArquivo) {
        ui.add_space(32.0);
        egui::Frame::new().fill(CARD).corner_radius(R12).inner_margin(32.0).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("📁").font(FontId::proportional(48.0)));
                ui.add_space(12.0);
                ui.label(RichText::new(if self.query.is_empty() { "Nenhum projeto ainda" } else { "Nenhum resultado" }).font(FontId::proportional(16.0)).color(TEXT));
                ui.add_space(6.0);
                ui.label(RichText::new(if self.query.is_empty() { "Crie um novo projeto para começar" } else { "Tente buscar com outro termo" }).font(FontId::proportional(12.0)).color(TEXT_MUTED));
                ui.add_space(16.0);
                if self.primary_button(ui, "+ Novo Projeto", Vec2::new(180.0, 38.0)).clicked() { self.show_new = true; }
            });
        });
    }

    fn project_card(&mut self, ui: &mut egui::Ui, nome: &str, w: f32, h: f32) -> Option<ProjetoArquivo> {
        let mut aberto = None;
        let rect = ui.available_rect_before_wrap();
        let card_rect = egui::Rect::from_min_size(rect.min, Vec2::new(w, h));
        let hover = ui.rect_contains_pointer(card_rect);
        let (bg, border, bw) = if hover { (CARD_HOVER, ACCENT, 2.0) } else { (CARD, BORDER, 1.0) };

        ui.painter().rect_filled(card_rect, R10, bg);
        ui.painter().rect_stroke(card_rect, R10, Stroke::new(bw, border), egui::StrokeKind::Middle);

        ui.allocate_ui(card_rect.shrink(16.0).size(), |ui| {
            ui.vertical(|ui| {
                ui.add_space(4.0);

                let display_name = nome.trim_end_matches(".lory");
                ui.label(RichText::new(display_name).font(FontId::proportional(13.0)).color(TEXT).strong());
                ui.add_space(4.0);

                let meta = std::fs::metadata(self.caminho(nome)).ok();
                let modified = meta.as_ref().and_then(|m| m.modified().ok()).map(Self::fmt_time);
                let size = meta.map(|m| m.len()).unwrap_or(0);
                ui.label(RichText::new(match modified { Some(d) => d, None => "---".to_string() }).font(FontId::proportional(10.0)).color(TEXT_MUTED));
                ui.label(RichText::new(Self::fmt_size(size)).font(FontId::proportional(10.0)).color(TEXT_MUTED));

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    if self.primary_button(ui, "Abrir", Vec2::new((w - 40.0) / 2.0, 30.0)).clicked() {
                        match self.ler(nome) {
                            Ok(data) => { self.current_project = Some(nome.to_string()); aberto = Some(data); }
                            Err(e) => self.aviso = Some(e),
                        }
                    }
                    ui.add_space(8.0);
                    if ui.add(
                        egui::Button::new(RichText::new("Excluir").font(FontId::proportional(11.0)).color(DANGER).strong())
                            .fill(Color32::TRANSPARENT)
                            .corner_radius(R6)
                            .stroke(Stroke::new(1.0, DANGER))
                            .min_size(Vec2::new((w - 40.0) / 2.0, 30.0))
                    ).clicked() {
                        self.delete_target = Some(nome.to_string());
                    }
                    ui.add_space(4.0);
                });
            });
        });

        ui.advance_cursor_after_rect(card_rect);
        aberto
    }

    fn pag_instalacoes(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Instalações").font(FontId::proportional(20.0)).color(TEXT).strong());
        ui.add_space(8.0);

        let cols = 4;
        let gap = 12.0;
        let cw = (ui.available_width() - (cols as f32 - 1.0) * gap) / cols as f32;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for chunk in VERSOES.chunks(cols) {
                ui.horizontal(|ui| {
                    for v in chunk {
                        self.card_versao(ui, v, cw);
                    }
                });
                ui.add_space(gap);
            }
            ui.add_space(16.0);
        });
    }

    fn card_versao(&mut self, ui: &mut egui::Ui, v: &versoes::Versao, w: f32) {
        let instalado = self.installed.contains(&v.numero.to_string());
        let baixando = self.install.as_ref().map(|t| t.version == v.numero).unwrap_or(false);

        let border = if instalado {
            Stroke::new(1.5, GREEN)
        } else {
            Stroke::new(1.0, BORDER)
        };

        let h = 280.0;
        let rect = ui.available_rect_before_wrap();
        let card_rect = egui::Rect::from_min_size(rect.min, Vec2::new(w, h));

        ui.painter().rect_filled(card_rect, R10, CARD);
        ui.painter().rect_stroke(card_rect, R10, border, egui::StrokeKind::Middle);

        ui.allocate_ui(card_rect.shrink(16.0).size(), |ui| {
            ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("v{}", v.numero)).font(FontId::proportional(16.0)).color(TEXT).strong());
                        if instalado {
                            ui.add_space(6.0);
                            egui::Frame::new()
                                .fill(Color32::from_rgba_premultiplied(80, 200, 120, 25))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(4)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("instalado").font(FontId::proportional(10.0)).color(GREEN).strong());
                                });
                        }
                    });
                    ui.add_space(2.0);
                    ui.label(RichText::new(v.titulo).font(FontId::proportional(12.0)).color(TEXT_MUTED));
                    ui.add_space(8.0);
                    for item in v.itens.iter().take(4) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("•").font(FontId::proportional(12.0)).color(ACCENT));
                            ui.add_space(4.0);
                            ui.label(RichText::new(*item).font(FontId::proportional(11.0)).color(TEXT_MUTED));
                        });
                    }
                    if v.itens.len() > 4 {
                        ui.label(RichText::new(format!("+{} mais", v.itens.len() - 4)).font(FontId::proportional(10.0)).color(TEXT_MUTED));
                    }

                    ui.add_space(12.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if baixando {
                            let p = self.install.as_ref().unwrap().progress;
                            let bar_w = 120.0;
                            let bar_h = 6.0;
                            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, bar_h), egui::Sense::hover());
                            ui.painter_at(bar_rect).rect_filled(bar_rect, R6, Color32::from_rgba_premultiplied(255, 255, 255, 20));
                            let fill_w = bar_w * p;
                            if fill_w > 0.0 {
                                let fill_rect = egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_h));
                                ui.painter_at(fill_rect).rect_filled(fill_rect, R6, ACCENT);
                            }
                            let pct = (p * 100.0) as u32;
                            ui.label(RichText::new(format!("{pct}%")).font(FontId::proportional(11.0)).color(TEXT_MUTED).strong());
                        } else if instalado {
                            if ui.add(
                                egui::Button::new(RichText::new("Desinstalar").font(FontId::proportional(11.0)).color(DANGER).strong())
                                    .fill(Color32::TRANSPARENT)
                                    .corner_radius(R6)
                                    .stroke(Stroke::new(1.0, DANGER))
                                    .min_size(Vec2::new(100.0, 26.0))
                            ).clicked() {
                                self.installed.retain(|x| x != v.numero);
                            }
                        } else {
                            if ui.add(
                                egui::Button::new(RichText::new("Instalar").font(FontId::proportional(11.0)).color(TEXT).strong())
                                    .fill(ACCENT)
                                    .corner_radius(R6)
                                    .min_size(Vec2::new(100.0, 26.0))
                            ).clicked() {
                                self.install = Some(InstallTask { version: v.numero.to_string(), progress: 0.0 });
                                ui.ctx().request_repaint();
                            }
                        }
                    });
                });
            });

        ui.advance_cursor_after_rect(card_rect);
    }

    fn pag_sobre(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Sobre").font(FontId::proportional(22.0)).color(TEXT).strong());
        ui.add_space(12.0);

        egui::Frame::new().fill(CARD).corner_radius(R10).inner_margin(24.0).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Loryventoy").font(FontId::proportional(28.0)).color(TEXT).strong());
                ui.add_space(4.0);
                ui.label(RichText::new(format!("v{}", VERSAO_ATUAL)).font(FontId::proportional(14.0)).color(ACCENT).strong());
                ui.add_space(8.0);
                ui.label(RichText::new("Editor de animação procedural baseado em nós.").font(FontId::proportional(13.0)).color(TEXT_MUTED));
            });
        });

        ui.add_space(20.0);
        ui.label(RichText::new("Histórico de Versões").font(FontId::proportional(16.0)).color(TEXT).strong());
        ui.add_space(8.0);

        egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
            for (i, v) in VERSOES.iter().enumerate() {
                let atual = v.numero == VERSAO_ATUAL;
                let selecionado = self.version_idx == i;
                let (bg, stroke) = if atual {
                    (Color32::from_rgba_premultiplied(241, 60, 119, 15), Stroke::new(2.0, ACCENT))
                } else if selecionado {
                    (CARD_HOVER, Stroke::new(1.0, ACCENT))
                } else {
                    (CARD, Stroke::new(1.0, BORDER))
                };

                let resp = egui::Frame::new()
                    .fill(bg)
                    .stroke(stroke)
                    .corner_radius(R8)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                ui.label(RichText::new(format!("v{}", v.numero)).font(FontId::proportional(13.0)).color(TEXT).strong());
                                if atual {
                                    ui.add_space(6.0);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgba_premultiplied(241, 60, 119, 30))
.corner_radius(CornerRadius::same(3))
.inner_margin(4.0)
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("atual").font(FontId::proportional(10.0)).color(ACCENT).strong());
                                        });
                                }
                            });
                            ui.add_space(2.0);
                            ui.horizontal(|ui| { ui.add_space(4.0); ui.label(RichText::new(v.titulo).font(FontId::proportional(12.0)).color(TEXT_MUTED)); });
                            ui.add_space(6.0);
                            for item in v.itens {
                                ui.horizontal(|ui| {
                                    ui.add_space(12.0);
                                    ui.label(RichText::new("•").font(FontId::proportional(12.0)).color(ACCENT));
                                    ui.add_space(6.0);
                                    ui.label(RichText::new(*item).font(FontId::proportional(12.0)).color(TEXT_MUTED));
                                });
                            }
                        });
                    }).response;

                if resp.clicked() { self.version_idx = i; }
                ui.add_space(8.0);
            }
        });
    }

    fn modal_novo_projeto(&mut self, ctx: &egui::Context, create: &dyn Fn() -> ProjetoArquivo, aberto: &mut Option<ProjetoArquivo>) {
        egui::Window::new("Novo Projeto")
            .id(egui::Id::new("new_proj"))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false).resizable(false).movable(false)
            .frame(egui::Frame::new().fill(SURFACE).stroke(Stroke::new(1.0, BORDER)).corner_radius(R12).inner_margin(24.0))
            .show(ctx, |ui| {
                ui.set_min_width(380.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Criar Projeto").font(FontId::proportional(20.0)).color(TEXT).strong());
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(RichText::new("Nome").font(FontId::proportional(13.0)).color(TEXT));
                        ui.add_space(12.0);
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.nome_novo)
                                .hint_text("meu-projeto")
                                .desired_width(220.0)
                                .font(FontId::monospace(13.0))
                        );
                        if resp.has_focus() {
                            ui.painter().rect_stroke(resp.rect, R6, Stroke::new(2.0, ACCENT), egui::StrokeKind::Middle);
                        }
                        ui.add_space(20.0);
                    });

                    ui.add_space(24.0);

                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        let cancel = self.secondary_button(ui, "Cancelar", Vec2::new(110.0, 36.0));
                        ui.add_space(12.0);
                        let create_btn = self.primary_button(ui, "Criar", Vec2::new(110.0, 36.0));
                        ui.add_space(20.0);

                        if cancel.clicked() {
                            self.show_new = false;
                            self.nome_novo.clear();
                        }
                        if create_btn.clicked() {
                            let base = if self.nome_novo.trim().is_empty() { "projeto" } else { self.nome_novo.trim() };
                            let nome = format!("{base}.lory");
                            if self.caminho(&nome).exists() {
                                self.aviso = Some(format!("ja existe: {nome}"));
                            } else {
                                let mut data = create();
                                data.script_text = format!("project \"{base}\" {{ width 1920 height 1080 fps 30 duration 8 background #1c191e }}\n");
                                match self.salvar(&nome, &data) {
                                    Ok(()) => {
                                        self.current_project = Some(nome);
                                        self.varrer();
                                        *aberto = Some(data);
                                        self.show_new = false;
                                        self.nome_novo.clear();
                                    }
                                    Err(e) => self.aviso = Some(e),
                                }
                            }
                        }
                    });
                    ui.add_space(8.0);
                });
            });
    }

    fn modal_excluir(&mut self, ctx: &egui::Context, nome: &str) {
        egui::Window::new("Excluir Projeto")
            .id(egui::Id::new("del_confirm"))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false).resizable(false).movable(false)
            .frame(egui::Frame::new().fill(SURFACE).stroke(Stroke::new(1.0, BORDER)).corner_radius(R12).inner_margin(24.0))
            .show(ctx, |ui| {
                ui.set_min_width(360.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("Excluir '{nome}' permanentemente?")).font(FontId::proportional(14.0)).color(DANGER));
                    ui.add_space(8.0);
                    ui.label(RichText::new("Esta ação não pode ser desfeita.").font(FontId::proportional(12.0)).color(TEXT_MUTED));
                    ui.add_space(24.0);

                    ui.horizontal(|ui| {
                        if self.secondary_button(ui, "Cancelar", Vec2::new(110.0, 36.0)).clicked() {
                            self.delete_target = None;
                        }
                        ui.add_space(12.0);
                        if ui.add(
                            egui::Button::new(RichText::new("Sim, excluir").font(FontId::proportional(12.0)).color(TEXT).strong())
                                .fill(DANGER)
                                .corner_radius(R6)
                                .min_size(Vec2::new(110.0, 36.0))
                        ).clicked() {
                            if let Err(e) = std::fs::remove_file(self.caminho(nome)) {
                                self.aviso = Some(format!("falha ao excluir: {e}"));
                            } else {
                                self.varrer();
                                self.aviso = Some(format!("excluido: {nome}"));
                            }
                            self.delete_target = None;
                        }
                    });
                    ui.add_space(8.0);
                });
            });
    }

    fn show_toast(&self, ctx: &egui::Context, msg: &str) {
        egui::Area::new(egui::Id::new("hub_toast"))
            .anchor(egui::Align2::RIGHT_BOTTOM, Vec2::new(-24.0, -24.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(Color32::from_rgba_premultiplied(255, 110, 120, 220))
                    .stroke(Stroke::new(1.0, DANGER))
                    .corner_radius(R8)
                    .inner_margin(16)
                    .shadow(egui::Shadow::NONE)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            ui.label(RichText::new("⚠").font(FontId::proportional(14.0)));
                            ui.add_space(8.0);
                            ui.label(RichText::new(msg).font(FontId::proportional(13.0)).color(TEXT));
                            ui.add_space(4.0);
                        });
                    });
            });
    }

    fn primary_button(&mut self, ui: &mut egui::Ui, label: &str, size: Vec2) -> egui::Response {
        ui.add(
            egui::Button::new(RichText::new(label).font(FontId::proportional(12.0)).color(TEXT).strong())
                .fill(ACCENT)
                .corner_radius(R6)
                .min_size(size)
        )
    }

    fn secondary_button(&mut self, ui: &mut egui::Ui, label: &str, size: Vec2) -> egui::Response {
        ui.add(
            egui::Button::new(RichText::new(label).font(FontId::proportional(12.0)).color(TEXT_MUTED).strong())
                .fill(Color32::TRANSPARENT)
                .corner_radius(R6)
                .stroke(Stroke::new(1.0, BORDER))
                .min_size(size)
        )
    }

    fn fmt_time(t: std::time::SystemTime) -> String {
        let dur = match t.duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d,
            Err(_) => return String::new(),
        };
        let s = dur.as_secs();
        let d = s / 86400;
        let mut y = 1970i32;
        let mut r = d as i64;
        loop {
            let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
            if r < if leap { 366 } else { 365 } { break; }
            r -= if leap { 366 } else { 365 };
            y += 1;
        }
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let dim = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut m = 0usize;
        let mut d2 = r;
        loop {
            let max = dim[m] + if m == 1 && leap { 1 } else { 0 };
            if d2 < max { break; }
            d2 -= max;
            m += 1;
        }
        let h = (s % 86400) / 3600;
        let min = (s % 3600) / 60;
        format!("{:02}/{:02}/{} {:02}:{:02}", d2 + 1, m + 1, y, h, min)
    }

    fn fmt_size(s: u64) -> String {
        if s < 1024 { format!("{s} B") }
        else if s < 1024 * 1024 { format!("{:.1} KB", s as f64 / 1024.0) }
        else { format!("{:.1} MB", s as f64 / (1024.0 * 1024.0)) }
    }
}