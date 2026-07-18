//! Mini-linguagem procedural do nó Pen.
//!
//! O usuário escreve um programa linha a linha, no estilo "caneta" (move/line/
//! bezier/...). O código é parseado para uma [`Program`] (AST) e avaliado por
//! frame com o tempo `t`, produzindo uma lista de [`PathCmd`] (pontos crus +
//! estilo) que o preview converte em `eframe::egui::Shape`.
//!
//! Sintaxe (coordenadas em unidades de projeto, centro em largura/2,altura/2):
//! ```text
//! # comentário
//! let nome = expr
//! move x y
//! line x y
//! rect x y w h
//! circle x y r
//! bezier cx1 cy1 cx2 cy2 x y
//! close
//! fill on | off
//! stroke w
//! color r g b            (0..1)
//! repeat n { ... }       (usa `i` = índice 0..n-1)
//! if cond { ... }         (cond != 0 executa; 0 pula)
//! if cond { ... } else { ... }
//! if cond { ... } else if cond2 { ... } else { ... }
//! for v in a..b { ... }    (v varre [a, b) com passo 1)
//! while cond { ... }       (repete enquanto cond != 0)
//! fn nome(a, b) { ... }    (define função; chamada por nome(a, b))
//! return expr             (retorna valor de dentro de função)
//! color nome | r g b | r g b a   (nome, rgb ou rgba 0..1)
//! text "string" x y [size]   (desenha texto; size em px de projeto opcional)
//! ```
//! Expressões suportam `+ - * / %`, lógica `and`/`or`, comparações
//! `> < >= <= == !=` (resultam em 1.0 ou 0.0), parênteses, variáveis, e
//! chamadas `cos, sin, tan, sqrt, abs, floor, noise`. `noise(x)` é 1D e
//! `noise(x,y)` é 2D. A variável implícita `t` é o tempo em segundos e `i`
//! é o índice do `repeat`.

use std::collections::HashMap;

use crate::procedural::GVec2;
use eframe::egui::Color32;

/// Comandos de path emitidos pela avaliação (geometria crua).
#[derive(Debug, Clone, PartialEq)]
pub enum PathCmd {
    Move(GVec2),
    Line(GVec2),
    Bezier(GVec2, GVec2, GVec2),
    Close,
    Fill(bool),
    Stroke(f32),
    /// Define AMBAS as cores (traço e preenchimento).
    Color(Color32),
    /// Define apenas a cor do traço (contorno).
    ColorStroke(Color32),
    /// Define apenas a cor do preenchimento.
    ColorFill(Color32),
    /// Desenha texto direto na caneta. O texto é rasterizado no preview/export
    /// usando a mesma fonte do nó Texto. `conteudo` é a string; `(x, y)` é o
    /// canto superior-esquerdo (ou centro, se alinhamento center) em coords de
    /// projeto; `tamanho` em px de projeto; `negrito`/`italico` flags;
    /// `cor` vem do estado de cor da caneta; `rotacao` em graus.
    Text {
        conteudo: String,
        x: f32,
        y: f32,
        tamanho: f32,
        negrito: bool,
        italico: bool,
        alinhamento: TextoAlinhamento,
        rotacao: f32,
        cor: Color32,
    },
}

/// Texto emitido pela caneta (`PathCmd::Text`), já com a cor resolvida, para
/// ser rasterizado pelo preview/export.
#[derive(Debug, Clone)]
pub struct PenText {
    pub conteudo: String,
    pub x: f32,
    pub y: f32,
    pub tamanho: f32,
    pub negrito: bool,
    pub italico: bool,
    pub alinhamento: TextoAlinhamento,
    pub rotacao: f32,
    pub cor: Color32,
}

/// Extrai os textos de uma lista de [`PathCmd`], convertendo para [`PenText`]
/// (já com a cor resolvida). Usado pelo preview e pelo export PNG.
pub fn extrair_textos(cmds: &[PathCmd]) -> Vec<PenText> {
    cmds.iter()
        .filter_map(|c| match c {
            PathCmd::Text {
                conteudo,
                x,
                y,
                tamanho,
                negrito,
                italico,
                alinhamento,
                rotacao,
                cor,
            } => Some(PenText {
                conteudo: conteudo.clone(),
                x: *x,
                y: *y,
                tamanho: *tamanho,
                negrito: *negrito,
                italico: *italico,
                alinhamento: *alinhamento,
                rotacao: *rotacao,
                cor: *cor,
            }),
            _ => None,
        })
        .collect()
}

/// Erros de parse/avaliação com localização.
#[derive(Debug, Clone)]
pub enum DslError {
    Parse { msg: String, linha: usize, coluna: usize },
    Eval { msg: String, linha: usize },
}

impl std::fmt::Display for DslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DslError::Parse { msg, linha, coluna } => {
                write!(f, "linha {linha}:{coluna}: {msg}")
            }
            DslError::Eval { msg, linha } => {
                if *linha > 0 {
                    write!(f, "linha {linha}: {msg}")
                } else {
                    write!(f, "{msg}")
                }
            }
        }
    }
}

// ------------------------------------------------------------------ AST

