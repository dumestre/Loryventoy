//! Módulo de linguagens do Movimento.
//!
//! - `pen`: a mini-linguagem procedural do nó Pen (reexportada aqui, então
//!   `crate::dsl::Program`, `crate::dsl::PathCmd` etc. continuam válidos).
//! - `project_dsl`: DSL declarativa que descreve o projeto inteiro.
//! - `patch_dsl`: DSL de edição incremental (não destrutiva) do projeto.
//! - `application`: trait Application para desacoplar avaliação da DSL do GraphPanel.
//! - `evaluator`: avaliadores de Project DSL e Patch DSL.

mod pen;
pub use pen::*;

pub mod project_dsl;
pub mod patch_dsl;
pub mod application;
pub mod evaluator;
