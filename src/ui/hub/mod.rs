use eframe::egui;
use egui::CornerRadius;

use crate::projeto_arquivo::ProjetoArquivo;

pub mod versoes;
use versoes::{VERSOES, VERSAO_ATUAL};

// ── Paleta do ícone ──
const PINK: egui::Color32 = egui::Color32::from_rgb(241, 60, 119);
const GREEN: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);
const CARD: egui::Color32 = egui::Color32::from_rgb(40, 33, 37);
const CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(50, 42, 47);
const SURFACE: egui::Color32 = egui::Color32::from_rgb(30, 24, 28);
const TEXT: egui::Color32 = egui::Color32::from_rgb(245, 238, 242);
const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(168, 152, 160);
const DANGER: egui::Color32 = egui::Color32::from_rgb(255, 120, 130);
const BORDER: egui::Color32 = egui::Color32::from_rgb(58, 48, 54);

const R6: CornerRadius = CornerRadius::same(6);
const R8: CornerRadius = CornerRadius::same(8);
const R10: CornerRadius = CornerRadius::same(10);

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
    // projetos
    arquivos: Vec<String>,
    aviso: Option<String>,
    query: String,
    sort: SortBy,
    nome_novo: String,
    show_new: bool,
    delete_target: Option<String>,
    version_idx: usize,
    // instalacoes
    install: Option<InstallTask>,
    installed: Vec<String>,
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
            None => self.salvar("projeto.movimento.json", data),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        create: impl Fn() -> ProjetoArquivo + 'static,
    ) -> Option<ProjetoArquivo> {
        let mut aberto: Option<ProjetoArquivo> = None;
        let sidebar_w = 200.0;

        // animacao de instalacao
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

        let full_h = ui.available_height();
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(sidebar_w, full_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| self.sidebar(ui),
            );

            let cw = ui.available_width().max(100.0);
            ui.allocate_ui_with_layout(
                egui::vec2(cw, full_h),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::Frame::new().fill(SURFACE).corner_radius(R8).show(ui, |ui| {
                        ui.add_space(16.0);
                        ui.add_space(20.0);
                        self.conteudo(ui, &mut aberto, &create);
                    });
                },
            );
        });

        // ── Modal Novo Projeto ──
        if self.show_new {
            egui::Window::new("Novo Projeto")
                .id(egui::Id::new("new_proj"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false).resizable(false).movable(false)
                .frame(egui::Frame::new().fill(SURFACE).corner_radius(R10))
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(12.0);
                        ui.strong(egui::RichText::new("Criar Projeto").size(16.0).color(TEXT));
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Nome:");
                            ui.add(egui::TextEdit::singleline(&mut self.nome_novo).hint_text("meu-projeto").desired_width(200.0));
                        });
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new("Criar").fill(PINK).corner_radius(R6).min_size(egui::vec2(100.0, 32.0))).clicked() {
                                let base = if self.nome_novo.trim().is_empty() { "projeto" } else { self.nome_novo.trim() };
                                let nome = format!("{base}.movimento.json");
                                if self.caminho(&nome).exists() {
                                    self.aviso = Some(format!("ja existe: {nome}"));
                                } else {
                                    let mut data = create();
                                    data.script_text = format!("project \"{base}\" {{ width 1920 height 1080 fps 30 duration 8 background #1e1e26 }}\n");
                                    match self.salvar(&nome, &data) {
                                        Ok(()) => {
                                            self.current_project = Some(nome);
                                            self.varrer();
                                            aberto = Some(data);
                                            self.show_new = false;
                                            self.nome_novo.clear();
                                        }
                                        Err(e) => self.aviso = Some(e),
                                    }
                                }
                            }
                            if ui.add(egui::Button::new("Cancelar").corner_radius(R6).min_size(egui::vec2(100.0, 32.0))).clicked() {
                                self.show_new = false;
                                self.nome_novo.clear();
                            }
                        });
                        ui.add_space(12.0);
                    });
                });
        }

        // ── Modal Confirmar Exclusão ──
        if let Some(t) = self.delete_target.clone() {
            egui::Window::new("Excluir")
                .id(egui::Id::new("del_confirm"))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .collapsible(false).resizable(false).movable(false)
                .frame(egui::Frame::new().fill(SURFACE).corner_radius(R10))
                .show(ui.ctx(), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(12.0);
                        ui.label(egui::RichText::new(format!("Excluir '{t}' permanentemente?")).color(DANGER).size(14.0));
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new("Cancelar").corner_radius(R6).min_size(egui::vec2(100.0, 30.0))).clicked() {
                                self.delete_target = None;
                            }
                            if ui.add(egui::Button::new(egui::RichText::new("Sim, excluir").color(DANGER)).fill(egui::Color32::from_rgba_premultiplied(255, 120, 130, 20)).corner_radius(R6).min_size(egui::vec2(100.0, 30.0))).clicked() {
                                if let Err(e) = std::fs::remove_file(self.caminho(&t)) {
                                    self.aviso = Some(format!("falha ao excluir: {e}"));
                                } else {
                                    self.varrer();
                                    self.aviso = Some(format!("excluido: {t}"));
                                }
                                self.delete_target = None;
                            }
                        });
                        ui.add_space(12.0);
                    });
                });
        }

        // ── Aviso ──
        if let Some(msg) = &self.aviso {
            ui.add_space(8.0);
            egui::Frame::new().fill(egui::Color32::from_rgba_premultiplied(255, 120, 130, 15)).corner_radius(R6).show(ui, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| { ui.add_space(12.0); ui.label(egui::RichText::new(msg).color(DANGER).size(12.0)); });
                ui.add_space(8.0);
            });
        }

        aberto
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| { ui.add_space(16.0); ui.strong(egui::RichText::new("Loryventoy").size(18.0).color(TEXT)); });
        ui.add_space(4.0);
        ui.horizontal(|ui| { ui.add_space(16.0); ui.label(egui::RichText::new(format!("v{VERSAO_ATUAL}")).color(PINK).size(11.0).strong()); });
        ui.add_space(32.0);

        let itens: &[(&str, Pagina)] = &[
            ("  Projetos", Pagina::Projetos),
            ("  Instalações", Pagina::Instalacoes),
            ("  Sobre", Pagina::Sobre),
        ];
        for &(rotulo, pag) in itens {
            let ativo = self.pagina == pag;
            let resp = ui.add(
                egui::Button::new(egui::RichText::new(rotulo).size(14.0).color(if ativo { PINK } else { TEXT_MUTED }).strong())
                    .fill(egui::Color32::TRANSPARENT)
                    .corner_radius(R6)
                    .min_size(egui::vec2(180.0, 36.0)),
            );
            if ativo {
                let r = resp.rect;
                let bar = egui::Rect::from_min_max(egui::pos2(r.left(), r.top() + 4.0), egui::pos2(r.left() + 4.0, r.bottom() - 4.0));
                let p = ui.painter();
                for y in (bar.top() as usize)..(bar.bottom() as usize) {
                    let t = (y as f32 - bar.top()) / bar.height();
                    let cr = (255.0 * (1.0 - t) + 241.0 * t) as u8;
                    let cg = (214.0 * (1.0 - t) + 60.0 * t) as u8;
                    let cb = (107.0 * (1.0 - t) + 119.0 * t) as u8;
                    p.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(bar.left(), y as f32), egui::vec2(bar.width(), 1.0)),
                        CornerRadius::ZERO,
                        egui::Color32::from_rgb(cr, cg, cb),
                    );
                }
            }
            if resp.clicked() { self.pagina = pag; }
            ui.add_space(2.0);
        }
    }

    fn conteudo(&mut self, ui: &mut egui::Ui, aberto: &mut Option<ProjetoArquivo>, create: &dyn Fn() -> ProjetoArquivo) {
        match self.pagina {
            Pagina::Projetos => self.pag_projetos(ui, aberto, create),
            Pagina::Instalacoes => self.pag_instalacoes(ui),
            Pagina::Sobre => self.pag_sobre(ui),
        }
    }

    // ───────────────────── Projetos ─────────────────────

    fn pag_projetos(&mut self, ui: &mut egui::Ui, aberto: &mut Option<ProjetoArquivo>, _create: &dyn Fn() -> ProjetoArquivo) {
        ui.strong(egui::RichText::new("Projetos").size(20.0).color(TEXT));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Buscar...").desired_width(160.0).font(egui::TextStyle::Monospace));
            ui.add_space(6.0);
            if ui.add(egui::Button::new(egui::RichText::new("\u{21BB}").size(13.0)).corner_radius(R6).min_size(egui::vec2(26.0, 22.0))).clicked() { self.varrer(); }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for &(l, v) in &[("Nome", SortBy::Nome), ("Data", SortBy::Data), ("Tam", SortBy::Tamanho)] {
                    let sel = self.sort == v;
                    if ui.add(egui::Button::new(egui::RichText::new(l).size(11.0).color(if sel { PINK } else { TEXT_MUTED }).strong()).fill(if sel { egui::Color32::from_rgba_premultiplied(241, 60, 119, 18) } else { egui::Color32::TRANSPARENT }).corner_radius(R6).min_size(egui::vec2(44.0, 22.0))).clicked() {
                        self.sort = v;
                        self.varrer();
                    }
                }
            });
        });
        ui.add_space(6.0);

        let filtrados: Vec<String> = self.arquivos.iter().filter(|n| n.to_lowercase().contains(&self.query.to_lowercase())).cloned().collect();

        if filtrados.is_empty() {
            ui.add_space(32.0);
            egui::Frame::new().fill(CARD).corner_radius(R10).show(ui, |ui| {
                ui.add_space(32.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("\u{1F4C1}").size(32.0));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(if self.query.is_empty() { "Nenhum projeto ainda" } else { "Nenhum resultado" }).size(14.0).color(TEXT));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(if self.query.is_empty() { "Crie um novo projeto para comecar" } else { "Tente buscar com outro termo" }).color(TEXT_MUTED).size(12.0));
                    ui.add_space(10.0);
                    if ui.add(egui::Button::new(egui::RichText::new("+ Novo Projeto").color(PINK)).fill(egui::Color32::from_rgba_premultiplied(241, 60, 119, 12)).corner_radius(R8).min_size(egui::vec2(160.0, 32.0))).clicked() { self.show_new = true; }
                });
                ui.add_space(32.0);
            });
            return;
        }

        let cols = 4;
        let gap = 8.0;
        let cw = (ui.available_width() - (cols as f32 - 1.0) * gap) / cols as f32;
        let ch = 130.0;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for chunck in filtrados.chunks(cols) {
                ui.horizontal(|ui| {
                    for nome in chunck {
                        if let Some(a) = self.card(ui, nome, cw, ch) { *aberto = Some(a); }
                    }
                });
                ui.add_space(gap);
            }

            ui.add_space(gap * 2.0);
            ui.vertical_centered(|ui| {
                if ui.add(egui::Button::new(egui::RichText::new("+  Novo Projeto").color(PINK).size(13.0)).fill(egui::Color32::from_rgba_premultiplied(241, 60, 119, 10)).corner_radius(R8).min_size(egui::vec2(180.0, 34.0))).clicked() { self.show_new = true; }
            });
            ui.add_space(12.0);
        });
    }

    fn card(&mut self, ui: &mut egui::Ui, nome: &str, w: f32, h: f32) -> Option<ProjetoArquivo> {
        let mut aberto = None;
        let hover = ui.rect_contains_pointer(ui.max_rect());
        let (bg, border, sw) = if hover { (CARD_HOVER, PINK, 2.0) } else { (CARD, BORDER, 1.0) };

        let _ = ui.allocate_ui(egui::vec2(w, h), |ui| {
            egui::Frame::new().fill(bg).stroke(egui::Stroke::new(sw, border)).corner_radius(R8).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);

                    ui.strong(egui::RichText::new(nome.trim_end_matches(".movimento.json")).size(12.0).color(TEXT));

                    ui.add_space(4.0);

                    let meta = std::fs::metadata(self.caminho(nome)).ok();
                    let modified = meta.as_ref().and_then(|m| m.modified().ok()).map(Self::fmt_time);
                    let size = meta.map(|m| m.len()).unwrap_or(0);
                    ui.label(egui::RichText::new(match modified { Some(d) => d, None => "---".to_string() }).color(TEXT_MUTED).size(10.0));
                    ui.label(egui::RichText::new(Self::fmt_size(size)).color(TEXT_MUTED).size(10.0));

                    ui.add_space(8.0);

                    if ui.add(egui::Button::new(egui::RichText::new("Abrir").size(11.0).color(egui::Color32::WHITE)).fill(egui::Color32::from_rgba_premultiplied(241, 60, 119, 200)).corner_radius(R6).min_size(egui::vec2(w - 20.0, 24.0))).clicked() {
                        match self.ler(nome) {
                            Ok(data) => { self.current_project = Some(nome.to_string()); aberto = Some(data); }
                            Err(e) => self.aviso = Some(e),
                        }
                    }

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let bw = w - 28.0;
                        if ui.add(egui::Button::new(egui::RichText::new("Excluir").size(10.0).color(DANGER)).fill(egui::Color32::TRANSPARENT).corner_radius(R6).min_size(egui::vec2(bw, 20.0))).clicked() {
                            self.delete_target = Some(nome.to_string());
                        }
                    });
                });
            });
        });

        aberto
    }

    // ───────────────────── Instalações ─────────────────────

    fn pag_instalacoes(&mut self, ui: &mut egui::Ui) {
        ui.strong(egui::RichText::new("Instalações").size(20.0).color(TEXT));
        ui.add_space(8.0);

        let cols = 4;
        let gap = 8.0;
        let cw = (ui.available_width() - (cols as f32 - 1.0) * gap) / cols as f32;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for chunck in VERSOES.chunks(cols) {
                ui.horizontal(|ui| {
                    for v in chunck {
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

        let (bg, border) = if instalado {
            (egui::Color32::from_rgba_premultiplied(80, 200, 120, 12), egui::Stroke::new(1.5, GREEN))
        } else {
            (CARD, egui::Stroke::new(1.0, BORDER))
        };

        egui::Frame::new().fill(bg).stroke(border).corner_radius(R8).show(ui, |ui| {
            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.strong(egui::RichText::new(format!("v{}", v.numero)).size(15.0).color(TEXT));
                    if instalado {
                        ui.add_space(6.0);
                        egui::Frame::new().fill(egui::Color32::from_rgba_premultiplied(80, 200, 120, 25)).corner_radius(CornerRadius::same(3)).show(ui, |ui| {
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new("instalado").color(GREEN).size(10.0).strong());
                            ui.add_space(4.0);
                        });
                    }
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(v.titulo).color(TEXT_MUTED).size(12.0));
                });
                ui.add_space(8.0);
                for item in v.itens.iter().take(3) {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("\u{2022}").color(PINK).size(8.0));
                        ui.label(egui::RichText::new(*item).color(TEXT_MUTED).size(10.0));
                    });
                }
                if v.itens.len() > 3 {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("+{} mais", v.itens.len() - 3)).color(TEXT_MUTED).size(10.0));
                    });
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    if baixando {
                        let p = self.install.as_ref().unwrap().progress;
                        let bar_w = w - 32.0;
                        let bar_h = 6.0;
                        let bar_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(bar_w, bar_h));
                        ui.painter_at(bar_rect).rect_filled(bar_rect, R6, egui::Color32::from_rgba_premultiplied(255, 255, 255, 20));
                        let fill_w = bar_w * p;
                        if fill_w > 0.0 {
                            let fill_rect = egui::Rect::from_min_size(bar_rect.min, egui::vec2(fill_w, bar_h));
                            ui.painter_at(fill_rect).rect_filled(fill_rect, R6, PINK);
                        }
                        let pct = (p * 100.0) as u32;
                        ui.add_space(bar_w + 8.0);
                        ui.label(egui::RichText::new(format!("{pct}%")).color(TEXT_MUTED).size(11.0));
                    } else if instalado {
                        if ui.add(egui::Button::new(egui::RichText::new("Desinstalar").size(11.0).color(DANGER)).fill(egui::Color32::TRANSPARENT).corner_radius(R6).min_size(egui::vec2(w - 32.0, 26.0))).clicked() {
                            self.installed.retain(|x| x != v.numero);
                        }
                    } else {
                        if ui.add(egui::Button::new(egui::RichText::new("Instalar").size(11.0).color(egui::Color32::WHITE)).fill(egui::Color32::from_rgba_premultiplied(241, 60, 119, 200)).corner_radius(R6).min_size(egui::vec2(w - 32.0, 26.0))).clicked() {
                            self.install = Some(InstallTask { version: v.numero.to_string(), progress: 0.0 });
                            ui.ctx().request_repaint();
                        }
                    }
                });
            });
            ui.add_space(12.0);
        });
    }

    // ───────────────────── Sobre ─────────────────────

    fn pag_sobre(&mut self, ui: &mut egui::Ui) {
        ui.strong(egui::RichText::new("Sobre").size(22.0).color(TEXT));
        ui.add_space(12.0);

        egui::Frame::new().fill(CARD).corner_radius(R8).show(ui, |ui| {
            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                ui.strong(egui::RichText::new("Loryventoy").size(24.0).color(TEXT));
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("v{}", VERSAO_ATUAL)).color(PINK).size(14.0).strong());
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Editor de animacao procedural baseado em nos.").color(TEXT_MUTED));
            });
            ui.add_space(16.0);
        });

        ui.add_space(20.0);
        ui.strong(egui::RichText::new("Historico de Versoes").size(16.0).color(TEXT));
        ui.add_space(8.0);

        egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
            for (i, v) in VERSOES.iter().enumerate() {
                let atual = v.numero == VERSAO_ATUAL;
                let selecionado = self.version_idx == i;
                let (bg, stroke) = if atual {
                    (egui::Color32::from_rgba_premultiplied(241, 60, 119, 15), egui::Stroke::new(2.0, PINK))
                } else if selecionado {
                    (CARD_HOVER, egui::Stroke::new(1.0, PINK))
                } else {
                    (CARD, egui::Stroke::new(1.0, BORDER))
                };

                let resp = egui::Frame::new().fill(bg).stroke(stroke).corner_radius(R8).show(ui, |ui| {
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            ui.strong(egui::RichText::new(format!("v{}", v.numero)).size(13.0).color(TEXT));
                            if atual {
                                ui.add_space(6.0);
                                egui::Frame::new().fill(egui::Color32::from_rgba_premultiplied(241, 60, 119, 30)).corner_radius(CornerRadius::same(3)).show(ui, |ui| {
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("atual").color(PINK).size(10.0).strong());
                                    ui.add_space(4.0);
                                });
                            }
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| { ui.add_space(4.0); ui.label(egui::RichText::new(v.titulo).color(TEXT_MUTED).size(12.0)); });
                        ui.add_space(6.0);
                        for item in v.itens {
                            ui.horizontal(|ui| { ui.add_space(12.0); ui.label(egui::RichText::new("\u{2022}").color(PINK).size(9.0)); ui.label(egui::RichText::new(*item).color(TEXT_MUTED).size(11.0)); });
                        }
                    });
                    ui.add_space(10.0);
                }).response;

                if resp.clicked() { self.version_idx = i; }
                ui.add_space(6.0);
            }
        });
    }

    // ───────────────────── Helpers ─────────────────────

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
