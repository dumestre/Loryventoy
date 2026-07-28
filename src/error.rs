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

    #[error("erro de exportação: {0}")]
    Export(String),

    #[error("erro de avaliação: {0}")]
    Evaluation(String),
}

impl AppError {
    pub fn is_validation(&self) -> bool {
        matches!(self, AppError::InvalidProject(_))
    }
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
    fn invalid_project_eh_erro_de_validacao() {
        let e = AppError::InvalidProject("campo ausente".to_string());
        assert!(e.is_validation());
    }

    #[test]
    fn erro_nao_eh_validacao() {
        let e = AppError::Dsl("sintaxe inválida".to_string());
        assert!(!e.is_validation());
    }

    #[test]
    fn to_string_exibe_mensagem_clara() {
        let e = AppError::Parse("JSON malformado".to_string());
        assert_eq!(e.to_string(), "erro de formato: JSON malformado");
    }
}