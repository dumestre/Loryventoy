use std::path::Path;

use crate::domain::Project;
use crate::error::AppError;

use super::format::ProjetoArquivo;
use super::migrations;

pub fn load_project<P: AsRef<Path>>(caminho: P) -> Result<Project, AppError> {
    let texto = std::fs::read_to_string(caminho)?;
    load_from_str(&texto)
}

pub fn save_project<P: AsRef<Path>>(caminho: P, projeto: &Project) -> Result<(), AppError> {
    let arquivo = ProjetoArquivo::from_project(projeto);
    let json =
        serde_json::to_string_pretty(&arquivo).map_err(|e| AppError::Parse(e.to_string()))?;
    std::fs::write(caminho, json)?;
    Ok(())
}

pub fn load_from_str(texto: &str) -> Result<Project, AppError> {
    let mut arquivo: ProjetoArquivo =
        serde_json::from_str(texto).map_err(|e| AppError::Parse(e.to_string()))?;
    migrations::migrate(&mut arquivo);
    arquivo
        .to_project()
        .map_err(|e| AppError::InvalidProject(e))
}
