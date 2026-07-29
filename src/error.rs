//! Erros unificados da aplicação.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("erro de formato: {0}")]
    Parse(String),

    #[error("projeto inválido: {0}")]
    InvalidProject(String),

    #[error("erro de DSL: {0}")]
    Dsl(String),

    #[error("linha {linha}: {msg}")]
    DslParse { msg: String, linha: usize },

    #[error("erro de exportação: {0}")]
    Export(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_converte_para_app_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "arquivo não encontrado");
        let app_err = AppError::from(io_err);
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    fn to_string_exibe_mensagem_clara() {
        let e = AppError::Parse("JSON malformado".to_string());
        assert_eq!(e.to_string(), "erro de formato: JSON malformado");
    }

    #[test]
    fn dsl_parse_inclui_linha() {
        let e = AppError::DslParse {
            msg: "esperado '{'".to_string(),
            linha: 5,
        };
        assert_eq!(e.to_string(), "linha 5: esperado '{'");
    }

    #[test]
    fn export_exibe_mensagem() {
        let e = AppError::Export("disco cheio".to_string());
        assert_eq!(e.to_string(), "erro de exportação: disco cheio");
    }
}
