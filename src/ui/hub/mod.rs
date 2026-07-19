use eframe::egui;
use crate::projeto_arquivo::ProjetoArquivo;

pub mod versoes;

pub struct HubPanel {
    pub pasta: String,
    pub current_project: Option<String>,
}

impl HubPanel {
    pub fn new() -> Self {
        Self {
            pasta: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string()),
            current_project: None,
        }
    }

    pub fn varrer(&mut self) {}

    pub fn salvar_atual(&self, _data: &ProjetoArquivo) -> Result<(), String> {
        Ok(())
    }

    pub fn show(
        &mut self,
        _ui: &mut egui::Ui,
        _create: impl Fn() -> ProjetoArquivo + 'static,
    ) -> Option<ProjetoArquivo> {
        None
    }
}
