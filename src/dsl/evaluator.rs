//! Avaliadores de alto nível para Project DSL e Patch DSL.
//! Estes são apenas re-exports convenientes das funções em `application.rs`.

pub use super::application::aplicar_script;
#[allow(unused_imports)] // re-export reservado para integração patch DSL
pub use super::application::aplicar_patch;
