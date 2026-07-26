use eframe::egui;

pub struct Biblioteca {
    aberta: bool,
    pasta: String,
    arquivos: Vec<(String, String)>,
}

impl Biblioteca {
    pub fn new() -> Self {
        let pasta = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from("."));
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
            self.pasta = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| String::from("."));
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
                    if ["svg", "png", "jpg", "jpeg", "mp4", "mov", "webm", "gif", "bmp", "tiff"].contains(&ext.as_str()) {
                        if let Some(nome) = path.file_name().and_then(|n| n.to_str()) {
                            self.arquivos.push((nome.to_string(), ext));
                        }
                    }
                }
            }
        }
        self.arquivos.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn mostrar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Biblioteca");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Pasta:");
            ui.text_edit_singleline(&mut self.pasta);
            if ui.button("Escolher pasta").clicked() {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.pasta = dir.to_string_lossy().to_string();
                    self.atualizar();
                }
            }
        });

        ui.separator();

        if self.arquivos.is_empty() {
            ui.label("Nenhum ativo encontrado na pasta selecionada.");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("biblioteca_grid").striped(true).show(ui, |ui| {
                    ui.label("Nome");
                    ui.label("Tipo");
                    ui.label("Ação");
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
