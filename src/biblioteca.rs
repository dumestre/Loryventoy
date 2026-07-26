use eframe::egui;
use std::path::PathBuf;

fn pasta_padrao() -> String {
    let caminho = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("biblioteca");
    let _ = std::fs::create_dir_all(&caminho);
    caminho.to_string_lossy().to_string()
}

pub struct Biblioteca {
    aberta: bool,
    pasta: String,
    arquivos: Vec<(String, String)>,
}

impl Biblioteca {
    pub fn new() -> Self {
        let pasta = pasta_padrao();
        let mut b = Self {
            aberta: false,
            pasta,
            arquivos: Vec::new(),
        };
        b.atualizar();
        b
    }

    pub fn toggle(&mut self) {
        self.aberta = !self.aberta;
        if self.aberta {
            self.atualizar();
        }
    }

    pub fn atualizar(&mut self) {
        self.arquivos.clear();
        let pasta = std::path::Path::new(&self.pasta);
        if let Ok(entries) = std::fs::read_dir(pasta) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext = ext.to_lowercase();
                    if ["svg", "png", "jpg", "jpeg", "mp4", "mov", "webm", "gif", "bmp", "tiff"]
                        .contains(&ext.as_str())
                    {
                        if let Some(nome) = path.file_name().and_then(|n| n.to_str()) {
                            self.arquivos.push((nome.to_string(), ext));
                        }
                    }
                }
            }
        }
        self.arquivos.sort_by(|a, b| a.0.cmp(&b.0));
    }

    fn copiar_arquivo(&self, origem: &PathBuf) -> Result<(), String> {
        let nome = origem
            .file_name()
            .ok_or_else(|| "caminho invalido".to_string())?;
        let destino = std::path::Path::new(&self.pasta).join(nome);
        std::fs::copy(origem, &destino).map_err(|e| format!("erro ao copiar: {e}"))?;
        Ok(())
    }

    pub fn mostrar(&mut self, ui: &mut egui::Ui) {
        let dropped = ui.ctx().input(|i| i.raw.dropped_files.clone());

        ui.heading("Biblioteca");
        ui.label("Arraste arquivos para esta janela para importar");
        ui.label(&self.pasta);
        ui.separator();

        if !dropped.is_empty() {
            for file in &dropped {
                if let Some(path) = &file.path {
                    let _ = self.copiar_arquivo(path);
                }
            }
            self.atualizar();
        }

        if self.arquivos.is_empty() {
            ui.label("Nenhum ativo encontrado.");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("biblioteca_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Nome");
                        ui.label("Tipo");
                        ui.label("Acao");
                        ui.end_row();

                        for (nome, ext) in &self.arquivos.clone() {
                            ui.label(nome);
                            ui.label(ext.to_uppercase());
                            if ui.button("Usar").clicked() {
                                self.aberta = false;
                            }
                            ui.end_row();
                        }
                    });
            });
        }

        if ui.button("Fechar Biblioteca").clicked() {
            self.aberta = false;
        }
    }

    pub fn is_open(&self) -> bool {
        self.aberta
    }
}