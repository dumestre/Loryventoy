// Código pronto, aguardando a UI/IA que chamará `aplicar_patch`. Silencia
// warnings de "never used" até a integração.
#![allow(dead_code)]
//! DSL de *patch* (edição incremental e NÃO destrutiva) do projeto.
//!
//! Diferente do `project_dsl` (que descreve o projeto inteiro e reconstrói o
//! grafo do zero), esta linguagem descreve **operações** aplicadas sobre o
//! grafo existente. Cada linha é um comando que toca apenas o alvo indicado,
//! preservando tudo o mais. É o formato pensado para a IA emitir: barato
//! (poucas linhas por pedido), seguro (transacional + desfazível) e
//! não destrutivo.
//!
//! Comandos:
//! ```text
//! add pen p1 { scene s1 pos 960 540 stroke 3 }   # cria nó com id "p1"
//! set p1.stroke_color #ff3366                     # muda um campo
//! set p1.codigo { move 0 0  line 100 0 }          # muda o código PenDSL
//! remove p1                                       # remove o nó
//! connect p1.out -> master.in                     # cria conexão
//! disconnect p1.out -> master.in                  # remove conexão
//! ```
//!
//! Os `id`s referenciam nós pelo nome DSL persistente (veja
//! `GraphPanel::dsl_ids`). Nós padrão têm ids fixos: `canvas`, `scene` (a
//! primeira cena) e `master`.

use crate::dsl::project_dsl::{Expr, ScriptError};

// ----------------------------------------------------------------- AST

#[derive(Debug, Clone)]
pub enum PatchCmd {
    /// `add <tipo> <id> { campos }`
    Add {
        tipo: String,
        id: String,
        campos: Vec<(String, Expr)>,
        codigo: Option<String>,
    },
    /// `set <id>.<campo> <valor>` (campo "codigo" usa `codigo`)
    Set {
        id: String,
        campo: String,
        valor: Expr,
        codigo: Option<String>,
    },
    /// `remove <id>`
    Remove { id: String },
    /// `connect <id>.<porto> -> <id>.<porto>`
    Connect(Conexao),
    /// `disconnect <id>.<porto> -> <id>.<porto>`
    Disconnect(Conexao),
}

#[derive(Debug, Clone)]
pub struct Conexao {
    pub de: String,
    pub saida: String,
    pub para: String,
    pub entrada: String,
}

// ----------------------------------------------------------------- Parser
//
// Reusa o tokenizador do `project_dsl` para não duplicar regras (números,
// strings, hex, `->`, blocos `{ }`, comentários `#`).

struct Parser<'a> {
    linhas: Vec<&'a str>,
    li: usize,
    ci: usize,
}

impl<'a> Parser<'a> {
    fn new(codigo: &'a str) -> Self {
        Self {
            linhas: codigo.lines().collect(),
            li: 0,
            ci: 0,
        }
    }

    fn err(&self, msg: impl Into<String>) -> ScriptError {
        ScriptError::Parse {
            msg: msg.into(),
            linha: self.li + 1,
        }
    }

