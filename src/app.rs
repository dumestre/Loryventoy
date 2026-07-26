use eframe::egui;
use egui::Vec2;

use crate::theme;

use crate::projeto_arquivo::ProjetoArquivo;

use crate::ui::{
    graph::GraphPanel,
    preview::PreviewPanel,
    splitter::VerticalSplitter,
    timeline::TimelinePanel,
    bartool::BarTool,
    hub::HubPanel,
};


pub struct MovimentoApp {

    preview: PreviewPanel,
    timeline: TimelinePanel,
    graph: GraphPanel,
    bartool: BarTool,

    was_playing: bool,
    play_fps: f32,
    frame_accum: f32,
    last_time: f64,

    preview_height: f32,
    timeline_height: f32,
    graph_height: f32,

    min_panel_height: f32,
    splitter_size: f32,

    script_open: bool,
    script_text: String,
    script_erro: Option<String>,
    script_logs: Vec<String>,
    script_mostrar_exemplos: bool,
    script_rect: Option<egui::Rect>,
    script_primeira_vez: bool,

    salvar_pendente: bool,
    carregar_pendente: bool,
    projeto_aviso: Option<String>,

    hub: HubPanel,
    no_hub: bool,
}



impl MovimentoApp {


    pub fn new(
        cc: &eframe::CreationContext<'_>,
        start_project: Option<String>
    ) -> Self {

        theme::apply_theme(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut app = Self {
            preview: PreviewPanel::new(),
            timeline: TimelinePanel::new(),
            graph: GraphPanel::new(),
            bartool: BarTool::new(),

            was_playing: false,
            play_fps: 24.0,
            frame_accum: 0.0,
            last_time: 0.0,

            preview_height: 400.0,
            timeline_height: 80.0,
            graph_height: 300.0,

            min_panel_height: 100.0,
            splitter_size: 2.0,

            script_open: false,
            script_text: SCRIPT_EXEMPLO.to_string(),
            script_erro: None,
            script_logs: Vec::new(),
            script_mostrar_exemplos: false,
            script_rect: None,
            script_primeira_vez: true,

            salvar_pendente: false,
            carregar_pendente: false,
            projeto_aviso: None,

            hub: HubPanel::new(),
            no_hub: true,
        };

        if let Some(path) = start_project {
            if let Ok(raw) = std::fs::read_to_string(&path) {
                if let Ok(proj) = serde_json::from_str::<ProjetoArquivo>(&raw) {
                    app.carregar_arquivo_hub(proj);
                    app.no_hub = false; // Oculta o hub antigo
                    app.hub.current_project = Some(path);
                }
            }
        }

        app
    }



    fn resize_preview_graph(
        &mut self,
        delta: f32,
    ) {

        self.preview_height += delta;

        self.graph_height -= delta;


        self.clamp_sizes();
    }




    fn clamp_sizes(
        &mut self
    ) {

        let min = self.min_panel_height;


        if self.preview_height < min {

            let diff =
                min - self.preview_height;


            self.preview_height = min;

            self.graph_height -= diff;
        }



        if self.graph_height < min {

            let diff =
                min - self.graph_height;


            self.graph_height = min;

            self.preview_height -= diff;
        }

    }



    fn adjust_to_available_height(
        &mut self,
        available_height: f32,
        item_spacing_y: f32,
    ) {
        let total_fixed = self.timeline_height + 2.0 * self.splitter_size + 4.0 * item_spacing_y;
        let total_resizable = (available_height - total_fixed).max(2.0 * self.min_panel_height);

        let current_sum = self.preview_height + self.graph_height;
        if (current_sum - total_resizable).abs() > 0.01 {
            if current_sum > 0.0 {
                let ratio = total_resizable / current_sum;
                self.preview_height *= ratio;
                self.graph_height *= ratio;
            } else {
                self.preview_height = total_resizable / 2.0;
                self.graph_height = total_resizable / 2.0;
            }
            self.clamp_sizes();
        }
    }

    /// Registra uma mensagem no log do editor de script (com timestamp
    /// relativo simples: o índice sequencial). Mantém no máximo 500 linhas.
    fn log_script(&mut self, msg: impl Into<String>) {
        let n = self.script_logs.len() + 1;
        self.script_logs.push(format!("[{n}] {}", msg.into()));
        if self.script_logs.len() > 500 {
            let excesso = self.script_logs.len() - 500;
            self.script_logs.drain(0..excesso);
        }
    }

    /// Aplica o texto do editor DSL ao grafo, registrando erros de parse.
    /// Reutilizado pelo botão "Aplicar" e pelo atalho Ctrl+Enter.
    fn aplicar_script_editor(&mut self) {
        self.graph.empurrar_historico();
        match self.graph.aplicar_script(&self.script_text) {
            Ok(()) => {
                self.script_erro = None;
                self.log_script("OK: script aplicado, grafo reconstruído.");
            }
            Err(e) => {
                let msg = e.to_string();
                self.log_script(format!("ERRO: {msg}"));
                self.script_erro = Some(msg);
            }
        }
    }

    /// Roda todos os exemplos (projetos e canetas) um por vez, aplicando cada
    /// um ao grafo e capturando erros nos logs de forma rápida. Não altera a
    /// aba atual nem o texto do editor.
    fn testar_todos_exemplos(&mut self) {
        self.log_script("=== Teste de todos os exemplos ===".to_string());
        let mut ok = 0u32;
        let mut falha = 0u32;
        for ex in exemplos().iter() {
            let codigo = if ex.is_projeto {
                ex.codigo.to_string()
            } else {
                caneta_para_projeto(ex.codigo)
            };
            match self.graph.aplicar_script(&codigo) {
                Ok(()) => {
                    ok += 1;
                    self.log_script(format!("OK: {}", ex.nome));
                }
                Err(e) => {
                    falha += 1;
                    self.log_script(format!("ERRO [{}]: {}", ex.nome, e));
                }
            }
        }
        self.log_script(format!("=== Fim: {ok} ok, {falha} falha ==="));
    }

    /// Salva o projeto atual em arquivo JSON (caminho fixo na pasta do
    /// usuário). Sem dependência externa de diálogo: grava em
    /// `<dir_usuario>/movimento/projeto.lory`.
    fn salvar_projeto(&mut self) {
        let (nos, arestas) = self.graph.snapshot();
        let arquivo = ProjetoArquivo::from_graph(&nos, &arestas, &self.script_text);
        let json = match serde_json::to_string_pretty(&arquivo) {
            Ok(j) => j,
            Err(e) => {
                let msg = format!("falha ao serializar: {e}");
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
                return;
            }
        };
        let dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let caminho = dir.join("projeto.lory");
        match std::fs::write(&caminho, json) {
            Ok(()) => {
                let msg = format!("salvo em {}", caminho.display());
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
            }
            Err(e) => {
                let msg = format!("não foi possível salvar {}: {e}", caminho.display());
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
            }
        }
    }

    /// Carrega um projeto de arquivo JSON, reconstruindo o grafo e o script.
    fn carregar_projeto(&mut self) {
        let dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let caminho = dir.join("projeto.lory");
        let texto = match std::fs::read_to_string(&caminho) {
            Ok(t) => t,
            Err(e) => {
                let msg = format!("não foi possível ler {}: {e}", caminho.display());
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
                return;
            }
        };
        let arquivo: ProjetoArquivo = match serde_json::from_str(&texto) {
            Ok(a) => a,
            Err(e) => {
                let msg = format!("arquivo inválido: {e}");
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
                return;
            }
        };
        match arquivo.to_graph() {
            Ok((nos, arestas)) => {
                self.graph.empurrar_historico();
                self.graph.carregar_snapshot(&nos, &arestas);
                self.script_text = arquivo.script_text.clone();
                self.script_erro = None;
                let msg = format!("carregado de {}", caminho.display());
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
            }
            Err(e) => {
                let msg = format!("não foi possível reconstruir o grafo: {e}");
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
            }
        }
    }

    /// Aplica um `ProjetoArquivo` (aberto do hub) ao grafo e ao script.
    fn carregar_arquivo_hub(&mut self, arquivo: ProjetoArquivo) {
        match arquivo.to_graph() {
            Ok((nos, arestas)) => {
                self.graph.empurrar_historico();
                self.graph.carregar_snapshot(&nos, &arestas);
                self.script_text = arquivo.script_text.clone();
                self.script_erro = None;
            }
            Err(e) => {
                let msg = format!("não foi possível reconstruir o grafo: {e}");
                eprintln!("[Movimento] {msg}");
                self.projeto_aviso = Some(msg);
            }
        }
    }

    /// Snapshot do estado atual como `ProjetoArquivo` (para salvar no hub).
    fn snapshot_arquivo(&self) -> ProjetoArquivo {
        let (nos, arestas) = self.graph.snapshot();
        ProjetoArquivo::from_graph(&nos, &arestas, &self.script_text)
    }

}




impl eframe::App for MovimentoApp {


    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {

        // ---- TELA INICIAL (HUB DE PROJETOS) ----
        if self.no_hub {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title("Lory Hub".into()));
            let aberto = self.hub.show(ui, || {
                // criador de projeto em branco
                let (nos, arestas) = GraphPanel::new().snapshot();
                ProjetoArquivo::from_graph(&nos, &arestas, &String::new())
            });
            if let Some(arquivo) = aberto {
                self.carregar_arquivo_hub(arquivo);
                self.no_hub = false;
            }
            return;
        }

        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title("Loryventoy".into()));

