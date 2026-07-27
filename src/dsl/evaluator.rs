//! Avaliadores de alto nível para Project DSL e Patch DSL.
//! Estes são apenas re-exports convenientes das funções em `application.rs`.

pub use super::application::{Application, aplicar_patch, aplicar_script};
pub use crate::dsl::project_dsl::ScriptError;