    fn linha_atual(&self) -> &'a str {
        self.linhas.get(self.li).copied().unwrap_or("")
    }

    fn proximo(&mut self) -> Option<&'a str> {
        loop {
            let linha = self.linha_atual();
            let rest = &linha[self.ci..];
            let trimmed = rest.trim_start();
            let consumido = rest.len() - trimmed.len();
            if consumido > 0 {
                self.ci += consumido;
            }
            let linha = self.linha_atual();
            let rest = &linha[self.ci..];
            if linha.trim_start().starts_with('#') {
                self.li += 1;
                self.ci = 0;
                continue;
            }
            if rest.is_empty() {
                if self.li + 1 >= self.linhas.len() {
                    return None;
                }
                self.li += 1;
                self.ci = 0;
                continue;
            }
            let bytes = rest.as_bytes();
            let c0 = bytes[0] as char;
            if rest.starts_with("->") {
                self.ci += 2;
                return Some("->");
            }
            let is_dig = c0.is_ascii_digit();
            let is_num_inicio = is_dig
                || (c0 == '.' && bytes.len() > 1 && (bytes[1] as char).is_ascii_digit())
                || (c0 == '-' && bytes.len() > 1 && (bytes[1] as char).is_ascii_digit());
            let tok: &'a str = if c0 == '"' {
                let fim = rest[1..].find('"').map(|i| i + 1).unwrap_or(rest.len());
                &rest[..fim + 1]
            } else if is_num_inicio {
                let mut end = 0;
                if bytes[end] as char == '-' {
                    end += 1;
                }
                while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                    end += 1;
                }
                if end < bytes.len() && (bytes[end] as char) == '.' {
                    end += 1;
                    while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                        end += 1;
                    }
                }
                &rest[..end]
            } else if c0.is_alphanumeric() || c0 == '_' || c0 == '#' {
                let end = rest
                    .find(|ch: char| !(ch.is_alphanumeric() || ch == '_' || ch == '#'))
                    .unwrap_or(rest.len());
                if end == 0 {
                    &rest[..1]
                } else {
                    &rest[..end]
                }
            } else {
                &rest[..1]
            };
            self.ci += tok.len();
            return Some(tok);
        }
    }

    fn devolver(&mut self) {
        let linha = self.linha_atual();
        self.ci = linha.len() - linha.trim_start().len();
    }

    /// Lê um valor: número, string, hex, ou par de números (vec2).
    fn le_valor(&mut self) -> Result<Expr, ScriptError> {
        let t = self.proximo().ok_or_else(|| self.err("esperado valor"))?;
        if t.starts_with('"') {
            return Ok(Expr::Str(t[1..t.len().saturating_sub(1)].to_string()));
        }
        if t.starts_with('#') {
            return Ok(Expr::Hex(t.to_string()));
        }
        if t.parse::<f32>().is_ok() {
            let n = t.parse::<f32>().unwrap();
            let save_li = self.li;
            let save_ci = self.ci;
            if let Some(t2) = self.proximo() {
                if let Ok(n2) = t2.parse::<f32>() {
                    return Ok(Expr::Vec2(n, n2));
                } else {
                    self.li = save_li;
                    self.ci = save_ci;
                }
            }
            return Ok(Expr::Num(n));
        }
        Ok(Expr::Str(t.to_string()))
    }

    /// Lê `color`: aceita `#hex` OU `r g b` OU `r g b a` (0..1).
    fn le_cor(&mut self) -> Result<Expr, ScriptError> {
        let t = self.proximo().ok_or_else(|| self.err("esperado cor"))?;
        if t.starts_with('#') {
            return Ok(Expr::Hex(t.to_string()));
        }
        let r = t
            .parse::<f32>()
            .map_err(|_| self.err("cor espera #hex ou 'r g b' (0..1)"))?;
        let g = self
            .proximo()
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or_else(|| self.err("cor: faltou o canal verde"))?;
        let b = self
            .proximo()
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or_else(|| self.err("cor: faltou o canal azul"))?;
        let save_li = self.li;
        let save_ci = self.ci;
        if let Some(t4) = self.proximo() {
            if t4.parse::<f32>().is_err() {
                self.li = save_li;
                self.ci = save_ci;
            }
        }
        let to255 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let hex = format!("#{:02x}{:02x}{:02x}", to255(r), to255(g), to255(b));
        Ok(Expr::Hex(hex))
    }

    /// Consome um bloco `{ ... }` retornando o conteúdo como texto puro.
    fn bloco_texto(&mut self) -> Result<String, ScriptError> {
        if self.proximo() != Some("{") {
            return Err(self.err("esperado '{'"));
        }
        let mut texto = String::new();
        let mut prof = 1i32;
        loop {
            if self.li >= self.linhas.len() {
                return Err(self.err("bloco não fechado"));
            }
            let linha = self.linha_atual();
            let rest = &linha[self.ci..];
            if rest.is_empty() {
                if !texto.is_empty() {
                    texto.push('\n');
                }
                self.li += 1;
                self.ci = 0;
                continue;
            }
            let ch = rest.chars().next().unwrap();
            self.ci += ch.len_utf8();
            match ch {
                '{' => {
                    prof += 1;
                    texto.push(ch);
                }
                '}' => {
                    prof -= 1;
                    if prof == 0 {
                        break;
                    }
                    texto.push(ch);
                }
                _ => texto.push(ch),
            }
        }
        Ok(texto)
    }

    /// Lê `id` ou `id.campo`; devolve `(id, Option<campo>)`.
    fn le_ref(&mut self) -> Result<(String, Option<String>), ScriptError> {
        let id = self
            .proximo()
            .ok_or_else(|| self.err("esperado id do nó"))?
            .to_string();
        // um ponto imediato indica "id.campo/porto"
        let save_li = self.li;
        let save_ci = self.ci;
        if self.proximo() == Some(".") {
            let campo = self
                .proximo()
                .ok_or_else(|| self.err("esperado nome após '.'"))?
                .to_string();
            Ok((id, Some(campo)))
        } else {
            self.li = save_li;
            self.ci = save_ci;
            Ok((id, None))
        }
    }

    /// Lê uma referência de porto `.porto` (o ponto é token à parte).
    fn le_porto(&mut self) -> Result<String, ScriptError> {
        let t = self
            .proximo()
            .ok_or_else(|| self.err("esperado '.porto'"))?;
        if t == "." {
            let nome = self
                .proximo()
                .ok_or_else(|| self.err("esperado nome do porto após '.'"))?;
            Ok(nome.to_string())
        } else {
            Ok(t.trim_start_matches('.').to_string())
        }
    }

    fn parse_conexao(&mut self) -> Result<Conexao, ScriptError> {
        let de = self
            .proximo()
            .ok_or_else(|| self.err("esperado nó de origem"))?
            .to_string();
        let saida = self.le_porto()?;
        if self.proximo() != Some("->") {
            return Err(self.err("esperado '->' na conexão"));
        }
        let para = self
            .proximo()
            .ok_or_else(|| self.err("esperado nó de destino"))?
            .to_string();
        let entrada = self.le_porto()?;
        Ok(Conexao {
            de,
            saida,
            para,
            entrada,
        })
    }

    /// Consome um bloco `{ campo valor ... }` (campos de um `add`), chamando
    /// `f`. Difere do project_dsl por NÃO terminar por linhas em branco: só
    /// por `}`. Retorna o código PenDSL opcional (bloco `codigo { }`).
    fn parse_campos(&mut self) -> Result<(Vec<(String, Expr)>, Option<String>), ScriptError> {
        if self.proximo() != Some("{") {
            return Err(self.err("esperado '{'"));
        }
        let mut campos = Vec::new();
        let mut codigo = None;
        loop {
            let tok = self
                .proximo()
                .ok_or_else(|| self.err("bloco não fechado"))?;
            if tok == "}" {
                break;
            }
            if tok == "codigo" {
                codigo = Some(self.bloco_texto()?);
                continue;
            }
            if tok == "color" || tok == "colour" || tok.ends_with("_color") {
                let cor = self.le_cor()?;
                campos.push((tok.to_string(), cor));
                continue;
            }
            let valor = self.le_valor()?;
            campos.push((tok.to_string(), valor));
        }
        Ok((campos, codigo))
    }

    fn parse_program(&mut self) -> Result<Vec<PatchCmd>, ScriptError> {
        let mut out = Vec::new();
        while let Some(tok) = self.proximo() {
            match tok {
                "add" => {
                    let tipo = self
                        .proximo()
                        .ok_or_else(|| self.err("esperado tipo após 'add'"))?
                        .to_string();
                    let id = self
                        .proximo()
                        .ok_or_else(|| self.err("esperado id após tipo"))?
                        .to_string();
                    let (campos, codigo) = self.parse_campos()?;
                    out.push(PatchCmd::Add {
                        tipo,
                        id,
                        campos,
                        codigo,
                    });
                }
                "set" => {
                    let (id, campo) = self.le_ref()?;
                    let campo = campo.ok_or_else(|| self.err("set espera 'id.campo'"))?;
                    if campo == "codigo" {
                        let code = self.bloco_texto()?;
                        out.push(PatchCmd::Set {
                            id,
                            campo,
                            valor: Expr::Num(0.0),
                            codigo: Some(code),
                        });
                    } else {
                        let valor =
                            if campo == "color" || campo == "colour" || campo.ends_with("_color") {
                                self.le_cor()?
                            } else {
                                self.le_valor()?
                            };
                        out.push(PatchCmd::Set {
                            id,
                            campo,
                            valor,
                            codigo: None,
                        });
                    }
                }
                "remove" | "delete" => {
                    let id = self
                        .proximo()
                        .ok_or_else(|| self.err("esperado id após 'remove'"))?
                        .to_string();
                    out.push(PatchCmd::Remove { id });
                }
                "connect" => out.push(PatchCmd::Connect(self.parse_conexao()?)),
                "disconnect" => out.push(PatchCmd::Disconnect(self.parse_conexao()?)),
                other => {
                    let _ = self.devolver();
                    return Err(self.err(format!("comando desconhecido '{other}'")));
                }
            }
        }
        Ok(out)
    }
}