        egui::CentralPanel::default()
            .show(ui, |ui| {

                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("Arquivo", |ui| {
                        if ui.button("Novo").clicked() {
                            self.graph.empurrar_historico();
                            self.graph = GraphPanel::new();
                            self.projeto_aviso = None;
                        }
                        let _ = ui.button("Abrir");
                        if ui.button("Salvar").clicked() {
                            match self.hub.salvar_atual(&self.snapshot_arquivo()) {
                                Ok(()) => {
                                    self.projeto_aviso =
                                        Some(format!("salvo na pasta {}", self.hub.pasta));
                                }
                                Err(e) => self.projeto_aviso = Some(e),
                            }
                        }
                        if ui.button("Carregar").clicked() {
                            self.carregar_pendente = true;
                        }
                        if ui.button("Hub (projetos)").clicked() {
                            self.hub.varrer();
                            self.no_hub = true;
                        }
                        if ui.button("Script (DSL)").clicked() {
                            self.script_open = true;
                        }
                        if ui.button("Sair").clicked() {
                            ui.ctx().send_viewport_cmd(
                                egui::ViewportCommand::Close
                            );
                        }
                    });

                    ui.menu_button("Editar", |ui| {
                        if ui
                            .add_enabled(self.graph.pode_undo(), egui::Button::new("Desfazer"))
                            .clicked()
                        {
                            if !self.graph.undo() {
                                self.projeto_aviso = Some("nada para desfazer".to_string());
                            }
                        }
                        if ui
                            .add_enabled(self.graph.pode_redo(), egui::Button::new("Refazer"))
                            .clicked()
                        {
                            if !self.graph.redo() {
                                self.projeto_aviso = Some("nada para refazer".to_string());
                            }
                        }
                    });

                    ui.menu_button("Ajuda", |ui| {
                        ui.label("Loryventoy Editor");
                    });
                });

