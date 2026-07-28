use std::path::Path;

use crate::domain::Project;

use super::format::ProjetoArquivo;
use super::migrations;

#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Parse(String),
    InvalidProject(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "erro de I/O: {e}"),
            PersistenceError::Parse(e) => write!(f, "erro de formato: {e}"),
            PersistenceError::InvalidProject(e) => write!(f, "projeto inválido: {e}"),
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::Io(e)
    }
}

pub fn load_project<P: AsRef<Path>>(caminho: P) -> Result<Project, PersistenceError> {
    let texto = std::fs::read_to_string(caminho)?;
    load_from_str(&texto)
}

pub fn save_project<P: AsRef<Path>>(caminho: P, projeto: &Project) -> Result<(), PersistenceError> {
    let arquivo = ProjetoArquivo::from_project(projeto);
    let json = serde_json::to_string_pretty(&arquivo)
        .map_err(|e| PersistenceError::Parse(e.to_string()))?;
    std::fs::write(caminho, json)?;
    Ok(())
}

pub fn load_from_str(texto: &str) -> Result<Project, PersistenceError> {
    let mut arquivo: ProjetoArquivo =
        serde_json::from_str(texto).map_err(|e| PersistenceError::Parse(e.to_string()))?;
    migrations::migrate(&mut arquivo);
    arquivo
        .to_project()
        .map_err(PersistenceError::InvalidProject)
}