/// Parseia um patch DSL em uma lista de comandos.
pub fn parse_patch(codigo: &str) -> Result<Vec<PatchCmd>, ScriptError> {
    let mut p = Parser::new(codigo);
    p.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_add_set_connect() {
        let src = "\
add pen p1 { scene s1 pos 960 540 stroke 3 }
set p1.stroke_color #ff3366
set p1.codigo {
  move 0 0
  line 100 0
}
connect p1.out -> master.in
disconnect s1.out -> master.in
remove old2
";
        let cmds = parse_patch(src).expect("deve parsear");
        assert_eq!(cmds.len(), 6);
        match &cmds[0] {
            PatchCmd::Add {
                tipo, id, campos, ..
            } => {
                assert_eq!(tipo, "pen");
                assert_eq!(id, "p1");
                assert!(campos.iter().any(|(k, _)| k == "scene"));
            }
            _ => panic!("esperado Add"),
        }
        match &cmds[1] {
            PatchCmd::Set { id, campo, .. } => {
                assert_eq!(id, "p1");
                assert_eq!(campo, "stroke_color");
            }
            _ => panic!("esperado Set"),
        }
        match &cmds[2] {
            PatchCmd::Set { campo, codigo, .. } => {
                assert_eq!(campo, "codigo");
                assert!(codigo.as_deref().unwrap().contains("line 100 0"));
            }
            _ => panic!("esperado Set codigo"),
        }
        assert!(matches!(&cmds[3], PatchCmd::Connect(_)));
        assert!(matches!(&cmds[4], PatchCmd::Disconnect(_)));
        assert!(matches!(&cmds[5], PatchCmd::Remove { .. }));
    }

    #[test]
    fn set_color_rgb() {
        let cmds = parse_patch("set p1.fill_color 1 0 0").unwrap();
        match &cmds[0] {
            PatchCmd::Set { campo, valor, .. } => {
                assert_eq!(campo, "fill_color");
                assert!(matches!(valor, Expr::Hex(_)));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn comando_invalido() {
        assert!(matches!(
            parse_patch("foo p1"),
            Err(ScriptError::Parse { .. })
        ));
    }
}