                // ---- SALVAR / CARREGAR PROJETO (após o menu, fora do closure) ----
                if self.salvar_pendente {
                    self.salvar_pendente = false;
                    self.salvar_projeto();
                }
                if self.carregar_pendente {
                    self.carregar_pendente = false;
                    self.carregar_projeto();
                }
                let aviso = self.projeto_aviso.clone();
                if let Some(aviso) = aviso {
                    let limpar = ui.horizontal(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(140, 220, 160),
                            format!("Projeto: {aviso}"),
                        );
                        ui.button("OK").clicked()
                    }).inner;
                    if limpar {
                        self.projeto_aviso = None;
                    }
                }

                ui.separator();

                let available =
                    ui.available_size();

                self.adjust_to_available_height(
                    available.y,
                    ui.spacing().item_spacing.y,
                );



                // PREVIEW

                // resolução do projeto (nó Canvas) reflete no canvas do preview
                let cfg_preview = self.graph.projeto();
                self.preview.set_resolucao(cfg_preview.largura, cfg_preview.altura, cfg_preview.fundo);

                // sistema procedural: cenas (formas + textos) vêm do grafo; o
                // tempo vem da timeline (frame / fps) para a animação acompanhar
                // o play/pause e o scrub da linha do tempo.
                self.preview.set_preview(self.graph.formas_para_preview());
                let tempo = self.timeline.current_frame as f32 / cfg_preview.fps.max(0.01);
                self.preview.set_tempo(tempo);

                let preview_response = ui.allocate_ui(
                    Vec2::new(
                        available.x,
                        self.preview_height,
                    ),
                    |ui| {

                        self.preview.show(ui);

                    },
                );

                // BARTOOL (flutuante na base do preview)
                self.bartool.show(ui, preview_response.response.rect);

                // Processa pedidos da barra de transporte
                if self.bartool.request_prev_frame {
                    self.timeline.current_frame = self.timeline.current_frame.saturating_sub(1);
                }
                if self.bartool.request_next_frame {
                    self.timeline.current_frame = self.timeline.current_frame.saturating_add(1);
                }
                if self.bartool.request_stop {
                    self.bartool.is_playing = false;
                }



                // SPLITTER 1

                let splitter1 =
                    VerticalSplitter::new(
                        self.splitter_size
                    )
                    .show(ui);



                if splitter1.dragged() {

                    self.resize_preview_graph(
                        splitter1.drag_delta().y
                    );
                }



                // TIMELINE

                let _timeline_response = ui.allocate_ui(
                    Vec2::new(
                        available.x,
                        self.timeline_height,
                    ),
                    |ui| {

                        self.timeline.show(ui, self.bartool.loop_enabled, self.bartool.is_playing);

                    },
                );

                if self.timeline.markers_modificados {
                    self.graph.sincronizar_marcadores_com_cenas(&self.timeline.markers);
                    self.timeline.markers_modificados = false;
                }



                // SPLITTER 2

                let splitter2 =
                    VerticalSplitter::new(
                        self.splitter_size
                    )
                    .show(ui);



                if splitter2.dragged() {

                    self.resize_preview_graph(
                        splitter2.drag_delta().y
                    );
                }



                // GRAPH

                let _graph_response = ui.allocate_ui(
                    Vec2::new(
                        available.x,
                        ui.available_height(),
                    ),
                    |ui| {

                        self.graph.show(ui);

                // sincroniza a timeline com o nó Canvas do grafo
                let cfg = self.graph.projeto();
                crate::ui::timeline::definir_fps(cfg.fps);
                self.timeline.content_seconds = cfg.duracao_seg;
                self.timeline.loop_end = cfg.duracao_seg;
                self.timeline.duracao_frames =
                    (cfg.duracao_seg * cfg.fps).round().max(1.0) as u32;
                if (self.timeline.current_frame as f32)
                    >= self.timeline.duracao_frames as f32
                {
                    self.timeline.current_frame =
                        self.timeline.duracao_frames.saturating_sub(1);
                }

                    },
                );


                // ---- INPUT (play/pause e navegação de keyframes) ----
                let ctx = ui.ctx();
                let now = ctx.input(|i| i.time);

                if !ctx.egui_wants_keyboard_input() {
                    let input = ctx.input(|i| (
                        i.key_pressed(egui::Key::Space),
                        i.key_pressed(egui::Key::ArrowRight),
                        i.key_pressed(egui::Key::ArrowLeft),
                    ));

                    // Espaço: play/pause (funciona com ou sem ponteiro sobre editor)
                    if input.0 {
                        self.bartool.is_playing = !self.bartool.is_playing;
                        self.last_time = now;
                        self.frame_accum = 0.0;
                        ctx.request_repaint();
                    }

                    // Setas: agulha para o próximo / anterior keyframe
                    if input.1 {
                        self.timeline.current_frame =
                            self.timeline.current_frame.saturating_add(1);
                    }
                    if input.2 {
                        self.timeline.current_frame =
                            self.timeline.current_frame.saturating_sub(1);
                    }

                    // Ctrl+Z: desfazer; Ctrl+Y ou Ctrl+Shift+Z: refazer
                    let (undo, redo) = ctx.input(|i| {
                        let ctrl = i.modifiers.ctrl;
                        let shift = i.modifiers.shift;
                        let undo = ctrl && !shift && i.key_pressed(egui::Key::Z);
                        let redo_y = ctrl && !shift && i.key_pressed(egui::Key::Y);
                        let redo_shift = ctrl && shift && i.key_pressed(egui::Key::Z);
                        (undo, redo_y || redo_shift)
                    });
                    if undo {
                        if !self.graph.undo() {
                            self.projeto_aviso = Some("nada para desfazer".to_string());
                        }
                        ctx.request_repaint();
                    }
                    if redo {
                        if !self.graph.redo() {
                            self.projeto_aviso = Some("nada para refazer".to_string());
                        }
                        ctx.request_repaint();
                    }
                }


                // ---- AVANÇO DE PLAY ----
                // Sincroniza a base de tempo ao (re)iniciar a reprodução
                if self.bartool.is_playing && !self.was_playing {
                    self.last_time = now;
                    self.frame_accum = 0.0;
                }
                self.was_playing = self.bartool.is_playing;

                if self.bartool.is_playing {
                    let dt = (now - self.last_time) as f32;
                    self.last_time = now;
                    self.frame_accum += dt * self.play_fps;
                    let advance = self.frame_accum.floor() as u32;
                    self.frame_accum -= advance as f32;
                    self.timeline.current_frame =
                        self.timeline.current_frame.saturating_add(advance);

                    // Volta ao início do loop APENAS se o loop estiver ativo
                    if self.bartool.loop_enabled
                        && (self.timeline.current_frame as f32)
                            >= self.timeline.loop_end * crate::ui::timeline::fps_atual()
                    {
                        self.timeline.current_frame =
                            self.timeline.loop_start as u32;
                    }

                    ctx.request_repaint();
                }


            });

        // ---- JANELA DE SCRIPT (DSL de projeto) ----
        if self.script_open {
            let mut open = self.script_open;
            // Aplica posição/tamanho salvos APENAS na primeira exibição desta
            // abertura (com default_*), deixando o egui livre para o usuário
            // redimensionar e arrastar depois. `fixed_rect` travaria o tamanho.
            let mut janela = egui::Window::new("Script do Projeto (DSL)")
                .id(egui::Id::new("script_dsl_janela"))
                .open(&mut open)
                .resizable(true)
                .min_width(320.0)
                .min_height(160.0);
            if self.script_primeira_vez {
                if let Some(r) = self.script_rect {
                    janela = janela
                        .default_pos(r.min)
                        .default_size(r.size());
                } else {
                    janela = janela.default_width(560.0).default_height(460.0);
                }
                self.script_primeira_vez = false;
            }
            let resposta = janela.show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(!self.script_mostrar_exemplos, "Editor").clicked() {
                            self.script_mostrar_exemplos = false;
                        }
                        if ui.selectable_label(self.script_mostrar_exemplos, "Exemplos").clicked() {
                            self.script_mostrar_exemplos = true;
                        }
                    });
                    ui.separator();
                    if self.script_mostrar_exemplos {
                        if ui.button("▶ Rodar todos os exemplos (teste)").clicked() {
                            self.testar_todos_exemplos();
                        }
                        ui.separator();
                        ui.label("Exemplos de projeto (carregam no editor):");
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut algum_proj = false;
                            for ex in exemplos().iter() {
                                if !ex.is_projeto {
                                    continue;
                                }
                                algum_proj = true;
                                if ui.button(ex.nome).clicked() {
                                    self.script_text = ex.codigo.to_string();
                                    self.aplicar_script_editor();
                                }
                            }
                            if !algum_proj {
                                ui.label("(nenhum)");
                            }
                            ui.separator();
                            ui.label("Exemplos de caneta (clique para carregar no editor como projeto, ou copie o código do nó Pen):");
                            for ex in exemplos().iter() {
                                if ex.is_projeto {
                                    continue;
                                }
                                ui.horizontal(|ui| {
                                    if ui.button(ex.nome).clicked() {
                                        self.script_text =
                                            caneta_para_projeto(ex.codigo);
                                        self.aplicar_script_editor();
                                    }
                                    if ui.button("Copiar").clicked() {
                                        ui.ctx().copy_text(ex.codigo.to_string());
                                    }
                                });
                            }
                        });
                    } else {
                    // Atalho Ctrl+Enter: aplica o script sem precisar clicar.
                    let ctrl_enter = ui.input(|i| {
                        i.modifiers.ctrl && i.key_pressed(egui::Key::Enter)
                    });

                    // Painel INFERIOR fixo: botões + erro + logs. Ancorado ao
                    // fundo da janela para nunca ser empurrado para fora quando
                    // a altura é reduzida (o textarea acima é que encolhe).
                    egui::Panel::bottom("script_dsl_rodape")
                        .resizable(false)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.button("Aplicar").clicked() || ctrl_enter {
                                    self.aplicar_script_editor();
                                }
                                if ui.button("Exemplo").clicked() {
                                    self.script_text = SCRIPT_EXEMPLO.to_string();
                                }
                            });
                            if let Some(erro) = &self.script_erro {
                                ui.colored_label(egui::Color32::RED, format!("Erro: {erro}"));
                            }
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Logs").strong());
                                if ui.button("Copiar log").clicked() {
                                    ui.ctx().copy_text(self.script_logs.join("\n"));
                                }
                                if ui.button("Limpar").clicked() {
                                    self.script_logs.clear();
                                }
                            });
                            egui::ScrollArea::vertical()
                                .id_salt("script_dsl_logs")
                                .max_height(100.0)
                                .auto_shrink([false, false])
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    if self.script_logs.is_empty() {
                                        ui.weak("(sem logs ainda)");
                                    } else {
                                        for linha in &self.script_logs {
                                            let cor = if linha.contains("ERRO") {
                                                egui::Color32::from_rgb(230, 120, 120)
                                            } else {
                                                egui::Color32::from_rgb(150, 200, 150)
                                            };
                                            ui.colored_label(cor, linha);
                                        }
                                    }
                                });
                        });

                    // Área CENTRAL: rótulo + editor, ocupa o restante e rola.
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.label("Descreva o projeto em texto. 'Aplicar' (Ctrl+Enter) reconstrói o grafo.");
                        egui::ScrollArea::vertical()
                            .id_salt("script_dsl_texto")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.script_text)
                                        .code_editor()
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(4),
                                );
                            });
                    });
                    }
                });
            if let Some(r) = resposta.map(|r| r.response.rect) {
                self.script_rect = Some(r);
            }
            self.script_open = open;
            if !open {
                // janela fechada: prepara para reaplicar tamanho/pos na
                // próxima abertura
                self.script_primeira_vez = true;
            }
        }
    }
}

