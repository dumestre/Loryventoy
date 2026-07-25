//! DSL de autoramento do projeto inteiro (linguagem de "script" declarativo).
//!
//! Descreve o app TODO em texto: `project`, `canvas`, `scene`, `layer`,
//! `shape`, `text`, `pen` e as conexões `edge id.porto -> id.porto`. O código
//! de um nó `pen` vive num bloco `codigo { ... }` (PenDSL puro, passado cru ao
//! `crate::dsl`).
//!
//! Exemplo:
//! ```text
//! project "Video" { width 1920 height 1080 fps 30 duration 10 background #1e1e26 }
//! scene s1 { name "Cena 1" opacity 1.0 }
//! layer l1 { scene s1 name "Formas" }
//! shape sh1 { scene s1 type rect pos 960 540 size 300 200 color #eb9678 }
//! pen p1 {
//!   scene s1 pos 960 540 stroke 3 fill on
//!   codigo {
//!     repeat 5 { let a = i*72 line (cos(a)*200) (sin(a)*200) } close fill on
//!   }
//! }
//! edge l1.Formas -> sh1.Layer
//! edge l1.Formas -> p1.Layer
//! edge sh1.out -> master.in
//! edge p1.out -> master.in
//! ```

use eframe::egui::Color32;

// ----------------------------------------------------------------- AST

#[derive(Debug, Clone)]
pub enum TopLevel {
    Project(ProjectBlock),
    Node(NodeDef),
    Edge(EdgeDef),
}

#[derive(Debug, Clone, Default)]
pub struct ProjectBlock {
    pub nome: Option<String>,
    pub largura: Option<f32>,
    pub altura: Option<f32>,
    pub fps: Option<f32>,
    pub duracao: Option<f32>,
    pub fundo: Option<Color32>,
}

#[derive(Debug, Clone)]
pub struct NodeDef {
    pub tipo: String,
    pub id: String,
    pub campos: Vec<(String, Expr)>,
    /// código PenDSL cru (só para o nó Pen)
    pub codigo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EdgeDef {
    pub de: String,
    pub saida: String,
    pub para: String,
    pub entrada: String,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Num(f32),
    Str(String),
    Hex(String),
    Vec2(f32, f32),
}

impl Expr {
    pub fn as_num(&self) -> f32 {
        match self {
            Expr::Num(n) => *n,
            Expr::Vec2(a, _) => *a,
            _ => 0.0,
        }
    }
    pub fn as_hex(&self) -> Color32 {
        match self {
            Expr::Hex(h) => hex_para_cor(h),
            _ => Color32::WHITE,
        }
    }
    pub fn as_str(&self) -> String {
        match self {
            Expr::Str(s) => s.clone(),
            Expr::Hex(h) => h.clone(),
            Expr::Num(n) => n.to_string(),
            Expr::Vec2(a, b) => format!("{a} {b}"),
        }
    }
    pub fn as_codigo(&self) -> String {
        match self {
            Expr::Str(s) => s.clone(),
            other => other.as_str(),
        }
    }
}

/// Converte `#rrggbb` ou `#rgb` em `Color32`.
pub fn hex_para_cor(h: &str) -> Color32 {
    let h = h.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(255);
    Color32::from_rgb(r, g, b)
}

// ----------------------------------------------------------------- Erros

#[derive(Debug, Clone)]
pub enum ScriptError {
    Parse { msg: String, linha: usize },
    Apply(String),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::Parse { msg, linha } => write!(f, "linha {linha}: {msg}"),
            ScriptError::Apply(m) => write!(f, "{m}"),
        }
    }
}