#[derive(Debug, Clone)]
pub enum Expr {
    Num(f32),
    Var(String),
    Bin(Box<Expr>, BinOp, Box<Expr>),
    Cmp(Box<Expr>, CmpOp, Box<Expr>),
    Call(String, Vec<Expr>),
    /// Acesso de campo: `expr.x` ou `expr.y` (componentes de um `vec2`).
    Field(Box<Expr>, String),
    /// Literal de string (usado por `ease(x, "tipo")` etc.).
    Str(String),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub enum CmpOp {
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

/// Alinhamento horizontal do texto desenhado pela caneta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextoAlinhamento {
    Left,
    Center,
    Right,
}

/// Um statement da linguagem, com a linha de origem (1-based) anotada pelo
/// parser para permitir reportar erros de avaliação localizados.
#[derive(Debug, Clone)]
pub struct Stmt {
    pub linha: usize,
    pub kind: StmtKind,
}

/// Valor da linguagem: escalar (`Num`) ou vetor 2D (`Vec`). Permite que
/// variáveis guardem `vec2(...)` e sejam acessadas por `.x`/`.y`.
#[derive(Debug, Clone, Copy)]
pub enum Valor {
    Num(f32),
    Vec(GVec2),
}

impl Valor {
    /// Extrai o escalar. Erro se for um vetor usado em contexto escalar.
    fn num(&self, linha: usize) -> Result<f32, DslError> {
        match self {
            Valor::Num(v) => Ok(*v),
            Valor::Vec(_) => Err(DslError::Eval {
                msg: "esperado número, mas a expressão é um vetor (use .x/.y)".to_string(),
                linha,
            }),
        }
    }
    /// Extrai o vetor. Erro se for um escalar usado em contexto de vetor.
    #[allow(dead_code)]
    fn vec(&self, linha: usize) -> Result<GVec2, DslError> {
        match self {
            Valor::Vec(v) => Ok(*v),
            Valor::Num(_) => Err(DslError::Eval {
                msg: "esperado vetor (vec2), mas a expressão é um número".to_string(),
                linha,
            }),
        }
    }
    /// Componente x (de vetor) ou o próprio escalar.
    fn comp_x(&self, _linha: usize) -> Result<f32, DslError> {
        match self {
            Valor::Num(v) => Ok(*v),
            Valor::Vec(v) => Ok(v.x),
        }
    }
    /// Componente y (de vetor) ou o próprio escalar.
    fn comp_y(&self, _linha: usize) -> Result<f32, DslError> {
        match self {
            Valor::Num(v) => Ok(*v),
            Valor::Vec(v) => Ok(v.y),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Let(String, Expr),
    Move(Expr, Expr),
    Line(Expr, Expr),
    Rect(Expr, Expr, Expr, Expr),
    Circle(Expr, Expr, Expr),
    Bezier(Expr, Expr, Expr, Expr, Expr, Expr),
    Close,
    Fill(bool),
    Stroke(Expr),
    Color(Vec<Expr>),
    StrokeColor(Vec<Expr>),
    FillColor(Vec<Expr>),
    /// Desenha texto: `text "str" x y [size] [align ...] [rot graus]`.
    /// A posição é o canto superior-esquerdo (ou centro, se `align center`).
    /// `size` (px de projeto) é opcional (padrão 48). Aceita `\n` (multilinha).
    Text {
        conteudo: String,
        x: Expr,
        y: Expr,
        tamanho: Expr,
        negrito: bool,
        italico: bool,
        alinhamento: TextoAlinhamento,
        rotacao: Expr,
    },
    /// Polígono regular de `n` lados e raio `r` centrado em (cx, cy).
    Polygon(Expr, Expr, Expr, Expr),
    /// Estrela de `n` pontas, raios `r1`/`r2`, centrada em (cx, cy).
    Star(Expr, Expr, Expr, Expr, Expr),
    /// Arco do ângulo `a0` ao `a1` (graus), raio `r`, centrado em (cx, cy).
    Arc(Expr, Expr, Expr, Expr, Expr),
    /// Retângulo com cantos arredondados (raio `r`) em (x, y, w, h).
    RoundRect(Expr, Expr, Expr, Expr, Expr),
    /// Grade de `cols`×`rows` pontos (círculos de raio `pr`) espaçados
    /// `w`×`h`, a partir de (x, y).
    Grid(Expr, Expr, Expr, Expr, Expr, Expr, Expr),
    /// Define o "ponto atual" sem desenhar (atalho de `move`). Útil para
    /// começar um caminho antes de `line_to`/`curve_to`.
    Point(Expr, Expr),
    /// Desenha uma linha do ponto atual até `(x, y)` (atalho de `line`).
    LineTo(Expr, Expr),
    /// Curva de Bézier do ponto atual até `(x, y)` com os dois controles.
    CurveTo(Expr, Expr, Expr, Expr, Expr, Expr),
    /// Translada o sistema de coordenadas em (x, y).
    Translate(Expr, Expr),
    /// Rotaciona o sistema de coordenadas em `ang` graus.
    Rotate(Expr),
    /// Escala o sistema de coordenadas por `s` (ou sx, sy).
    Scale(Expr, Expr),
    /// Salva o estado atual (transform + estilo) na pilha.
    Push,
    /// Restaura o estado salvo por `push`.
    Pop,
    /// Desenha uma "cobra" (linha serpenteante) iniciando em (x, y), com
    /// `length` total e `segments` segmentos, oscilando na direção perpendicular.
    Snake(Expr, Expr, Expr, Expr),
    Repeat(Expr, Vec<Stmt>),
    For(String, Expr, Expr, Expr, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    /// Define uma função: `fn nome(p1, p2) { ... }`. O corpo é armazenado e
    /// só executado quando a função é chamada via `Expr::Call`.
    Fn(String, Vec<String>, Vec<Stmt>),
    /// Retorna o valor de uma expressão de dentro de uma função, encerrando
    /// sua execução. Fora de função é um erro.
    Return(Expr),
    /// Atribuição direta: `nome = expr` (nome já deve existir ou será criado).
    Assign(String, Expr),
}

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl Program {
    pub fn parse(codigo: &str) -> Result<Program, DslError> {
        Parser::new(codigo).parse_program()
    }

    /// Avalia o programa no tempo `t`, com seed para `noise`. A variável
    /// implícita `progress` usa o ciclo padrão de 6s.
    pub fn eval(&self, t: f32, seed: u32) -> Result<Vec<PathCmd>, DslError> {
        self.eval_dur(t, seed, 6.0)
    }

    /// Igual a [`Program::eval`], mas permite informar a duração (segundos) do
    /// ciclo usada pela variável implícita `progress`.
    pub fn eval_dur(&self, t: f32, seed: u32, duracao: f32) -> Result<Vec<PathCmd>, DslError> {
        let mut ev = Eval::new(t, seed);
        ev.duracao = if duracao > 0.0 { duracao } else { 6.0 };
        if let Flow::Return(_) = ev.run(&self.stmts)? {
            return Err(DslError::Eval {
                msg: "return usado fora de uma função".to_string(),
                linha: 0,
            });
        }
        Ok(ev.cmds)
    }
}

// ------------------------------------------------------------------ Parser

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

    fn err(&self, msg: impl Into<String>) -> DslError {
        DslError::Parse {
            msg: msg.into(),
            linha: self.li + 1,
            coluna: self.ci + 1,
        }
    }

    fn linha_atual(&self) -> &'a str {
        self.linhas.get(self.li).copied().unwrap_or("")
    }

    /// Avança para o próximo token real (pula espaços e comentários).
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
            // Comentário: '#' no início da linha (após espaços) ou precedido de
            // espaço em qualquer ponto ("move 1 2  # nota"). Descarta o resto.
            // EXCEÇÃO: `#` seguido de dígitos hex válidos (3/6/8) é uma cor
            // (ex.: `color #ff5500`), e não comentário — retorna como token.
            let rest_trim = rest.trim_start();
            if rest_trim.starts_with('#') {
                let apos = &rest_trim[1..];
                let eh_cor = (apos.len() == 3 || apos.len() == 6 || apos.len() == 8)
                    && apos.bytes().all(|b| b.is_ascii_hexdigit());
                if eh_cor {
                    self.ci += rest.len() - rest_trim.len(); // posiciona no '#'
                    let tok = &rest_trim[..1 + apos.len()];
                    self.ci += tok.len();
                    return Some(tok);
                }
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
            // Operadores de 2 caracteres: .. <= >= == != (e o ponto sozinho
            // só conta se não for número; tratado abaixo).
            if bytes.len() >= 2 {
                let c1 = bytes[1] as char;
                let dois = match (c0, c1) {
                    ('.', '.') => true,
                    ('<', '=') => true,
                    ('>', '=') => true,
                    ('=', '=') => true,
                    ('!', '=') => true,
                    _ => false,
                };
                if dois {
                    let tok = &rest[..2];
                    self.ci += 2;
                    return Some(tok);
                }
            }
            // número (inclui decimais tipo 0.78 e .47, e negativos -5)
            let is_num_inicio = c0.is_ascii_digit()
                || (c0 == '.' && bytes.len() > 1 && (bytes[1] as char).is_ascii_digit())
                || (c0 == '-'
                    && bytes.len() > 1
                    && ((bytes[1] as char).is_ascii_digit()
                        || (bytes[1] as char == '.'
                            && bytes.len() > 2
                            && (bytes[2] as char).is_ascii_digit())));
            let tok: &'a str = if is_num_inicio {
                let mut end = 0;
                // sinal de menos (se houver)
                if bytes[end] as char == '-' {
                    end += 1;
                }
                // parte inteira
                while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                    end += 1;
                }
                // parte decimal: só consome o '.' se houver dígito em seguida
                // (evita confundir '..' de range com casa decimal).
                if end < bytes.len()
                    && (bytes[end] as char) == '.'
                    && end + 1 < bytes.len()
                    && (bytes[end + 1] as char).is_ascii_digit()
                {
                    end += 1;
                    while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
                        end += 1;
                    }
                }
                &rest[..end]
            } else if c0.is_alphanumeric() || c0 == '_' {
                let end = rest
                    .find(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
                    .unwrap_or(rest.len());
                &rest[..end]
            } else {
                &rest[..1]
            };
            self.ci += tok.len();
            let _ = &self.ci;
            return Some(tok);
        }
    }

    /// Lê uma string entre aspas duplas (`"..."`) a partir da posição atual.
    /// Suporta escape `\"`. Erro se a string não for fechada.
    fn parse_string(&mut self) -> Result<String, DslError> {
        // avança até a primeira aspa
        loop {
            let linha = self.linha_atual();
            let rest = &linha[self.ci..];
            let trimmed = rest.trim_start();
            if trimmed.starts_with('#') {
                self.li += 1;
                self.ci = 0;
                continue;
            }
            if trimmed.starts_with('"') {
                self.ci += (rest.len() - trimmed.len()) + 1; // pula a aspa inicial
                break;
            }
            if !trimmed.is_empty() {
                return Err(self.err("esperado string entre aspas após text"));
            }
            if self.li + 1 >= self.linhas.len() {
                return Err(self.err("string não fechada após text"));
            }
            self.li += 1;
            self.ci = 0;
        }
        let mut out = String::new();
        loop {
            if self.li >= self.linhas.len() {
                return Err(self.err("string não fechada após text"));
            }
            let linha = self.linha_atual();
            let rest = &linha[self.ci..];
            if rest.is_empty() {
                // string跨 linhas: junta com espaço (raro, mas tolerante)
                out.push(' ');
                self.li += 1;
                self.ci = 0;
                continue;
            }
            let bytes = rest.as_bytes();
            let c = bytes[0] as char;
            self.ci += 1;
            match c {
                '"' => break,
                '\\' if self.ci < linha.len() && linha.as_bytes()[self.ci] == b'"' => {
                    out.push('"');
                    self.ci += 1;
                }
                _ => out.push(c),
            }
        }
        Ok(out)
    }

    /// Lê uma string literal de expressão. Assume que a aspa inicial JÁ foi
    /// consumida (o `ci` aponta para o primeiro caractere do conteúdo).
    fn parse_string_expr(&mut self) -> Result<String, DslError> {
        let mut out = String::new();
        loop {
            if self.li >= self.linhas.len() {
                return Err(self.err("string não fechada"));
            }
            let linha = self.linha_atual();
            let rest = &linha[self.ci..];
            if rest.is_empty() {
                out.push(' ');
                self.li += 1;
                self.ci = 0;
                continue;
            }
            let bytes = rest.as_bytes();
            let c = bytes[0] as char;
            self.ci += 1;
            match c {
                '"' => break,
                '\\' if self.ci < linha.len() && linha.as_bytes()[self.ci] == b'"' => {
                    out.push('"');
                    self.ci += 1;
                }
                _ => out.push(c),
            }
        }
        Ok(out)
    }

    fn peek(&self) -> Option<String> {
        let mut clone = Parser {
            linhas: self.linhas.clone(),
            li: self.li,
            ci: self.ci,
        };
        clone.proximo().map(|s| s.to_string())
    }

    /// Verdadeiro se não há mais tokens na LINHA atual (só resta espaço em
    /// branco ou um comentário). Usado por comandos de aridade variável
    /// (como `color`) para não "vazar" para a linha seguinte.
    fn fim_de_linha(&self) -> bool {
        let linha = self.linha_atual();
        let rest = if self.ci <= linha.len() {
            &linha[self.ci..]
        } else {
            ""
        };
        let t = rest.trim_start();
        t.is_empty() || t.starts_with('#')
    }

    /// Lê os argumentos de um comando de cor: `nome`, ou `r g b`, ou `r g b a`.
    /// Para no FIM DA LINHA (ou em `}`/EOF) para não engolir o comando da
    /// linha seguinte como argumento extra. Usado por `color`,
    /// `stroke_color` e `fill_color`.
    fn parse_args_cor(&mut self) -> Result<Vec<Expr>, DslError> {
        let mut args = Vec::new();
        // Atalho: cor em hexadecimal direto, ex.: `color #ff5500` ou
        // `color #ff550088` (com alpha). Convertido para r g b [a] em 0..1,
        // reaproveitando o mesmo caminho de `r g b [a]`.
        if let Some(tok) = self.peek() {
            if tok.starts_with('#') {
                let hex = self.proximo().unwrap();
                let rgba = self.parse_hex_cor(hex)?;
                let exprs: Vec<Expr> = rgba.into_iter().map(Expr::Num).collect();
                return Ok(exprs);
            }
        }
        while args.len() < 4 {
            if self.fim_de_linha() {
                break;
            }
            match self.peek().as_deref() {
                Some("}") | None => break,
                _ => args.push(self.parse_expr()?),
            }
        }
        if args.is_empty() || (args.len() != 1 && args.len() != 3 && args.len() != 4) {
            return Err(self.err("cor espera: nome, #hex, ou r g b, ou r g b a"));
        }
        Ok(args)
    }

    /// Converte uma cor hexadecimal (`#rgb`, `#rrggbb`, `#rrggbbaa`) em uma
    /// lista de componentes em 0..1 (r, g, b [, a]).
    fn parse_hex_cor(&self, tok: &str) -> Result<Vec<f32>, DslError> {
        let h = tok.trim_start_matches('#');
        let bytes = h.as_bytes();
        let parse2 = |s: &[u8]| {
            let txt = std::str::from_utf8(s).unwrap_or("");
            u8::from_str_radix(txt, 16).map_err(|_| {
                self.err(format!("dígitos hex inválidos em '{tok}'"))
            })
        };
        let to_f = |v: u8| v as f32 / 255.0;
        match bytes.len() {
            6 => Ok(vec![to_f(parse2(&bytes[0..2])?), to_f(parse2(&bytes[2..4])?), to_f(parse2(&bytes[4..6])?)]),
            8 => Ok(vec![
                to_f(parse2(&bytes[0..2])?),
                to_f(parse2(&bytes[2..4])?),
                to_f(parse2(&bytes[4..6])?),
                to_f(parse2(&bytes[6..8])?),
            ]),
            3 => Ok(vec![
                to_f(parse2(&[bytes[0], bytes[0]])?),
                to_f(parse2(&[bytes[1], bytes[1]])?),
                to_f(parse2(&[bytes[2], bytes[2]])?),
            ]),
            _ => Err(self.err(format!(
                "cor hex inválida '{tok}' (use #rrggbb ou #rrggbbaa)"
            ))),
        }
    }

    fn parse_program(&mut self) -> Result<Program, DslError> {
        let mut stmts = Vec::new();
        while let Some(tok) = self.proximo() {
            stmts.push(self.parse_stmt(tok)?);
        }
        Ok(Program { stmts })
    }

    /// Constrói um `Stmt` anotando a linha atual (1-based) para reportar
    /// erros de avaliação localizados.
    fn stmt(&self, kind: StmtKind) -> Stmt {
        Stmt {
            linha: self.li + 1,
            kind,
        }
    }

    /// Palavras-chave do statements (não podem ser alvo de atribuição direta).
    fn eh_palavra_chave(tok: &str) -> bool {
        matches!(
            tok,
            "let"
                | "move"
                | "line"
                | "rect"
                | "circle"
                | "bezier"
                | "close"
                | "fill"
                | "stroke"
                | "color"
                | "strokecolor"
                | "fillcolor"
                | "text"
                | "polygon"
                | "star"
                | "arc"
                | "roundrect"
                | "round_rect"
                | "grid"
                | "point"
                | "line_to"
                | "lineto"
                | "lt"
                | "curve_to"
                | "curveto"
                | "ct"
                | "translate"
                | "trans"
                | "rotate"
                | "rot"
                | "scale"
                | "push"
                | "pop"
                | "snake"
                | "repeat"
                | "for"
                | "while"
                | "if"
                | "else"
                | "fn"
                | "return"
        )
    }

    fn parse_stmt(&mut self, tok: &str) -> Result<Stmt, DslError> {
        // Atribuição direta: `nome = expr`. Só quando `tok` é um identificador
        // válido (não palavra-chave) e o próximo token é `=` (e não `==`).
        if !Self::eh_palavra_chave(tok) && self.peek().as_deref() == Some("=") {
            self.proximo(); // consome o '='
            let expr = self.parse_expr()?;
            return Ok(self.stmt(StmtKind::Assign(tok.to_string(), expr)));
        }
        match tok {
            "let" => {
                let nome = self
                    .proximo()
                    .ok_or_else(|| self.err("esperado nome após let"))?;
                if self.proximo() != Some("=") {
                    return Err(self.err("esperado '=' após nome"));
                }
                let expr = self.parse_expr()?;
                Ok(self.stmt(StmtKind::Let(nome.to_string(), expr)))
            }
            "move" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Move(x, y)))
            }
            "line" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Line(x, y)))
            }
            "rect" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                let w = self.parse_arg()?;
                let h = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Rect(x, y, w, h)))
            }
            "circle" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                let r = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Circle(x, y, r)))
            }
            "bezier" => {
                let c1x = self.parse_arg()?;
                let c1y = self.parse_arg()?;
                let c2x = self.parse_arg()?;
                let c2y = self.parse_arg()?;
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Bezier(c1x, c1y, c2x, c2y, x, y)))
            }
            "close" => Ok(self.stmt(StmtKind::Close)),
            "fill" => {
                let v = self
                    .proximo()
                    .ok_or_else(|| self.err("esperado on/off após fill"))?;
                match v {
                    "on" => Ok(self.stmt(StmtKind::Fill(true))),
                    "off" => Ok(self.stmt(StmtKind::Fill(false))),
                    _ => Err(self.err("fill espera on ou off")),
                }
            }
            "stroke" => {
                let w = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Stroke(w)))
            }
            "color" => {
                let args = self.parse_args_cor()?;
                Ok(self.stmt(StmtKind::Color(args)))
            }
            "stroke_color" | "strokecolor" => {
                let args = self.parse_args_cor()?;
                Ok(self.stmt(StmtKind::StrokeColor(args)))
            }
            "fill_color" | "fillcolor" => {
                let args = self.parse_args_cor()?;
                Ok(self.stmt(StmtKind::FillColor(args)))
            }
            "text" => {
                let conteudo = self.parse_string()?;
                let x = self.parse_expr()?;
                let y = self.parse_arg()?;
                // `size` (px de projeto) é opcional; padrão 48.
                let tamanho = if self.peek().as_deref().map_or(false, |t| {
                    t != "}" && !t.starts_with('#') && !self.fim_de_linha()
                }) {
                    self.parse_arg()?
                } else {
                    Expr::Num(48.0)
                };
                // flags opcionais `bold`/`italic` (palavras isoladas), e
                // `align left|center|right` e `rot graus` (opcionais).
                let mut negrito = false;
                let mut italico = false;
                let mut alinhamento = TextoAlinhamento::Left;
                let mut rotacao = Expr::Num(0.0);
                while let Some(tok) = self.peek() {
                    match tok.as_str() {
                        "bold" => {
                            self.proximo();
                            negrito = true;
                        }
                        "italic" => {
                            self.proximo();
                            italico = true;
                        }
                        "align" => {
                            self.proximo();
                            let a = self
                                .proximo()
                                .ok_or_else(|| self.err("esperado alinhamento após align"))?;
                            alinhamento = match a {
                                "center" => TextoAlinhamento::Center,
                                "right" => TextoAlinhamento::Right,
                                "left" => TextoAlinhamento::Left,
                                _ => return Err(self.err("align espera left/center/right")),
                            };
                        }
                        "rot" => {
                            self.proximo();
                            rotacao = self.parse_expr()?;
                        }
                        _ => break,
                    }
                }
                Ok(self.stmt(StmtKind::Text {
                    conteudo,
                    x,
                    y,
                    tamanho,
                    negrito,
                    italico,
                    alinhamento,
                    rotacao,
                }))
            }
            "polygon" => {
                let n = self.parse_arg()?;
                let cx = self.parse_arg()?;
                let cy = self.parse_arg()?;
                let r = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Polygon(n, cx, cy, r)))
            }
            "star" => {
                let n = self.parse_arg()?;
                let cx = self.parse_arg()?;
                let cy = self.parse_arg()?;
                let r1 = self.parse_arg()?;
                let r2 = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Star(n, cx, cy, r1, r2)))
            }
            "arc" => {
                let a0 = self.parse_arg()?;
                let a1 = self.parse_arg()?;
                let r = self.parse_arg()?;
                let cx = self.parse_arg()?;
                let cy = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Arc(a0, a1, r, cx, cy)))
            }
            "round_rect" | "roundrect" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                let w = self.parse_arg()?;
                let h = self.parse_arg()?;
                let r = self.parse_arg()?;
                Ok(self.stmt(StmtKind::RoundRect(x, y, w, h, r)))
            }
            "grid" => {
                let cols = self.parse_arg()?;
                let rows = self.parse_arg()?;
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                let w = self.parse_arg()?;
                let h = self.parse_arg()?;
                let pr = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Grid(cols, rows, x, y, w, h, pr)))
            }
            "point" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Point(x, y)))
            }
            "line_to" | "lineto" | "lt" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                Ok(self.stmt(StmtKind::LineTo(x, y)))
            }
            "curve_to" | "curveto" | "ct" => {
                let c1x = self.parse_arg()?;
                let c1y = self.parse_arg()?;
                let c2x = self.parse_arg()?;
                let c2y = self.parse_arg()?;
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                Ok(self.stmt(StmtKind::CurveTo(c1x, c1y, c2x, c2y, x, y)))
            }
            "translate" | "trans" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Translate(x, y)))
            }
            "rotate" | "rot" => {
                let ang = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Rotate(ang)))
            }
            "scale" => {
                let sx = self.parse_arg()?;
                let sy = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Scale(sx, sy)))
            }
            "push" => Ok(self.stmt(StmtKind::Push)),
            "pop" => Ok(self.stmt(StmtKind::Pop)),
            "snake" => {
                let x = self.parse_arg()?;
                let y = self.parse_arg()?;
                let length = self.parse_arg()?;
                let segments = self.parse_arg()?;
                Ok(self.stmt(StmtKind::Snake(x, y, length, segments)))
            }
            "repeat" => {
                let n = self.parse_expr()?;
                if self.proximo() != Some("{") {
                    return Err(self.err("esperado '{' após repeat n"));
                }
                let corpo = self.parse_bloco()?;
                Ok(self.stmt(StmtKind::Repeat(n, corpo)))
            }
            "if" => {
                let cond = self.parse_expr()?;
                if self.proximo() != Some("{") {
                    return Err(self.err("esperado '{' após if cond"));
                }
                let entao = self.parse_bloco()?;
                let mut senao = Vec::new();
                // opcional: else { ... }  ou  else if cond { ... }
                if self.peek().as_deref() == Some("else") {
                    self.proximo();
                    if self.peek().as_deref() == Some("if") {
                        // reusa o parse de 'if' para o ramo else-if
                        let tok = self
                            .proximo()
                            .ok_or_else(|| self.err("esperado if após else"))?;
                        let sub = self.parse_stmt(&tok)?;
                        if let StmtKind::If(c, t, e) = &sub.kind {
                            senao.push(self.stmt(StmtKind::If(c.clone(), t.clone(), e.clone())));
                        }
                    } else if self.proximo() != Some("{") {
                        return Err(self.err("esperado '{' após else"));
                    } else {
                        senao = self.parse_bloco()?;
                    }
                }
                Ok(self.stmt(StmtKind::If(cond, entao, senao)))
            }
            "for" => {
                let var = self
                    .proximo()
                    .ok_or_else(|| self.err("esperado nome da variável após for"))?;
                if self.proximo() != Some("in") {
                    return Err(self.err("esperado 'in' após for var"));
                }
                let inicio = self.parse_expr()?;
                if self.proximo() != Some("..") {
                    return Err(self.err("esperado '..' no for (ex.: 0..10)"));
                }
                let fim = self.parse_expr()?;
                // Passo opcional: `for v in a..b step N { ... }`
                let passo = if self.peek().as_deref() == Some("step") {
                    self.proximo();
                    self.parse_arg()?
                } else {
                    Expr::Num(1.0)
                };
                if self.proximo() != Some("{") {
                    return Err(self.err("esperado '{' após for"));
                }
                let corpo = self.parse_bloco()?;
                Ok(self.stmt(StmtKind::For(
                    var.to_string(),
                    inicio,
                    fim,
                    passo,
                    corpo,
                )))
            }
            "while" => {
                let cond = self.parse_expr()?;
                if self.proximo() != Some("{") {
                    return Err(self.err("esperado '{' após while cond"));
                }
                let corpo = self.parse_bloco()?;
                Ok(self.stmt(StmtKind::While(cond, corpo)))
            }
            "fn" => {
                let nome = self
                    .proximo()
                    .ok_or_else(|| self.err("esperado nome da função após fn"))?;
                if self.proximo() != Some("(") {
                    return Err(self.err("esperado '(' após nome da função"));
                }
                let mut params = Vec::new();
                if self.peek().as_deref() != Some(")") {
                    loop {
                        let p = self
                            .proximo()
                            .ok_or_else(|| self.err("esperado parâmetro ou ')'"))?;
                        if p == ")" {
                            break;
                        }
                        params.push(p.to_string());
                        match self.peek().as_deref() {
                            Some(",") => {
                                self.proximo();
                                continue;
                            }
                            Some(")") => {
                                self.proximo();
                                break;
                            }
                            _ => return Err(self.err("esperado ',' ou ')' na lista de parâmetros")),
                        }
                    }
                } else {
                    self.proximo();
                }
                if self.proximo() != Some("{") {
                    return Err(self.err("esperado '{' após fn"));
                }
                let corpo = self.parse_bloco()?;
                Ok(self.stmt(StmtKind::Fn(nome.to_string(), params, corpo)))
            }
            "return" => {
                let e = self.parse_expr()?;
                Ok(self.stmt(StmtKind::Return(e)))
            }
            _ => Err(self.err(format!("comando desconhecido '{tok}'"))),
        }
    }

    /// Parseia um bloco `{ ... }` de statements até o `}` correspondente.
    fn parse_bloco(&mut self) -> Result<Vec<Stmt>, DslError> {
        let mut corpo = Vec::new();
        loop {
            let tok = self
                .proximo()
                .ok_or_else(|| self.err("bloco não fechado ('}' faltando)"))?;
            if tok == "}" {
                break;
            }
            corpo.push(self.parse_stmt(&tok)?);
        }
        Ok(corpo)
    }

    /// expr := or
    /// Topo da gramática: lógica (or/and) sobre o nível aditivo.
    fn parse_expr(&mut self) -> Result<Expr, DslError> {
        self.parse_or()
    }

    /// Lê um argumento de comando. Igual a `parse_expr`, porém o sinal `-`
    /// NÃO é tratado como subtração binária: ele sempre inicia um novo
    /// argumento (unário). Isso resolve a ambiguidade `move 0 -ra` (x=0,
    /// y=-ra) em vez de `0 - ra` (subtração). Para subtrair DENTRO de um
    /// argumento, use parênteses: `circle 0 0 (100 - 5)`.
    ///
    /// O `+` ainda é aceito como operador binário, então `line x1 + 5 y1`
    /// continua significando x = x1+5.
    fn parse_arg(&mut self) -> Result<Expr, DslError> {
        // or/and: mantidos para não quebrar expressões vindas de parênteses,
        // mas o nível aditivo (abaixo) é o que evita o '-' binário.
        let mut lhs = self.parse_aditivo_arg()?;
        while self.peek().as_deref() == Some("and") {
            self.proximo();
            let rhs = self.parse_aditivo_arg()?;
            lhs = Expr::Bin(Box::new(lhs), BinOp::And, Box::new(rhs));
        }
        while self.peek().as_deref() == Some("or") {
            self.proximo();
            let rhs = self.parse_aditivo_arg()?;
            lhs = Expr::Bin(Box::new(lhs), BinOp::Or, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// aditivo para argumentos: consome apenas '+' (não '-') como binário.
    fn parse_aditivo_arg(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_cmp_arg()?;
        while let Some(op) = self.peek() {
            if op == "+" {
                self.proximo();
                let rhs = self.parse_cmp_arg()?;
                lhs = Expr::Bin(Box::new(lhs), BinOp::Add, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// Igual a `parse_cmp`, mas usado dentro de argumentos (delega term/arg).
    fn parse_cmp_arg(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_term_arg()?;
        loop {
            let op = match self.peek().as_deref() {
                Some(">") => CmpOp::Gt,
                Some("<") => CmpOp::Lt,
                Some(">=") => CmpOp::Ge,
                Some("<=") => CmpOp::Le,
                Some("==") => CmpOp::Eq,
                Some("!=") => CmpOp::Ne,
                Some("=") => CmpOp::Eq,
                _ => break,
            };
            self.proximo();
            let rhs = self.parse_term_arg()?;
            lhs = Expr::Cmp(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Igual a `parse_term`, mas o nível aditivo acima não consome '-'.
    fn parse_term_arg(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_factor()?;
        while let Some(op) = self.peek() {
            if op == "*" || op == "/" || op == "%" {
                self.proximo();
                let rhs = self.parse_factor()?;
                let b = match op.as_str() {
                    "*" => BinOp::Mul,
                    "/" => BinOp::Div,
                    "%" => BinOp::Mod,
                    _ => unreachable!(),
                };
                lhs = Expr::Bin(Box::new(lhs), b, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// or := and ( 'or' and )*
    fn parse_or(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_and()?;
        while self.peek().as_deref() == Some("or") {
            self.proximo();
            let rhs = self.parse_and()?;
            lhs = Expr::Bin(Box::new(lhs), BinOp::Or, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// and := aditivo ( 'and' aditivo )*
    fn parse_and(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_aditivo()?;
        while self.peek().as_deref() == Some("and") {
            self.proximo();
            let rhs = self.parse_aditivo()?;
            lhs = Expr::Bin(Box::new(lhs), BinOp::And, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// aditivo := cmp (('+' | '-') cmp)*
    fn parse_aditivo(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_cmp()?;
        while let Some(op) = self.peek() {
            if op == "+" || op == "-" {
                self.proximo();
                let rhs = self.parse_cmp()?;
                let b = if op == "+" { BinOp::Add } else { BinOp::Sub };
                lhs = Expr::Bin(Box::new(lhs), b, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// cmp := term (('>' | '<' | '>=' | '<=' | '==' | '!=' | '=') term)*
    /// Produz 1.0 se a comparação for verdadeira, 0.0 caso contrário.
    /// `=` sozinho é aceito como `==` (tolerância).
    fn parse_cmp(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_term()?;
        loop {
            let op = match self.peek().as_deref() {
                Some(">") => CmpOp::Gt,
                Some("<") => CmpOp::Lt,
                Some(">=") => CmpOp::Ge,
                Some("<=") => CmpOp::Le,
                Some("==") => CmpOp::Eq,
                Some("!=") => CmpOp::Ne,
                Some("=") => CmpOp::Eq,
                _ => break,
            };
            self.proximo();
            let rhs = self.parse_term()?;
            lhs = Expr::Cmp(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    /// term := factor (('*' | '/' | '%') factor)*
    fn parse_term(&mut self) -> Result<Expr, DslError> {
        let mut lhs = self.parse_factor()?;
        while let Some(op) = self.peek() {
            if op == "*" || op == "/" || op == "%" {
                self.proximo();
                let rhs = self.parse_factor()?;
                let b = match op.as_str() {
                    "*" => BinOp::Mul,
                    "/" => BinOp::Div,
                    "%" => BinOp::Mod,
                    _ => unreachable!(),
                };
                lhs = Expr::Bin(Box::new(lhs), b, Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    /// Aplica repetidamente acessos de campo `.x`/`.y` a uma expressão base,
    /// ex.: `pos.x`, `vec2(1,2).y`, `(a + b).x`.
    fn aplicar_campo(&mut self, mut e: Expr) -> Result<Expr, DslError> {
        while self.peek().as_deref() == Some(".") {
            self.proximo(); // consome o '.'
            let campo = self
                .proximo()
                .ok_or_else(|| self.err("esperado campo após '.' (x ou y)"))?;
            if campo != "x" && campo != "y" {
                return Err(self.err("campo desconhecido (use .x ou .y)"));
            }
            e = Expr::Field(Box::new(e), campo.to_string());
        }
        Ok(e)
    }

    /// factor := number | var | var '(' args ')' | '(' expr ')' | '-' factor
    /// | expr '.' ('x' | 'y')
    fn parse_factor(&mut self) -> Result<Expr, DslError> {
        let tok = self
            .proximo()
            .ok_or_else(|| self.err("esperado expressão"))?;
        if tok == "(" {
            let e = self.parse_expr()?;
            if self.proximo() != Some(")") {
                return Err(self.err("esperado ')'"));
            }
            return self.aplicar_campo(e);
        }
        if tok == "-" {
            let f = self.parse_factor()?;
            return Ok(Expr::Bin(
                Box::new(Expr::Num(0.0)),
                BinOp::Sub,
                Box::new(f),
            ));
        }
        if let Ok(n) = tok.parse::<f32>() {
            return Ok(Expr::Num(n));
        }
        if tok == "\"" {
            let s = self.parse_string_expr()?;
            return Ok(Expr::Str(s));
        }
        if self.peek().as_deref() == Some("(") {
            self.proximo();
            let mut args = Vec::new();
            if self.peek().as_deref() != Some(")") {
                loop {
                    args.push(self.parse_expr()?);
                    if self.peek().as_deref() == Some(",") {
                        self.proximo();
                        continue;
                    }
                    break;
                }
            }
            if self.proximo() != Some(")") {
                return Err(self.err("esperado ')' após argumentos"));
            }
            return self.aplicar_campo(Expr::Call(tok.to_string(), args));
        }
        self.aplicar_campo(Expr::Var(tok.to_string()))
    }
}

// ------------------------------------------------------------------ Eval

/// Estado salvo por `push`/`pop`: transformação afim atual + estilo.
#[derive(Clone, Copy)]
struct EstadoSalvo {
    transform: [f32; 6],
    cor_atual: Color32,
    cor_fill_atual: Color32,
    stroke_atual: f32,
    fill_atual: bool,
}

/// Matriz afim 2D compacta: [[a, b, c], [d, e, f]] aplicada como
/// x' = a*x + b*y + c ; y' = d*x + e*y + f.
fn ident() -> [f32; 6] {
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
}

/// Pré-multiplica a matriz `m` pela transformação dada por `t` (aplicada
/// "ao redor" do sistema atual — ou seja, `t` age primeiro, depois `m`).
fn compose(m: &[f32; 6], t: &[f32; 6]) -> [f32; 6] {
    // resultado = m * t
    [
        m[0] * t[0] + m[1] * t[3],
        m[0] * t[1] + m[1] * t[4],
        m[0] * t[2] + m[1] * t[5] + m[2],
        m[3] * t[0] + m[4] * t[3],
        m[3] * t[1] + m[4] * t[4],
        m[3] * t[2] + m[4] * t[5] + m[5],
    ]
}

struct Eval {
    t: f32,
    seed: u32,
    /// Duração do ciclo (segundos) usada pela variável implícita `progress`.
    /// Padrão 6s (loop dos exemplos); pode ser ajustada pelo app.
    duracao: f32,
    /// Estado do gerador de números pseudoaleatórios determinístico. Cada
    /// chamada de `rand()` avança este estado, mantendo a reprodutibilidade
    /// (mesma seed do nó => mesma sequência), essencial para a exportação.
    rand_state: u32,
    vars: HashMap<String, Valor>,
    cmds: Vec<PathCmd>,
    /// Transformação afim atual (acumulada por translate/rotate/scale).
    transform: [f32; 6],
    /// Pilha de estados salvos por `push`/`pop`.
    stack: Vec<EstadoSalvo>,
    /// Funções definidas pelo usuário: nome -> (parâmetros, corpo).
    funcs: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    cor_atual: Color32,
    cor_fill_atual: Color32,
    stroke_atual: f32,
    fill_atual: bool,
}

/// Sinal interno de controle de fluxo (return de função).
enum Flow {
    Normal,
    Return(Valor),
}

impl Eval {
    fn new(t: f32, seed: u32) -> Self {
        Self {
            t,
            seed,
            duracao: 6.0,
            rand_state: seed.wrapping_add(0x9E3779B9),
            // `i` vale 0 fora de um `repeat` (evita erro ao usar fora do loop).
            vars: {
                let mut m = HashMap::new();
                m.insert("i".to_string(), Valor::Num(0.0));
                m
            },
            cmds: Vec::new(),
            transform: ident(),
            stack: Vec::new(),
            funcs: HashMap::new(),
            cor_atual: Color32::WHITE,
            cor_fill_atual: Color32::WHITE,
            stroke_atual: 2.0,
            fill_atual: false,
        }
    }

    /// Aplica a transformação afim atual a um ponto.
    fn xf(&self, p: GVec2) -> GVec2 {
        let m = &self.transform;
        GVec2::new(
            m[0] * p.x + m[1] * p.y + m[2],
            m[3] * p.x + m[4] * p.y + m[5],
        )
    }

    /// Empurra um `Move`, aplicando a transformação atual.
    fn push_move(&mut self, x: f32, y: f32) {
        self.cmds.push(PathCmd::Move(self.xf(GVec2::new(x, y))));
    }
    /// Empurra um `Line`, aplicando a transformação atual.
    fn push_line(&mut self, x: f32, y: f32) {
        self.cmds.push(PathCmd::Line(self.xf(GVec2::new(x, y))));
    }
    /// Empurra um `Bezier`, aplicando a transformação atual.
    fn push_bezier(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.cmds.push(PathCmd::Bezier(
            self.xf(GVec2::new(c1x, c1y)),
            self.xf(GVec2::new(c2x, c2y)),
            self.xf(GVec2::new(x, y)),
        ));
    }

    /// Próximo valor pseudoaleatório em [0,1), determinístico a partir de
    /// `rand_state` (xorshift de 32 bits). Avança o estado a cada chamada.
    fn prox_rand(&mut self) -> f32 {
        let mut x = self.rand_state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rand_state = x;
        // mapeia para [0,1)
        (x & 0x00FF_FFFF) as f32 / (0x01_00_00_00u32 as f32)
    }

    fn run(&mut self, stmts: &[Stmt]) -> Result<Flow, DslError> {
        // Registra as definições de função antes de executar (permite chamar
        // funções declaradas depois no código).
        for s in stmts {
            if let StmtKind::Fn(nome, params, corpo) = &s.kind {
                self.funcs.insert(nome.clone(), (params.clone(), corpo.clone()));
            }
        }
        for s in stmts {
            if let Flow::Return(v) = self.exec(s)? {
                return Ok(Flow::Return(v));
            }
        }
        Ok(Flow::Normal)
    }

    /// Erro de avaliação localizado na linha do statement atual.
    fn eval_err(&self, msg: impl Into<String>, linha: usize) -> DslError {
        DslError::Eval {
            msg: msg.into(),
            linha,
        }
    }

    fn exec(&mut self, s: &Stmt) -> Result<Flow, DslError> {
        let linha = s.linha;
        match &s.kind {
            StmtKind::Let(nome, e) => {
                let v = self.eval_expr(e, linha)?;
                self.vars.insert(nome.clone(), v);
                Ok(Flow::Normal)
            }
            StmtKind::Move(x, y) => {
                let (x, y) = (self.eval_expr_num(x, linha)?, self.eval_expr_num(y, linha)?);
                self.push_move(x, y);
                Ok(Flow::Normal)
            }
            StmtKind::Line(x, y) => {
                let (x, y) = (self.eval_expr_num(x, linha)?, self.eval_expr_num(y, linha)?);
                self.push_line(x, y);
                Ok(Flow::Normal)
            }
            StmtKind::Point(x, y) => {
                let (x, y) = (self.eval_expr_num(x, linha)?, self.eval_expr_num(y, linha)?);
                self.push_move(x, y);
                Ok(Flow::Normal)
            }
            StmtKind::LineTo(x, y) => {
                let (x, y) = (self.eval_expr_num(x, linha)?, self.eval_expr_num(y, linha)?);
                self.push_line(x, y);
                Ok(Flow::Normal)
            }
            StmtKind::CurveTo(c1x, c1y, c2x, c2y, x, y) => {
                let (c1x, c1y, c2x, c2y, x, y) = (
                    self.eval_expr_num(c1x, linha)?,
                    self.eval_expr_num(c1y, linha)?,
                    self.eval_expr_num(c2x, linha)?,
                    self.eval_expr_num(c2y, linha)?,
                    self.eval_expr_num(x, linha)?,
                    self.eval_expr_num(y, linha)?,
                );
                self.push_bezier(c1x, c1y, c2x, c2y, x, y);
                Ok(Flow::Normal)
            }
            StmtKind::Translate(x, y) => {
                let (x, y) = (self.eval_expr_num(x, linha)?, self.eval_expr_num(y, linha)?);
                // T = translate(x, y) aplicada "por fora" (no sistema atual).
                self.transform = compose(&self.transform, &[1.0, 0.0, x, 0.0, 1.0, y]);
                Ok(Flow::Normal)
            }
            StmtKind::Rotate(ang) => {
                let ang = self.eval_expr_num(ang, linha)?;
                let r = ang.to_radians();
                let (s, c) = (r.sin(), r.cos());
                self.transform = compose(&self.transform, &[c, -s, 0.0, s, c, 0.0]);
                Ok(Flow::Normal)
            }
            StmtKind::Scale(sx, sy) => {
                let (sx, sy) = (self.eval_expr_num(sx, linha)?, self.eval_expr_num(sy, linha)?);
                self.transform = compose(&self.transform, &[sx, 0.0, 0.0, 0.0, sy, 0.0]);
                Ok(Flow::Normal)
            }
            StmtKind::Push => {
                self.stack.push(EstadoSalvo {
                    transform: self.transform,
                    cor_atual: self.cor_atual,
                    cor_fill_atual: self.cor_fill_atual,
                    stroke_atual: self.stroke_atual,
                    fill_atual: self.fill_atual,
                });
                Ok(Flow::Normal)
            }
            StmtKind::Pop => {
                if let Some(e) = self.stack.pop() {
                    self.transform = e.transform;
                    self.cor_atual = e.cor_atual;
                    self.cor_fill_atual = e.cor_fill_atual;
                    self.stroke_atual = e.stroke_atual;
                    self.fill_atual = e.fill_atual;
                }
                Ok(Flow::Normal)
            }
            StmtKind::Snake(x, y, length, segments) => {
                let (x, y, length, segments) =
                    (self.eval_expr_num(x, linha)?, self.eval_expr_num(y, linha)?,
                     self.eval_expr_num(length, linha)?, self.eval_expr_num(segments, linha)?);
                let segs = segments.max(1.0) as usize;
                let step = length / segs as f32;
                let amp = step * 0.5;
                self.push_move(x, y);
                for s in 1..=segs {
                    let t = s as f32 / segs as f32;
                    // oscila perpendicular (eixo y local) com seno
                    let wob = (t * std::f32::consts::PI * 2.0 * segs as f32 * 0.5).sin() * amp;
                    self.push_line(x + step * s as f32, y + wob);
                }
                Ok(Flow::Normal)
            }
            StmtKind::Rect(x, y, w, h) => {
                let (x, y, w, h) = (
                    self.eval_expr_num(x, linha)?,
                    self.eval_expr_num(y, linha)?,
                    self.eval_expr_num(w, linha)?,
                    self.eval_expr_num(h, linha)?,
                );
                self.push_move(x, y);
                self.push_line(x + w, y);
                self.push_line(x + w, y + h);
                self.push_line(x, y + h);
                self.cmds.push(PathCmd::Close);
                Ok(Flow::Normal)
            }
            StmtKind::Circle(x, y, r) => {
                let (x, y, r) = (
                    self.eval_expr_num(x, linha)?,
                    self.eval_expr_num(y, linha)?,
                    self.eval_expr_num(r, linha)?,
                );
                // Nº de segmentos FIXO (independente do raio): se dependesse do
                // raio, animar o raio faria a resolução "saltar" entre frames,
                // causando um piscar/deformar. 64 é suave o bastante para
                // círculos grandes sem custo relevante.
                let n = 64u32;
                // Gera 0..n (SEM repetir o ponto inicial): o `Close` fecha o
                // laço. Repetir o 1º ponto e ainda emitir `Close` criava uma
                // aresta de comprimento zero (vértice degenerado) que o
                // tessellator com anti-alias transformava numa "aba" que
                // tremulava conforme a posição sub-pixel mudava no tempo.
                for i in 0..n {
                    let a = (i as f32 / n as f32) * std::f32::consts::TAU;
                    let px = x + a.cos() * r;
                    let py = y + a.sin() * r;
                    if i == 0 {
                        self.push_move(px, py);
                    } else {
                        self.push_line(px, py);
                    }
                }
                self.cmds.push(PathCmd::Close);
                Ok(Flow::Normal)
            }
            StmtKind::Bezier(c1x, c1y, c2x, c2y, x, y) => {
                let v = (
                    self.eval_expr_num(c1x, linha)?,
                    self.eval_expr_num(c1y, linha)?,
                    self.eval_expr_num(c2x, linha)?,
                    self.eval_expr_num(c2y, linha)?,
                    self.eval_expr_num(x, linha)?,
                    self.eval_expr_num(y, linha)?,
                );
                self.push_bezier(v.0, v.1, v.2, v.3, v.4, v.5);
                Ok(Flow::Normal)
            }
            StmtKind::Close => {
                self.cmds.push(PathCmd::Close);
                Ok(Flow::Normal)
            }
            StmtKind::Fill(b) => {
                self.fill_atual = *b;
                self.cmds.push(PathCmd::Fill(*b));
                Ok(Flow::Normal)
            }
            StmtKind::Stroke(w) => {
                let w = self.eval_expr_num(w, linha)?;
                self.stroke_atual = w;
                self.cmds.push(PathCmd::Stroke(w));
                Ok(Flow::Normal)
            }
            StmtKind::Color(args) => {
                let c = self.resolver_cor(args, linha)?;
                self.cor_atual = c;
                self.cor_fill_atual = c;
                self.cmds.push(PathCmd::Color(c));
                Ok(Flow::Normal)
            }
            StmtKind::StrokeColor(args) => {
                let c = self.resolver_cor(args, linha)?;
                self.cor_atual = c;
                self.cmds.push(PathCmd::ColorStroke(c));
                Ok(Flow::Normal)
            }
            StmtKind::FillColor(args) => {
                let c = self.resolver_cor(args, linha)?;
                self.cor_fill_atual = c;
                self.cmds.push(PathCmd::ColorFill(c));
                Ok(Flow::Normal)
            }
            StmtKind::Text {
                conteudo,
                x,
                y,
                tamanho,
                negrito,
                italico,
                alinhamento,
                rotacao,
            } => {
                let (x, y, tam, rot) = (
                    self.eval_expr_num(x, linha)?,
                    self.eval_expr_num(y, linha)?,
                    self.eval_expr_num(tamanho, linha)?,
                    self.eval_expr_num(rotacao, linha)?,
                );
                self.cmds.push(PathCmd::Text {
                    conteudo: conteudo.clone(),
                    x,
                    y,
                    tamanho: tam.max(1.0),
                    negrito: *negrito,
                    italico: *italico,
                    alinhamento: *alinhamento,
                    rotacao: rot,
                    cor: self.cor_atual,
                });
                Ok(Flow::Normal)
            }
            StmtKind::Polygon(n, cx, cy, r) => {
                let (n, cx, cy, r) = (
                    self.eval_expr_num(n, linha)? as i32,
                    self.eval_expr_num(cx, linha)?,
                    self.eval_expr_num(cy, linha)?,
                    self.eval_expr_num(r, linha)?,
                );
                if n < 3 {
                    return Err(self.eval_err("polygon precisa de n >= 3", linha));
                }
                self.emit_poligono(n as usize, cx, cy, r, 0.0);
                Ok(Flow::Normal)
            }
            StmtKind::Star(n, cx, cy, r1, r2) => {
                let (n, cx, cy, r1, r2) = (
                    self.eval_expr_num(n, linha)? as i32,
                    self.eval_expr_num(cx, linha)?,
                    self.eval_expr_num(cy, linha)?,
                    self.eval_expr_num(r1, linha)?,
                    self.eval_expr_num(r2, linha)?,
                );
                if n < 2 {
                    return Err(self.eval_err("star precisa de n >= 2", linha));
                }
                // estrela: alterna entre r1 (pontas) e r2 (vales), 2*n vértices
                let total = (n as usize) * 2;
                let mut primeiro = true;
                for k in 0..total {
                    let ang = (k as f32) * (std::f32::consts::PI / n as f32) - std::f32::consts::FRAC_PI_2;
                    let rr = if k % 2 == 0 { r1 } else { r2 };
                    let px = cx + ang.cos() * rr;
                    let py = cy + ang.sin() * rr;
                    if primeiro {
                        self.push_move(px, py);
                        primeiro = false;
                    } else {
                        self.push_line(px, py);
                    }
                }
                self.cmds.push(PathCmd::Close);
                Ok(Flow::Normal)
            }
            StmtKind::Arc(a0, a1, r, cx, cy) => {
                let (a0, a1, r, cx, cy) = (
                    self.eval_expr_num(a0, linha)?,
                    self.eval_expr_num(a1, linha)?,
                    self.eval_expr_num(r, linha)?,
                    self.eval_expr_num(cx, linha)?,
                    self.eval_expr_num(cy, linha)?,
                );
                let a0r = a0.to_radians();
                let a1r = a1.to_radians();
                let passos = 64u32;
                let mut primeiro = true;
                for k in 0..=passos {
                    let u = k as f32 / passos as f32;
                    let ang = a0r + (a1r - a0r) * u;
                    let px = cx + ang.cos() * r;
                    let py = cy + ang.sin() * r;
                    if primeiro {
                        self.push_move(px, py);
                        primeiro = false;
                    } else {
                        self.push_line(px, py);
                    }
                }
                Ok(Flow::Normal)
            }
            StmtKind::RoundRect(x, y, w, h, r) => {
                let (x, y, w, h, r) = (
                    self.eval_expr_num(x, linha)?,
                    self.eval_expr_num(y, linha)?,
                    self.eval_expr_num(w, linha)?,
                    self.eval_expr_num(h, linha)?,
                    self.eval_expr_num(r, linha)?,
                );
                let r = r.max(0.0).min((w.abs() / 2.0).min(h.abs() / 2.0));
                self.emit_round_rect(x, y, w, h, r);
                Ok(Flow::Normal)
            }
            StmtKind::Grid(cols, rows, x, y, w, h, pr) => {
                let (cols, rows, x, y, w, h, pr) = (
                    self.eval_expr_num(cols, linha)? as i32,
                    self.eval_expr_num(rows, linha)? as i32,
                    self.eval_expr_num(x, linha)?,
                    self.eval_expr_num(y, linha)?,
                    self.eval_expr_num(w, linha)?,
                    self.eval_expr_num(h, linha)?,
                    self.eval_expr_num(pr, linha)?,
                );
                if cols < 1 || rows < 1 {
                    return Err(self.eval_err("grid precisa de cols,rows >= 1", linha));
                }
                let pr = pr.max(0.5);
                for iy in 0..rows {
                    for ix in 0..cols {
                        let gx = x + (ix as f32) * w;
                        let gy = y + (iy as f32) * h;
                        self.push_move(gx + pr, gy);
                        self.push_line(gx - pr, gy);
                        self.push_move(gx, gy + pr);
                        self.push_line(gx, gy - pr);
                    }
                }
                Ok(Flow::Normal)
            }
            StmtKind::Repeat(n, corpo) => {
                let n = self.eval_expr_num(n, linha)?;
                if n < 0.0 {
                    return Err(self.eval_err("repeat com n negativo", linha));
                }
                let n = (n as u32).min(2000);
                for i in 0..n {
                    self.vars.insert("i".to_string(), Valor::Num(i as f32));
                    self.run(corpo)?;
                }
                Ok(Flow::Normal)
            }
            StmtKind::For(var, inicio, fim, passo, corpo) => {
                let a = self.eval_expr_num(inicio, linha)?;
                let b = self.eval_expr_num(fim, linha)?;
                let step = self.eval_expr_num(passo, linha)?;
                // Limita o número de iterações (evita loop infinito com passo 0
                // ou muito pequeno) e suporta passo negativo.
                let step = if step == 0.0 { 1.0 } else { step };
                let mut k = a;
                let mut iter = 0u32;
                while (if step > 0.0 { k < b } else { k > b }) && iter < 2000 {
                    self.vars.insert(var.clone(), Valor::Num(k));
                    self.run(corpo)?;
                    k += step;
                    iter += 1;
                }
                Ok(Flow::Normal)
            }
            StmtKind::While(cond, corpo) => {
                let mut guard = 0u32;
                while self.eval_expr_num(cond, linha)? != 0.0 {
                    guard += 1;
                    if guard > 100_000 {
                        return Err(self.eval_err("while excedeu 100000 iterações", linha));
                    }
                    self.run(corpo)?;
                }
                Ok(Flow::Normal)
            }
            StmtKind::If(cond, entao, senao) => {
                let v = self.eval_expr_num(cond, linha)?;
                if v != 0.0 {
                    self.run(entao)?;
                } else {
                    self.run(senao)?;
                }
                Ok(Flow::Normal)
            }
            StmtKind::Fn(_, _, _) => {
                // Definição já foi registrada em `run`; nada a executar aqui.
                Ok(Flow::Normal)
            }
            StmtKind::Return(e) => {
                let v = self.eval_expr(e, linha)?;
                Ok(Flow::Return(v))
            }
            StmtKind::Assign(nome, e) => {
                let v = self.eval_expr(e, linha)?;
                self.vars.insert(nome.clone(), v);
                Ok(Flow::Normal)
            }
        }
    }

    /// Emite um polígono regular de `n` lados centrado em `(cx, cy)` com raio `r`,
    /// opcionalmente rotacionado por `rot` graus.
    fn emit_poligono(&mut self, n: usize, cx: f32, cy: f32, r: f32, rot: f32) {
        let rot = rot.to_radians();
        let mut primeiro = true;
        for k in 0..n {
            let ang = (k as f32) * (2.0 * std::f32::consts::PI / n as f32)
                - std::f32::consts::FRAC_PI_2
                + rot;
            let px = cx + ang.cos() * r;
            let py = cy + ang.sin() * r;
            if primeiro {
                self.push_move(px, py);
                primeiro = false;
            } else {
                self.push_line(px, py);
            }
        }
        self.cmds.push(PathCmd::Close);
    }

    /// Emite um retângulo com cantos arredondados de raio `r`.
    fn emit_round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        let (x0, y0) = (x, y);
        let (x1, y1) = (x + w, y + h);
        let r = r.abs();
        // Cantos: arco de 90° em cada quina.
        let mut quad = |cx: f32, cy: f32, a0: f32, a1: f32, first: &mut bool| {
            let passos = 12u32;
            for k in 0..=passos {
                let u = k as f32 / passos as f32;
                let ang = a0 + (a1 - a0) * u;
                let px = cx + ang.cos() * r;
                let py = cy + ang.sin() * r;
                if *first {
                    self.push_move(px, py);
                    *first = false;
                } else {
                    self.push_line(px, py);
                }
            }
        };
        let mut first = true;
        // top-right
        quad(x1 - r, y0 + r, -std::f32::consts::FRAC_PI_2, 0.0, &mut first);
        // bottom-right
        quad(x1 - r, y1 - r, 0.0, std::f32::consts::FRAC_PI_2, &mut first);
        // bottom-left
        quad(x0 + r, y1 - r, std::f32::consts::FRAC_PI_2, std::f32::consts::PI, &mut first);
        // top-left
        quad(
            x0 + r,
            y0 + r,
            std::f32::consts::PI,
            std::f32::consts::PI + std::f32::consts::FRAC_PI_2,
            &mut first,
        );
        self.cmds.push(PathCmd::Close);
    }

    /// Resolve um comando `color`: nome (ex.: "red"), rgb (3 args) ou rgba
    /// (4 args, o 4º é o alpha 0..1).
    fn resolver_cor(&mut self, args: &[Expr], linha: usize) -> Result<Color32, DslError> {
        if args.len() == 1 {
            if let Expr::Var(nome) = &args[0] {
                if let Some(c) = cor_nome(nome) {
                    return Ok(c);
                }
                return Err(self.eval_err(format!("cor '{nome}' desconhecida"), linha));
            }
            return Err(self.eval_err("color com nome espera um identificador", linha));
        }
        let r = self.eval_expr_num(&args[0], linha)?;
        let g = self.eval_expr_num(&args[1], linha)?;
        let b = self.eval_expr_num(&args[2], linha)?;
        let a = if args.len() == 4 {
            self.eval_expr_num(&args[3], linha)?
        } else {
            1.0
        };
        Ok(Color32::from_rgba_premultiplied(
            (r.clamp(0.0, 1.0) * 255.0) as u8,
            (g.clamp(0.0, 1.0) * 255.0) as u8,
            (b.clamp(0.0, 1.0) * 255.0) as u8,
            (a.clamp(0.0, 1.0) * 255.0) as u8,
        ))
    }

    fn eval_expr(&mut self, e: &Expr, linha: usize) -> Result<Valor, DslError> {
        match e {
            Expr::Num(nn) => Ok(Valor::Num(*nn)),
            Expr::Str(_) => Err(self.eval_err(
                "string literal só é válida como argumento de ease (ex.: ease(x, \"quad\"))",
                linha,
            )),
            Expr::Var(v) => {
                if v == "t" {
                    return Ok(Valor::Num(self.t));
                }
                // `phase` = fase contínua em radianos (t * 2π), útil para senos.
                if v == "phase" {
                    return Ok(Valor::Num(self.t * std::f32::consts::TAU));
                }
                // `beat` = fração 0..1 de uma batida a 120 BPM (2 batidas/s).
                if v == "beat" {
                    return Ok(Valor::Num((self.t * 2.0) % 1.0));
                }
                // `progress` = fração 0..1 do ciclo padrão de 6s (loop).
                if v == "progress" {
                    return Ok(Valor::Num((self.t / self.duracao).clamp(0.0, 1.0)));
                }
                // `i` vale 0 fora de repeat (já inserido no `new`).
                self.vars.get(v).copied().ok_or_else(|| {
                    self.eval_err(format!("variável '{v}' não definida"), linha)
                })
            }
            Expr::Bin(a, op, b) => {
                let (a, b) = (self.eval_expr(a, linha)?, self.eval_expr(b, linha)?);
                // Operações aritméticas exigem escalares.
                let (a, b) = (a.num(linha)?, b.num(linha)?);
                Ok(Valor::Num(match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => {
                        if b == 0.0 {
                            return Err(self.eval_err("divisão por zero", linha));
                        }
                        a / b
                    }
                    BinOp::Mod => {
                        if b == 0.0 {
                            return Err(self.eval_err("módulo por zero", linha));
                        }
                        a % b
                    }
                    BinOp::And => {
                        let av = a != 0.0;
                        let bv = b != 0.0;
                        if av && bv { 1.0 } else { 0.0 }
                    }
                    BinOp::Or => {
                        let av = a != 0.0;
                        let bv = b != 0.0;
                        if av || bv { 1.0 } else { 0.0 }
                    }
                }))
            }
            Expr::Cmp(a, op, b) => {
                let (a, b) = (self.eval_expr(a, linha)?, self.eval_expr(b, linha)?);
                let (a, b) = (a.num(linha)?, b.num(linha)?);
                let r = match op {
                    CmpOp::Lt => a < b,
                    CmpOp::Gt => a > b,
                    CmpOp::Le => a <= b,
                    CmpOp::Ge => a >= b,
                    CmpOp::Eq => a == b,
                    CmpOp::Ne => a != b,
                };
                Ok(Valor::Num(if r { 1.0 } else { 0.0 }))
            }
            Expr::Field(base, campo) => {
                let v = self.eval_expr(base, linha)?;
                match campo.as_str() {
                    "x" => Ok(Valor::Num(v.comp_x(linha)?)),
                    "y" => Ok(Valor::Num(v.comp_y(linha)?)),
                    _ => Err(self.eval_err(
                        format!("campo '{campo}' desconhecido (use .x ou .y)"),
                        linha,
                    )),
                }
            }
            Expr::Call(nome, args) => {
                let mut vals = Vec::new();
                for a in args {
                    // Strings literais (ex.: o tipo de `ease`) não são
                    // avaliadas como número; ocupam um slot neutro em `vals`
                    // e são lidas diretamente de `args` quando necessário.
                    if matches!(a, Expr::Str(_)) {
                        vals.push(Valor::Num(0.0));
                    } else {
                        vals.push(self.eval_expr(a, linha)?);
                    }
                }
                match nome.as_str() {
                    "cos" => Ok(Valor::Num(vals[0].num(linha)?.cos())),
                    "sin" => Ok(Valor::Num(vals[0].num(linha)?.sin())),
                    "tan" => Ok(Valor::Num(vals[0].num(linha)?.tan())),
                    "sqrt" => Ok(Valor::Num(vals[0].num(linha)?.max(0.0).sqrt())),
                    "abs" => Ok(Valor::Num(vals[0].num(linha)?.abs())),
                    "floor" => Ok(Valor::Num(vals[0].num(linha)?.floor())),
                    "vec2" => {
                        if vals.len() != 2 {
                            return Err(self.eval_err(
                                "vec2 espera 2 argumentos (x, y)",
                                linha,
                            ));
                        }
                        Ok(Valor::Vec(GVec2::new(
                            vals[0].num(linha)?,
                            vals[1].num(linha)?,
                        )))
                    }
                    "lerp" => {
                        // lerp(a, b, t) = a + (b - a)*t
                        if vals.len() != 3 {
                            return Err(self.eval_err("lerp espera 3 argumentos (a, b, t)", linha));
                        }
                        let (a, b, t) = (vals[0].num(linha)?, vals[1].num(linha)?, vals[2].num(linha)?);
                        Ok(Valor::Num(a + (b - a) * t))
                    }
                    "map" => {
                        // map(v, fromA, toA, fromB, toB) reescala v do intervalo
                        // [fromA, toA] para [fromB, toB].
                        if vals.len() != 5 {
                            return Err(self.eval_err(
                                "map espera 5 argumentos (v, fromA, toA, fromB, toB)",
                                linha,
                            ));
                        }
                        let nums: Vec<f32> = vals
                            .iter()
                            .map(|v| v.num(linha))
                            .collect::<Result<_, _>>()?;
                        let (v, from_a, to_a, from_b, to_b) =
                            (nums[0], nums[1], nums[2], nums[3], nums[4]);
                        let t = if (to_a - from_a).abs() < 1e-9 {
                            0.0
                        } else {
                            (v - from_a) / (to_a - from_a)
                        };
                        Ok(Valor::Num(from_b + (to_b - from_b) * t))
                    }
                    "ease" => {
                        // ease(x, "tipo") aplica uma curva de easing a x (0..1).
                        if vals.len() != 2 {
                            return Err(self.eval_err(
                                "ease espera 2 argumentos (x, \"tipo\")",
                                linha,
                            ));
                        }
                        let x = vals[0].num(linha)?;
                        let tipo = match &args[1] {
                            Expr::Str(s) => s.clone(),
                            _ => {
                                return Err(self.eval_err(
                                    "ease: o 2º argumento deve ser uma string (ex.: \"quad\")",
                                    linha,
                                ))
                            }
                        };
                        Ok(Valor::Num(ease_x(&tipo, x)))
                    }
                    "osc" => {
                        // osc(freq, amp, offset) = amp * sin(2π * freq * t + offset)
                        if vals.len() != 3 {
                            return Err(self.eval_err(
                                "osc espera 3 argumentos (freq, amp, offset)",
                                linha,
                            ));
                        }
                        let (freq, amp, off) = (
                            vals[0].num(linha)?,
                            vals[1].num(linha)?,
                            vals[2].num(linha)?,
                        );
                        Ok(Valor::Num(
                            amp * (self.t * std::f32::consts::TAU * freq + off).sin(),
                        ))
                    }
                    "noise" => {
                        let (x, y) = if vals.len() >= 2 {
                            (vals[0].num(linha)?, vals[1].num(linha)?)
                        } else {
                            (vals[0].num(linha)?, 0.0)
                        };
                        Ok(Valor::Num(if vals.len() >= 2 {
                            simplex2(self.seed, x, y)
                        } else {
                            simplex1(self.seed, x)
                        }))
                    }
                    // Random determinístico (0..1) ou rand(min,max). A mesma
                    // seed do nó produz a mesma sequência => export reprodutível.
                    "rand" => {
                        if vals.is_empty() {
                            Ok(Valor::Num(self.prox_rand()))
                        } else if vals.len() == 2 {
                            let a = vals[0].num(linha)?;
                            let b = vals[1].num(linha)?;
                            Ok(Valor::Num(a + (b - a) * self.prox_rand()))
                        } else {
                            return Err(self.eval_err(
                                "rand espera 0 ou 2 argumentos (rand ou rand(min,max))",
                                linha,
                            ));
                        }
                    }
                    _ => {
                        // Função definida pelo usuário.
                        if let Some((params, corpo)) = self.funcs.get(nome).cloned() {
                            if vals.len() != params.len() {
                                return Err(self.eval_err(
                                    format!(
                                        "função '{}' espera {} argumento(s), recebeu {}",
                                        nome,
                                        params.len(),
                                        vals.len()
                                    ),
                                    linha,
                                ));
                            }
                            // Cria um escopo novo preservando `t`, `i` e as
                            // variáveis do chamador (semântica de closure).
                            let mut escopo = self.vars.clone();
                            for (p, v) in params.iter().zip(vals.iter()) {
                                escopo.insert(p.clone(), *v);
                            }
                            let salvo = std::mem::replace(&mut self.vars, escopo);
                            let flow = self.run(&corpo);
                            self.vars = salvo;
                            return match flow {
                                Ok(Flow::Return(v)) => Ok(v),
                                Ok(Flow::Normal) => Ok(Valor::Num(0.0)),
                                Err(e) => Err(e),
                            };
                        }
                        Err(self.eval_err(format!("função '{nome}' desconhecida"), linha))
                    }
                }
            }
        }
    }

    /// Conveniência: avalia uma expressão e exige que seja escalar (f32).
    fn eval_expr_num(&mut self, e: &Expr, linha: usize) -> Result<f32, DslError> {
        self.eval_expr(e, linha)?.num(linha)
    }
}

/// Ruído 1D determinístico (baseado em seed) no intervalo [-1, 1].
fn simplex1(seed: u32, x: f32) -> f32 {
    (seed.wrapping_mul(374761393) as f32 + x * 13.37).sin()
}

/// Aplica uma curva de easing a `x` (esperado em [0, 1]). Suporta os tipos
/// `linear`, `quad`/`quadin`, `quadout`, `cubic`, `cubicin`/`cubicout`,
/// `quart`, `quint`, `expo`, `circ`, `sine`/`sin`, `back`, `elastic`, `bounce`.
/// Variações `*in` (ease-in) e `*out` (ease-out); sem sufixo = ease-in-out.
fn ease_x(tipo: &str, x: f32) -> f32 {
    // normaliza para [0,1] para evitar comportamentos estranhos fora do intervalo
    let x = x.clamp(0.0, 1.0);
    let (base, modo) = if let Some(resto) = tipo.strip_suffix("in") {
        (resto.trim_end_matches('_'), ModoEase::In)
    } else if let Some(resto) = tipo.strip_suffix("out") {
        (resto.trim_end_matches('_'), ModoEase::Out)
    } else {
        (tipo, ModoEase::InOut)
    };
    let f = match base {
        "linear" => Box::new(|t: f32| t) as Box<dyn Fn(f32) -> f32>,
        "quad" => Box::new(|t: f32| t * t),
        "cubic" => Box::new(|t: f32| t * t * t),
        "quart" => Box::new(|t: f32| t * t * t * t),
        "quint" => Box::new(|t: f32| t * t * t * t * t),
        "expo" => Box::new(|t: f32| if t >= 1.0 { 1.0 } else { (2.0f32).powf(10.0 * t - 10.0) }),
        "circ" => Box::new(|t: f32| 1.0 - (1.0 - t * t).sqrt()),
        "sine" | "sin" => Box::new(|t: f32| 1.0 - ((t * std::f32::consts::PI).cos())),
        "back" => Box::new(|t: f32| {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t * t * t - c1 * t * t
        }),
        "elastic" => Box::new(|t: f32| {
            if t <= 0.0 || t >= 1.0 {
                t
            } else {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                (2.0f32).powf(10.0 * t - 10.0) * ((t * 10.0 - 10.0 - c4) * c4).sin()
            }
        }),
        "bounce" => Box::new(|t: f32| bounce_out(t)),
        _ => Box::new(|t: f32| t),
    };
    match modo {
        ModoEase::In => f(x),
        ModoEase::Out => 1.0 - f(1.0 - x),
        ModoEase::InOut => {
            if x < 0.5 {
                f(2.0 * x) / 2.0
            } else {
                1.0 - f(2.0 * (1.0 - x)) / 2.0
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ModoEase {
    In,
    Out,
    InOut,
}

/// Curva bounce-out (parte de implementações clássicas de easing).
fn bounce_out(x: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if x < 1.0 / d1 {
        n1 * x * x
    } else if x < 2.0 / d1 {
        let x = x - 1.5 / d1;
        n1 * x * x + 0.75
    } else if x < 2.5 / d1 {
        let x = x - 2.25 / d1;
        n1 * x * x + 0.9375
    } else {
        let x = x - 2.625 / d1;
        n1 * x * x + 0.984375
    }
}

/// Resolve nomes de cores comuns (case-insensitive) para `Color32`.
fn cor_nome(nome: &str) -> Option<Color32> {
    let n = nome.to_ascii_lowercase();
    let c = match n.as_str() {
        "red" | "vermelho" => (220, 40, 50),
        "green" | "verde" => (40, 200, 80),
        "blue" | "azul" => (50, 90, 220),
        "yellow" | "amarelo" => (240, 210, 40),
        "cyan" | "ciano" => (40, 200, 220),
        "magenta" => (220, 60, 200),
        "white" | "branco" => (240, 240, 245),
        "black" | "preto" => (20, 20, 25),
        "orange" | "laranja" => (240, 130, 40),
        "purple" | "roxo" => (160, 80, 220),
        "pink" | "rosa" => (240, 130, 180),
        "gray" | "grey" | "cinza" => (140, 140, 150),
        _ => return None,
    };
    Some(Color32::from_rgb(c.0, c.1, c.2))
}

/// Ruído 2D determinístico (baseado em seed) no intervalo [-1, 1].
fn simplex2(seed: u32, x: f32, y: f32) -> f32 {
    let a = (seed.wrapping_mul(374761393) as f32 + x * 13.37 + y * 7.13).sin();
    let b = (seed.wrapping_mul(668265263) as f32 + y * 19.21 - x * 5.77).cos();
    (a + b) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_estrela_padrao() {
        let p = Program::parse(PEN_EXEMPLO_REF).expect("estrela deve parsear");
        let cmds = p.eval(0.0, 1).expect("avaliacao ok");
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Fill(true))));
    }


    #[test]
    fn erro_sintaxe_reporta_linha() {
        let r = Program::parse("move 1 2\nline 3\n");
        assert!(matches!(r, Err(DslError::Parse { linha: 2, .. })));
    }

    #[test]
    fn onda_senoidal_gera_pontos() {
        let codigo = "repeat 10 { let a = i * 36 \n line (cos(a)) (sin(a)) }";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        let linhas = cmds.iter().filter(|c| matches!(c, PathCmd::Line(_))).count();
        assert_eq!(linhas, 10);
    }

    #[test]
    fn tempo_anima() {
        let codigo = "circle 0 0 (100 + sin(t)*20)";
        let p = Program::parse(codigo).unwrap();
        let a = p.eval(0.0, 1).unwrap();
        let b = p.eval(1.57, 1).unwrap();
        assert_ne!(a.len(), 0);
        assert_eq!(a.len(), b.len());
    }

    #[test]
    fn if_else_executa_ramo() {
        // condição válida (1) deve desenhar o círculo (Move+Lines+Close);
        // senão, o retângulo. Verificamos a presença de 'Close' (path fechado)
        // e ausência de comandos quando o ramo é pulado.
        let ok = "if 1 { circle 0 0 50 } else { rect 0 0 10 10 }";
        let p = Program::parse(ok).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));

        let nao = "if 0 { circle 0 0 50 } else { rect 0 0 10 10 }";
        let p = Program::parse(nao).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));
    }

    #[test]
    fn comparacoes_retornam_01() {
        let codigo = "let a = (3 > 2)\n if a { circle 0 0 10 }";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));
    }

    #[test]
    fn if_falso_nao_gera_path() {
        // ramo falso e sem else: nenhum comando de path deve ser emitido.
        let codigo = "if (2 > 3) { circle 0 0 10 }";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.is_empty());
    }

    #[test]
    fn if_comparacao_anima() {
        // mostra cor diferente conforme metade do ciclo de t
        let codigo = "\
if (t < 1) { color 1 0 0 } else { color 0 0 1 }
circle 0 0 40";
        let p = Program::parse(codigo).unwrap();
        let a = p.eval(0.0, 1).unwrap();
        let b = p.eval(2.0, 1).unwrap();
        assert!(a.iter().any(|c| matches!(c, PathCmd::Color(_))));
        assert!(b.iter().any(|c| matches!(c, PathCmd::Color(_))));
    }

    #[test]
    fn for_varre_intervalo() {
        let codigo = "for k in 0..4 { line k 0 }";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        let linhas = cmds.iter().filter(|c| matches!(c, PathCmd::Line(_))).count();
        assert_eq!(linhas, 4);
    }

    #[test]
    fn while_com_limite() {
        let codigo = "let n = 0\nwhile (n < 3) { line n 0 \n let n = n + 1 }";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        let linhas = cmds.iter().filter(|c| matches!(c, PathCmd::Line(_))).count();
        assert_eq!(linhas, 3);
    }

    #[test]
    fn logica_and_or() {
        let codigo = "let a = (1 and 1)\n let b = (1 and 0)\n let c = (0 or 1)";
        let p = Program::parse(codigo).unwrap();
        let _ = p.eval(0.0, 1).unwrap();
        // não deve dar erro; apenas checa parse/avaliação
    }

    #[test]
    fn modulo_funciona() {
        let codigo = "let a = 10 % 3";
        let p = Program::parse(codigo).unwrap();
        let _ = p.eval(0.0, 1).unwrap();
    }

    #[test]
    fn noise_2d_aceita_dois_args() {
        let codigo = "let n = noise(1.0, 2.0)";
        let p = Program::parse(codigo).unwrap();
        let _ = p.eval(0.0, 1).unwrap();
    }

    #[test]
    fn color_nao_engole_proxima_linha() {
        // Regressão: `color r g b` seguido de outro comando NÃO deve consumir
        // o comando seguinte como 4º argumento.
        let codigo = "color 1 0.2 0.4\nfill on\nclose";
        let p = Program::parse(codigo).expect("deve parsear");
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Fill(true))));
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));

        // e o nome de cor seguido de comando também
        let codigo2 = "color red\ncircle 0 0 10";
        Program::parse(codigo2).expect("nome + comando");
    }

    #[test]
    fn cor_por_nome_e_rgba() {
        let nome = "color red";
        let p = Program::parse(nome).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Color(_))));

        let rgba = "color 1 0 0 0.5";
        let p = Program::parse(rgba).unwrap();
        let _ = p.eval(0.0, 1).unwrap();
    }

    #[test]
    fn cores_separadas_stroke_fill() {
        let codigo = "stroke_color 1 0 0\nfill_color 0 0 1\ncircle 0 0 10";
        let p = Program::parse(codigo).expect("deve parsear");
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::ColorStroke(_))));
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::ColorFill(_))));
        assert!(!cmds.iter().any(|c| matches!(c, PathCmd::Color(_))));

        // `color` (sem sufixo) continua emitindo o comando combinado
        let combo = Program::parse("color red").unwrap().eval(0.0, 1).unwrap();
        assert!(combo.iter().any(|c| matches!(c, PathCmd::Color(_))));
    }

    #[test]
    fn funcao_com_parametros_e_return() {
        // define função soma e usa no desenho
        let codigo = "\
fn soma(a, b) { return a + b }
let r = soma(3, 4)
circle 0 0 r";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        // raio 7 -> círculo deve gerar 64 segmentos + move + close
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));
    }

    #[test]
    fn funcao_sem_return_retorna_zero() {
        let codigo = "\
fn f() { let x = 5 }
let v = f()
if (v == 0) { circle 0 0 10 }";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));
    }

    #[test]
    fn funcao_aceita_seed_e_t_como_closure() {
        // função usa t do chamador (semântica de closure)
        let codigo = "\
fn pulso(x) { return x + t }
let y = pulso(10)
line 0 y";
        let p = Program::parse(codigo).unwrap();
        let a = p.eval(0.0, 1).unwrap();
        let b = p.eval(2.0, 1).unwrap();
        // ambos devem avaliar e produzir a linha; y muda com t
        assert!(a.iter().any(|c| matches!(c, PathCmd::Line(_))));
        assert!(b.iter().any(|c| matches!(c, PathCmd::Line(_))));
    }

    #[test]
    fn return_fora_de_funcao_erro() {
        let codigo = "return 1";
        let r = Program::parse(codigo).unwrap().eval(0.0, 1);
        assert!(r.is_err());
    }

    #[test]
    fn comando_text_gera_pathcmd() {
        // `text "ola" x y` deve emitir um PathCmd::Text com a cor atual.
        let codigo = "color 1 0 0\ntext \"Ola\" 100 200 64";
        let p = Program::parse(codigo).expect("deve parsear");
        let cmds = p.eval(0.0, 1).expect("avalia");
        let txt = crate::dsl::extrair_textos(&cmds);
        assert_eq!(txt.len(), 1);
        assert_eq!(txt[0].conteudo, "Ola");
        assert_eq!(txt[0].x, 100.0);
        assert_eq!(txt[0].y, 200.0);
        assert_eq!(txt[0].tamanho, 64.0);
    }

    #[test]
    fn comando_text_sem_size_usa_padrao() {
        let codigo = "text \"hi\" 0 0";
        let p = Program::parse(codigo).expect("deve parsear");
        let cmds = p.eval(0.0, 1).expect("avalia");
        let txt = crate::dsl::extrair_textos(&cmds);
        assert_eq!(txt.len(), 1);
        assert_eq!(txt[0].tamanho, 48.0);
    }

    #[test]
    fn comando_text_flags_e_animado() {
        // flags bold/italic e posição que depende de t
        let codigo = "text \"oi\" (sin(t)*50) 10 32 bold italic";
        let p = Program::parse(codigo).expect("deve parsear");
        let a = p.eval(0.0, 1).expect("avalia");
        let b = p.eval(1.0, 1).expect("avalia");
        let ta = crate::dsl::extrair_textos(&a);
        let tb = crate::dsl::extrair_textos(&b);
        assert!(ta[0].negrito && ta[0].italico);
        assert_ne!(ta[0].x, tb[0].x);
    }

    #[test]
    fn funcao_chamada_antes_definicao() {
        // registro prévio permite chamar função declarada depois
        let codigo = "\
let v = dobro(21)
fn dobro(x) { return x * 2 }
circle 0 0 v";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));
    }

    // referência local ao exemplo padrão (evita dependência de nodes)
    const PEN_EXEMPLO_REF: &str = "\
# estrela de 5 pontas
let ra = 200
let rb = 80
move 0 (-ra)
repeat 5 {
  let a = i * 72
  line (cos(a)*ra) (sin(a)*ra)
  let b = a + 36
  line (cos(b)*rb) (sin(b)*rb)
}
close
fill on
color 0.78 0.47 0.08
";

    #[test]
    fn primitivas_geram_path() {
        let casos = [
            "polygon 6 0 0 80",
            "star 5 0 0 110 50",
            "arc 0 270 150 0 0",
            "round_rect -160 120 320 90 24",
            "grid 8 1 -200 250 50 0 6",
        ];
        for c in casos {
            let p = Program::parse(c).expect("primitiva deve parsear");
            let cmds = p.eval(0.0, 1).expect("avaliacao ok");
            assert!(
                cmds.iter().any(|c| matches!(c, PathCmd::Move(_))),
                "primitiva '{c}' deve emitir Move"
            );
        }
    }

    #[test]
    fn polygon_fecha_caminho() {
        let p = Program::parse("polygon 4 0 0 50").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(cmds.iter().any(|c| matches!(c, PathCmd::Close)));
    }

    #[test]
    fn rand_e_deterministico() {
        // Mesma seed => mesma sequência de rand().
        let codigo = "let a = rand(-100, 100)\nlet b = rand(-100, 100)\ncircle a b 10";
        let p1 = Program::parse(codigo).unwrap();
        let p2 = Program::parse(codigo).unwrap();
        let c1 = p1.eval(0.0, 42).unwrap();
        let c2 = p2.eval(0.0, 42).unwrap();
        assert_eq!(c1, c2, "rand deve ser determinístico para a mesma seed");
        // Seeds diferentes => resultados distintos (quase sempre).
        let c3 = Program::parse(codigo).unwrap().eval(0.0, 7).unwrap();
        assert_ne!(c1, c3, "seeds diferentes devem dar sequências diferentes");
    }

    #[test]
    fn text_aceita_alinhamento_e_rotacao() {
        let codigo = "text \"x\" 0 0 40 bold align center rot 30";
        let p = Program::parse(codigo).expect("deve parsear");
        let cmds = p.eval(0.0, 1).unwrap();
        match &cmds[0] {
            PathCmd::Text { alinhamento, rotacao, .. } => {
                assert!(matches!(alinhamento, TextoAlinhamento::Center));
                assert_eq!(*rotacao, 30.0);
            }
            _ => panic!("esperado PathCmd::Text"),
        }
    }

    #[test]
    fn menos_unario_em_argumento() {
        // `move 0 -10` deve ser x=0, y=-10 (e não subtração 0 - 10).
        let p = Program::parse("move 0 -10").expect("deve parsear");
        let cmds = p.eval(0.0, 1).expect("avaliacao");
        match &cmds[0] {
            PathCmd::Move(g) => {
                assert_eq!(*g, GVec2::new(0.0, -10.0), "y deve ser -10");
            }
            _ => panic!("esperado Move"),
        }
        // move -5 -10 também funciona
        let p = Program::parse("move -5 -10").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(matches!(cmds[0], PathCmd::Move(GVec2 { x: -5.0, y: -10.0 })));
    }

    #[test]
    fn point_e_line_to_geram_path() {
        let p = Program::parse("point 10 20\nline_to 30 40").expect("deve parsear");
        let cmds = p.eval(0.0, 1).expect("avaliacao");
        assert!(matches!(cmds[0], PathCmd::Move(_)));
        assert!(matches!(cmds[1], PathCmd::Line(_)));
    }

    #[test]
    fn curve_to_gera_bezier() {
        let p = Program::parse("point 0 0\ncurve_to 10 10 20 0 30 30").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(matches!(cmds[1], PathCmd::Bezier(_, _, _)));
    }

    #[test]
    fn expressao_complexa_sem_let() {
        // sin(t*2 + i) aninhado em parênteses, sem precisar de `let`.
        let codigo = "let i = 3\ncircle 0 0 (100 + sin(t*2 + i)*20)";
        let p = Program::parse(codigo).expect("deve parsear");
        let _ = p.eval(1.0, 1).expect("avaliacao");
    }

    #[test]
    fn mais_binario_dentro_de_arg() {
        // `line x1 + 50 (sin(t)*30)` — o '+' é binário dentro do 1º arg.
        let p = Program::parse("let x1 = 10\nline x1 + 50 (sin(t)*30)").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(matches!(cmds[0], PathCmd::Line(_)));
    }

    #[test]
    fn lerp_e_map_funcionam() {
        let p = Program::parse("let a = lerp(0, 100, 0.5)\nlet b = map(50, 0, 100, 0, 200)").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        // só checamos que avaliou sem erro (lerp=50, map=100)
        let _ = cmds;
        let p = Program::parse("let x = lerp(0, 10, 0.25)").unwrap();
        let _ = p.eval(0.0, 1).unwrap();
    }

    #[test]
    fn atribuicao_direta_sem_let() {
        // px = px + 10 deve atualizar a variável existente.
        let codigo = "\
let px = 0
px = px + 10
px = px + 5
move px 0";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        match &cmds[0] {
            PathCmd::Move(g) => assert_eq!(g.x, 15.0, "px deve ser 15"),
            _ => panic!("esperado Move"),
        }
    }

    #[test]
    fn translate_rotate_scale_transformam_pontos() {
        // translate(100,0) move o ponto (0,0) para (100,0)
        let p = Program::parse("translate 100 0\nmove 0 0").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        match &cmds[0] {
            PathCmd::Move(g) => assert_eq!(g.x, 100.0),
            _ => panic!("esperado Move"),
        }
        // scale(2,2) dobra as coordenadas
        let p = Program::parse("scale 2 2\nmove 10 0").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        match &cmds[0] {
            PathCmd::Move(g) => assert_eq!(g.x, 20.0),
            _ => panic!("esperado Move"),
        }
    }

    #[test]
    fn push_pop_restaura_estado() {
        // translate dentro de push/pop não vaza para fora
        let codigo = "\
push
translate 100 0
pop
move 0 0";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        match &cmds[0] {
            PathCmd::Move(g) => assert_eq!(g.x, 0.0, "pop deve restaurar transform"),
            _ => panic!("esperado Move"),
        }
    }

    #[test]
    fn snake_gera_caminho_serpenteante() {
        let p = Program::parse("snake 0 0 200 8").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        let linhas = cmds.iter().filter(|c| matches!(c, PathCmd::Line(_))).count();
        assert_eq!(linhas, 8, "snake deve emitir 8 segmentos");
        assert!(matches!(cmds[0], PathCmd::Move(_)));
    }

    #[test]
    fn i_fora_de_repeat_vale_zero() {
        // `i` usado fora de repeat deve valer 0 (sem erro).
        let p = Program::parse("let x = i * 10\nline x 0").unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        assert!(matches!(cmds[0], PathCmd::Line(_)));
    }

    #[test]
    fn vec2_e_campos_x_y() {
        // `pos = vec2(100, sin(t)*50)` e `line pos.x pos.y`.
        let codigo = "let pos = vec2(100, sin(t)*50)\nline pos.x pos.y";
        let p = Program::parse(codigo).unwrap();
        let _ = p.eval(1.0, 1).unwrap();
    }

    #[test]
    fn funcao_retorna_vec2() {
        // função definida pelo usuário retornando vetor.
        let codigo = "\
fn ponto(x, y) { return vec2(x, y) }
let p = ponto(3, 4)
line p.x p.y";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        match &cmds[0] {
            PathCmd::Line(g) => {
                assert_eq!(g.x, 3.0);
                assert_eq!(g.y, 4.0);
            }
            _ => panic!("esperado Line"),
        }
    }

    #[test]
    fn for_com_passo() {
        // for i in 0..100 step 5 deve iterar 20 vezes (0,5,...,95).
        let codigo = "\
let c = 0
for i in 0..100 step 5 {
  let c = c + 1
}
line c 0";
        let p = Program::parse(codigo).unwrap();
        let cmds = p.eval(0.0, 1).unwrap();
        // c deve valer 20
        match &cmds[0] {
            PathCmd::Line(g) => assert_eq!(g.x, 20.0),
            _ => panic!("esperado Line"),
        }
    }

    #[test]
    fn cor_hex_com_e_sem_alpha() {
        // color #ff5500 -> (1, 0x55/255, 0); color #ff550088 -> com alpha.
        let p = Program::parse("color #ff5500\ncircle 0 0 5").unwrap();
        let _ = p.eval(0.0, 1).unwrap();
        let p = Program::parse("stroke_color #ff550088\nline 0 0").unwrap();
        let _ = p.eval(0.0, 1).unwrap();
    }

    #[test]
    fn variaveis_implicitas_animacao() {
        // t, phase, beat, progress são implícitas e variam com o tempo.
        let p = Program::parse("let a = t\nlet b = phase\nlet c = beat\nlet d = progress\nline 0 0").unwrap();
        let _ = p.eval(0.0, 1).unwrap();
        // progress em t=3 com duração 6 deve valer 0.5
        let cmds = p.eval_dur(3.0, 1, 6.0).unwrap();
        assert!(matches!(cmds[0], PathCmd::Line(_)));
    }

    #[test]
    fn ease_e_osc_funcionam() {
        // ease(x, "tipo") deve estar em [0,1] para x em [0,1].
        let p = Program::parse("let e = ease(0.5, \"quad\")\nline e 0").unwrap();
        let _ = p.eval(0.0, 1).unwrap();
        // osc(freq, amp, offset) deve produzir um valor em [-amp, amp].
        let p = Program::parse("let o = osc(1, 50, 0)\nline o 0").unwrap();
        let _ = p.eval(0.25, 1).unwrap(); // t=0.25 => sin(π/2)=1 => 50
        let cmds = p.eval(0.25, 1).unwrap();
        match &cmds[0] {
            PathCmd::Line(g) => assert!((g.x - 50.0).abs() < 1e-3, "osc deve dar 50"),
            _ => panic!("esperado Line"),
        }
    }

    #[test]
    fn ease_tipos_conhecidos() {
        for tipo in ["linear", "quad", "quadin", "quadout", "cubic", "sine", "expo", "bounce", "elastic", "back"] {
            let codigo = format!("let e = ease(0.5, \"{tipo}\")\nline e 0");
            let p = Program::parse(&codigo).unwrap();
            let _ = p.eval(0.0, 1).unwrap();
        }
    }
}