/// Exemplo de script DSL exibido por padrão na janela de Script.
const SCRIPT_EXEMPLO: &str = "\
project \"Exemplo\" {
  width 1920
  height 1080
  fps 30
  duration 8
  background #1e1e26
}

scene s1 { name \"Cena 1\" opacity 1.0 }
layer l1 { scene s1 name \"Formas\" }

pen coracao_principal {
  scene s1
  pos 960 540
  stroke 3.5
  fill on
  codigo {
    color 1 0.18 0.38

    let s = 13.5
    let rotacao = cos(t * 0.95) * 18.85

    repeat 120 {
      let a = i * 0.05236
      let x = 16 * sin(a) * sin(a) * sin(a)
      let y = -(13 * cos(a) - 5 * cos(2*a) - 2 * cos(3*a) - cos(4*a))

      let rx = x * cos(rotacao) - y * sin(rotacao)
      let ry = x * sin(rotacao) + y * cos(rotacao)

      line (rx * s) (ry * s)
    }
  }
}

pen particulas {
  scene s1
  pos 960 540
  codigo {
    color 1 0.65 0.9
    stroke 2

    repeat 24 {
      let ang = i * 0.2618 + t * 1.1
      let dist = 210 + sin(t * 2.8 + i) * 25
      let px = cos(ang) * dist
      let py = sin(ang) * dist * 0.75
      circle px py 3.5
    }
  }
}

pen coracao_orbitando {
  scene s1
  stroke 2.8
  fill on
  codigo {
    color 1 0.45 0.65

    let ang_orbita = t * 1.4
    let dist = 290
    let offset_x = cos(ang_orbita) * dist
    let offset_y = sin(ang_orbita) * (dist * 0.68)

    let scale = 4.2

    repeat 90 {
      let a = i * 0.06981
      let x = 16 * sin(a) * sin(a) * sin(a)
      let y = -(13 * cos(a) - 5 * cos(2*a) - 2 * cos(3*a) - cos(4*a))

      let rx = x * scale + offset_x
      let ry = y * scale + offset_y

      line rx ry
    }
  }
}

edge l1.Formas -> coracao_principal.Layer
edge l1.Formas -> particulas.Layer
edge l1.Formas -> coracao_orbitando.Layer
edge coracao_principal.out -> master.in
edge particulas.out -> master.in
edge coracao_orbitando.out -> master.in
";

/// Um exemplo, exibido na aba "Exemplos" da janela de Script.
struct ScriptExemplo {
    nome: &'static str,
    codigo: &'static str,
    /// true = exemplo de projeto (vai no editor DSL);
    /// false = exemplo de caneta (vai no clipboard para colar no nó Pen).
    is_projeto: bool,
}

/// Detecta se o código é um exemplo de projeto (script DSL) ou de caneta
/// (código puro da mini-linguagem). Projetos contêm as palavras-chave de
/// nível superior; canetas começam com comandos gráficos (stroke, fill...).
fn eh_projeto(codigo: &str) -> bool {
    let c = codigo.trim_start();
    c.starts_with("project")
        || c.contains("\nscene ")
        || c.contains("\nshape ")
        || c.contains("\ntext ")
        || c.contains("\npen ")
        || c.starts_with("scene ")
        || c.starts_with("shape ")
}

/// Conteúdo dos exemplos embutido de `docs/exemplos.md` em tempo de compilação.
const EXEMPLOS_MD: &str = include_str!("../docs/exemplos.md");

/// Divide o conteúdo de `exemplos.md` em exemplos. Cada exemplo é separado
/// dos demais por uma linha contendo apenas `---` (ou `===`). O título de
/// cada exemplo é a primeira linha de comentário (`# ...`) ou a primeira
/// linha não-vazia do bloco — comentários `#` internos de caneta NÃO criam
/// cortes, já que só o separador `---`/`===` marca o fim de um exemplo.
fn carregar_exemplos() -> Vec<ScriptExemplo> {
    let mut out = Vec::new();
    let linhas: Vec<&str> = EXEMPLOS_MD.lines().collect();
    let n = linhas.len();

    // Coleta os índices onde começam novos exemplos: linhas que são
    // exatamente o separador "---" ou "===". Guardamos o ÍNDICE DO PRÓPRIO
    // separador para que o bloco anterior TERME ANTES dele (o `---` não deve
    // entrar no código do exemplo).
    let mut cortes: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        let t = linhas[i].trim();
        if i > 0 && (t == "---" || t == "===") {
            cortes.push(i);
            // pula o separador e eventuais linhas vazias seguintes
            let mut j = i + 1;
            while j < n && linhas[j].trim().is_empty() {
                j += 1;
            }
            i = j;
            continue;
        }
        i += 1;
    }

    let mut exemplos: Vec<(String, String)> = Vec::new();
    if cortes.is_empty() {
        let texto = linhas.join("\n").trim().to_string();
        if !texto.is_empty() {
            exemplos.push((titulo_exemplo(&texto), texto));
        }
    } else {
        let mut inicio = 0;
        for c in cortes {
            let bloco: Vec<&str> = linhas[inicio..c].to_vec();
            let texto = bloco.join("\n").trim().to_string();
            if !texto.is_empty() {
                exemplos.push((titulo_exemplo(&texto), texto));
            }
            // o próximo exemplo começa na linha APÓS o separador (já pulada
            // pelo loop acima, então retomamos de `c + 1` se houver gap)
            inicio = c + 1;
        }
        let resto: Vec<&str> = linhas[inicio..].to_vec();
        let texto = resto.join("\n").trim().to_string();
        if !texto.is_empty() {
            exemplos.push((titulo_exemplo(&texto), texto));
        }
    }

    for (nome, codigo) in exemplos {
        let proj = eh_projeto(&codigo);
        out.push(ScriptExemplo {
            nome: Box::leak(nome.into_boxed_str()),
            codigo: Box::leak(codigo.into_boxed_str()),
            is_projeto: proj,
        });
    }
    if out.is_empty() {
        out.push(ScriptExemplo {
            nome: "Demo (padrão)",
            codigo: SCRIPT_EXEMPLO,
            is_projeto: true,
        });
    }
    out
}

/// Extrai um título legível para o exemplo: usa a primeira linha de
/// comentário (`# ...`) ou a primeira linha não-vazia.
fn titulo_exemplo(texto: &str) -> String {
    for linha in texto.lines() {
        let t = linha.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(resto) = t.strip_prefix('#') {
            let tit = resto.trim();
            if !tit.is_empty() {
                return tit.to_string();
            }
        }
        // primeira linha de código válida
        return if t.len() > 40 {
            format!("{}…", &t[..40])
        } else {
            t.to_string()
        };
    }
    "Exemplo".to_string()
}

/// Converte um código puro de caneta (PenDSL) num script de projeto
/// completo, embrulhando-o num bloco `pen { ... codigo { ... } }` dentro de
/// uma cena. Assim o exemplo pode ser carregado direto no editor DSL e
/// renderizado de imediato, em vez de só ser copiado para colar num nó Pen.
fn caneta_para_projeto(codigo_caneta: &str) -> String {
    let c = codigo_caneta.trim();
    let mut proj = String::new();
    proj.push_str("project \"Exemplo\" { width 1920 height 1080 fps 30 duration 8 background #1e1e26 }\n\n");
    proj.push_str("scene s1 { name \"Cena 1\" opacity 1.0 }\n\n");
    proj.push_str("layer l1 { scene s1 name \"Formas\" }\n\n");
    proj.push_str("pen p1 {\n");
    proj.push_str("  scene s1\n");
    proj.push_str("  pos 960 540\n");
    proj.push_str("  stroke 2\n");
    proj.push_str("  fill on\n");
    proj.push_str("  codigo {\n");
    for linha in c.lines() {
        proj.push_str("    ");
        proj.push_str(linha);
        proj.push('\n');
    }
    proj.push_str("  }\n");
    proj.push_str("}\n\n");
    proj.push_str("edge l1.Formas -> p1.Layer\n");
    proj.push_str("edge p1.out -> master.in\n");
    proj
}

/// Lista de exemplos disponíveis na aba "Exemplos", carregada de `docs/exemplos.md`.
static SCRIPT_EXEMPLOS: std::sync::OnceLock<Vec<ScriptExemplo>> = std::sync::OnceLock::new();

fn exemplos() -> &'static [ScriptExemplo] {
    SCRIPT_EXEMPLOS.get_or_init(carregar_exemplos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Program;
    use crate::dsl::project_dsl::parse_script;

    /// Garante que todos os exemplos de `docs/exemplos.md` parseiam sem erro
    /// (canetas via `Program`, projetos via `parse_script`). Quebra se o
    /// separador `---` vazar para dentro do código de um exemplo.
    #[test]
    fn todos_exemplos_parseiam() {
        for ex in exemplos().iter() {
            if ex.is_projeto {
                if let Err(e) = parse_script(ex.codigo) {
                    panic!("exemplo '{}' falhou ao parsear: {:?}", ex.nome, e);
                }
            } else {
                // canetas são envolvidas num pen antes de aplicar; o parser de
                // pen deve aceitar o código puro.
                if let Err(e) = Program::parse(ex.codigo) {
                    panic!("exemplo '{}' falhou ao parsear: {:?}", ex.nome, e);
                }
            }
        }
    }

    /// O separador `---` não deve aparecer dentro do código de nenhum exemplo.
    #[test]
    fn exemplos_nao_contem_separador() {
        for ex in exemplos().iter() {
            assert!(
                !ex.codigo.lines().any(|l| l.trim() == "---" || l.trim() == "==="),
                "exemplo '{}' contém o separador no código",
                ex.nome
            );
        }
    }
}