// ----------------------------------------------------------------- Parser

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
            // comentário: apenas linhas cujo primeiro caractere (após
            // espaços) é '#'. Isso evita conflito com cores hexadecimais
            // (ex.: #1e1e26), que são tokens válidos.
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
            // símbolos de 2 caracteres
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
                // número: dígitos + uma parte decimal
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
                // se o primeiro char não casa, é um símbolo simples
                if end == 0 {
                    &rest[..1]
                } else {
                    &rest[..end]
                }
            } else {
                // símbolo simples (inclui '.', '-', '=', etc.)
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

    fn parse_program(&mut self) -> Result<Vec<TopLevel>, ScriptError> {
        let mut out = Vec::new();
        while let Some(tok) = self.proximo() {
            match tok {
                "project" => out.push(TopLevel::Project(self.parse_project()?)),
                "canvas" | "scene" | "layer" | "shape" | "text" | "pen" => {
                    out.push(TopLevel::Node(self.parse_node(tok)?))
                }
                "edge" => out.push(TopLevel::Edge(self.parse_edge()?)),
                other => return Err(self.err(format!("palavra-chave desconhecida '{other}'"))),
            }
        }
        Ok(out)
    }

    fn parse_project(&mut self) -> Result<ProjectBlock, ScriptError> {
        let mut p = ProjectBlock::default();
        if let Some(t) = self.proximo() {
            if t.starts_with('"') {
                p.nome = Some(t[1..t.len().saturating_sub(1)].to_string());
            } else {
                self.devolver();
            }
        }
        self.ate_bloco(|campo, val| match campo {
            "width" => p.largura = Some(val.as_num()),
            "height" => p.altura = Some(val.as_num()),
            "fps" => p.fps = Some(val.as_num()),
            "duration" => p.duracao = Some(val.as_num()),
            "background" => p.fundo = Some(val.as_hex()),
            _ => {}
        })?;
        Ok(p)
    }

    fn parse_node(&mut self, kw: &str) -> Result<NodeDef, ScriptError> {
        let id = self
            .proximo()
            .ok_or_else(|| self.err("esperado id do nó após tipo"))?
            .to_string();
        let mut campos = Vec::new();
        let mut codigo = None;
        self.ate_bloco(|campo, val| {
            if campo == "codigo" {
                codigo = Some(val.as_codigo());
            } else {
                campos.push((campo.to_string(), val));
            }
        })?;
        Ok(NodeDef {
            tipo: kw.to_string(),
            id,
            campos,
            codigo,
        })
    }

    fn parse_edge(&mut self) -> Result<EdgeDef, ScriptError> {
        let de = self
            .proximo()
            .ok_or_else(|| self.err("esperado nó de origem"))?
            .to_string();
        // porta de saída: '.nome' (o ponto é um símbolo à parte)
        let saida = self.le_porto()?;
        if self.proximo() != Some("->") {
            return Err(self.err("esperado '->' na conexão"));
        }
        let para = self
            .proximo()
            .ok_or_else(|| self.err("esperado nó de destino"))?
            .to_string();
        let entrada = self.le_porto()?;
        Ok(EdgeDef {
            de,
            saida,
            para,
            entrada,
        })
    }

    /// Lê uma referência de porto: um ponto `.` seguido do nome (ex.: `.out`).
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
            // sem ponto: usa o token direto (ex.: "out")
            Ok(t.trim_start_matches('.').to_string())
        }
    }

    /// Número de linhas vazias consecutivas a partir da linha atual
    /// (sem avançar o parser). Usado para aceitar "duas ou mais linhas em
    /// branco" como fim de um bloco de objeto, além da chave `}`.
    fn vazias_consecutivas(&self) -> usize {
        let mut n = 0;
        let mut li = self.li;
        while li < self.linhas.len() && self.linhas[li].trim().is_empty() {
            n += 1;
            li += 1;
        }
        n
    }

    /// Consome um bloco `{ ... }`, chamando `f(campo, valor)`. O bloco
    /// termina com `}` OU com duas ou mais linhas em branco consecutivas
    /// (distância entre objetos no script).
    fn ate_bloco(&mut self, mut f: impl FnMut(&str, Expr)) -> Result<(), ScriptError> {
        if self.proximo() != Some("{") {
            return Err(self.err("esperado '{'"));
        }
        loop {
            // Fim por distância: duas ou mais linhas vazias seguidas.
            if self.vazias_consecutivas() >= 2 {
                return Ok(());
            }
            let tok = self.proximo().ok_or_else(|| self.err("bloco não fechado"))?;
            if tok == "}" {
                break;
            }
            if tok == "codigo" {
                let bloco = self.bloco_texto()?;
                f("codigo", Expr::Str(bloco));
                continue;
            }
            if tok == "color" || tok == "colour" {
                let cor = self.le_cor()?;
                f(tok, cor);
                continue;
            }
            let valor = self.le_valor()?;
            f(tok, valor);
        }
        Ok(())
    }

    /// Lê o valor de um campo `color`: aceita `#rrggbb` OU `r g b`
    /// OU `r g b a` (canais 0..1). Números são convertidos para `Expr::Hex`
    /// para reaproveitar `as_hex()`. Isso evita que o 3º/4º número "vaze" e
    /// desalinhe o resto do bloco.
    fn le_cor(&mut self) -> Result<Expr, ScriptError> {
        let t = self.proximo().ok_or_else(|| self.err("esperado cor"))?;
        if t.starts_with('#') {
            return Ok(Expr::Hex(t.to_string()));
        }
        // primeiro token não é hex: tenta ler 3 números (rgb 0..1)
        let r = t
            .parse::<f32>()
            .map_err(|_| self.err("color espera #hex ou 'r g b' (0..1)"))?;
        let g = self
            .proximo()
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or_else(|| self.err("color: faltou o canal verde"))?;
        let b = self
            .proximo()
            .and_then(|s| s.parse::<f32>().ok())
            .ok_or_else(|| self.err("color: faltou o canal azul"))?;
        // 4º número (alpha) é opcional: espia sem consumir se não for número
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

    /// Lê um valor: número, string, hex, ou par de números (vec2).
    fn le_valor(&mut self) -> Result<Expr, ScriptError> {
        let t = self.proximo().ok_or_else(|| self.err("esperado valor"))?;
        if t.starts_with('"') {
            return Ok(Expr::Str(t[1..t.len().saturating_sub(1)].to_string()));
        }
        if t.starts_with('#') {
            return Ok(Expr::Hex(t.to_string()));
        }
        if let Ok(n) = t.replace('-', "").parse::<f32>() {
            let save_li = self.li;
            let save_ci = self.ci;
            if let Some(t2) = self.proximo() {
                if t2.replace('-', "").parse::<f32>().is_ok() {
                    return Ok(Expr::Vec2(n, t2.parse::<f32>().unwrap()));
                } else {
                    self.li = save_li;
                    self.ci = save_ci;
                }
            }
            return Ok(Expr::Num(n));
        }
        Ok(Expr::Str(t.to_string()))
    }

    /// Consome um bloco `{ ... }` e retorna o conteúdo como texto puro.
    fn bloco_texto(&mut self) -> Result<String, ScriptError> {
        if self.proximo() != Some("{") {
            return Err(self.err("esperado '{' após codigo"));
        }
        let mut texto = String::new();
        let mut prof = 1i32;
        loop {
            if self.li >= self.linhas.len() {
                return Err(self.err("bloco codigo não fechado"));
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
}

/// Parseia um script DSL de projeto em uma lista de comandos de alto nível.
pub fn parse_script(codigo: &str) -> Result<Vec<TopLevel>, ScriptError> {
    let mut p = Parser::new(codigo);
    p.parse_program()
}

/// Mapa de tipo de nó (string da DSL) -> `TipoNo`.
pub fn tipo_da_dsl(s: &str) -> Option<crate::nodes::TipoNo> {
    match s {
        "canvas" => Some(crate::nodes::TipoNo::Canvas),
        "scene" => Some(crate::nodes::TipoNo::Cena),
        "layer" => Some(crate::nodes::TipoNo::Layer),
        "shape" => Some(crate::nodes::TipoNo::Shape),
        "text" => Some(crate::nodes::TipoNo::Texto),
        "pen" => Some(crate::nodes::TipoNo::Pen),
        "noise" | "ruido" => Some(crate::nodes::TipoNo::Ruido),
        "anim" | "animacao" | "animation" => Some(crate::nodes::TipoNo::Anim),
        _ => None,
    }
}

/// Resolve o índice de um porto pelo nome para um dado `TipoNo`.
/// `out=true` busca nas saídas; `false` nas entradas. Aceita "out"/"in"
/// como atalho do índice 0, além de aliases curtos (pos, color, rot...).
pub fn indice_porto(tipo: crate::nodes::TipoNo, nome: &str, out: bool) -> Option<usize> {
    use crate::nodes::portos;
    let spec = portos(tipo);
    if nome == "out" || nome == "in" {
        return Some(0);
    }
    let nome_real = alias_porto(nome);
    if out {
        spec.saidas
            .iter()
            .position(|p| p.nome == nome_real || p.nome == nome)
    } else {
        spec.entradas
            .iter()
            .position(|p| p.nome == nome_real || p.nome == nome)
    }
}

/// Traduz um nome curto da DSL para o nome de porto real (em português).
fn alias_porto(nome: &str) -> String {
    let real = match nome {
        "pos" | "position" => "Posição",
        "size" => "Largura",
        "width" => "Largura",
        "height" => "Altura",
        "rot" | "rotation" => "Rotação",
        "color" | "colour" => "Cor",
        "canvas" => "Canvas",
        "scene" => "Cena",
        "pen" => "Pen",
        "layer" => "Layer",
        _ => nome,
    };
    real.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCRIPT: &str = "\
project \"Demo\" { width 1920 height 1080 fps 30 duration 8 background #1e1e26 }
scene s1 { name \"Cena 1\" opacity 1.0 }
layer l1 { scene s1 name \"Formas\" }
shape sh1 { scene s1 type star pos 960 540 size 300 300 color #eb9678 }
pen p1 {
  scene s1 pos 960 540 stroke 3 fill on
  codigo {
    repeat 5 { line (cos(i)*10) (sin(i)*10) }
  }
}
edge l1.Formas -> sh1.Layer
edge l1.Formas -> p1.Layer
edge sh1.out -> master.in
edge p1.out -> master.in
";

    #[test]
    fn color_rgb_nao_desalinha_bloco() {
        // `color r g b` (3 números) deve ser totalmente consumido, sem deixar
        // o 3º número vazar e quebrar o bloco `codigo { }` seguinte.
        let script = "\
pen p1 {
  scene s1
  color 1 0.2 0.4
  codigo {
    let s = 12
    fill on
    close
  }
}
";
        let prog = parse_script(script).expect("parseia");
        let n = prog
            .iter()
            .find_map(|tl| match tl {
                TopLevel::Node(n) => Some(n),
                _ => None,
            })
            .expect("tem nó pen");
        let code = n.codigo.clone().expect("codigo extraído");
        assert!(code.contains("fill on"), "código: {code:?}");
        crate::dsl::Program::parse(&code).expect("PenDSL válido");
    }

    #[test]
    fn parse_script_completo() {
        let prog = parse_script(SCRIPT).expect("script deve parsear");
        let mut projetos = 0;
        let mut nos = 0;
        let mut edges = 0;
        for tl in &prog {
            match tl {
                TopLevel::Project(_) => projetos += 1,
                TopLevel::Node(n) => {
                    nos += 1;
                    if n.id == "p1" {
                        assert!(n.codigo.is_some());
                        assert_eq!(n.campos.len(), 4);
                    }
                }
                TopLevel::Edge(_) => edges += 1,
            }
        }
        assert_eq!(projetos, 1);
        assert_eq!(nos, 4);
        assert_eq!(edges, 4);
    }

    #[test]
    fn erro_palavra_chave() {
        let r = parse_script("foo bar { }");
        assert!(matches!(r, Err(ScriptError::Parse { .. })));
    }

    #[test]
    fn porta_resolve() {
        use crate::nodes::TipoNo;
        assert_eq!(indice_porto(TipoNo::Cena, "out", true), Some(0));
        assert_eq!(indice_porto(TipoNo::Cena, "in", false), Some(0));
        assert!(indice_porto(TipoNo::Shape, "pos", false).is_some());
        assert!(indice_porto(TipoNo::Shape, "xyz", false).is_none());
    }

    #[test]
    fn hex_cor() {
        let c = hex_para_cor("#eb9678");
        assert_eq!(c, eframe::egui::Color32::from_rgb(0xeb, 0x96, 0x78));
    }
}

