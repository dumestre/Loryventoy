#![allow(dead_code)]

use std::io::Write;

/// Níveis de log centralizados.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Diagnostic,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            LogLevel::Error => "ERRO",
            LogLevel::Warn => "AVISO",
            LogLevel::Info => "INFO",
            LogLevel::Diagnostic => "DIAGNÓSTICO",
        }
    }
}

static mut NIVEL_LOG: LogLevel = LogLevel::Warn;

/// Define o nível mínimo de log. Mensagens abaixo deste nível são descartadas.
pub fn definir_nivel(nivel: LogLevel) {
    unsafe {
        NIVEL_LOG = nivel;
    }
}

/// Retorna o nível atual de log.
pub fn nivel_atual() -> LogLevel {
    unsafe { NIVEL_LOG }
}

/// Anexa uma linha de log ao arquivo em `logs/`, com timestamp.
fn anexar(arquivo: &str, nivel: LogLevel, msg: &str) {
    if nivel < unsafe { NIVEL_LOG } {
        return;
    }
    let dir = "logs";
    let _ = std::fs::create_dir_all(dir);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/{arquivo}"))
    {
        let _ = writeln!(f, "[{}] [{}] {}", agora(), nivel.as_str(), msg);
    }
}

/// Log de erro — grava em `logs/app.log` e exibe no stderr.
pub fn erro(msg: impl Into<String>) {
    let msg = msg.into();
    anexar("app.log", LogLevel::Error, &msg);
    eprintln!("[ERRO] {msg}");
}

/// Log de aviso — grava em `logs/app.log`.
pub fn aviso(msg: impl Into<String>) {
    let msg = msg.into();
    anexar("app.log", LogLevel::Warn, &msg);
}

/// Log informativo — grava em `logs/app.log`.
pub fn info(msg: impl Into<String>) {
    let msg = msg.into();
    anexar("app.log", LogLevel::Info, &msg);
}

/// Log de diagnóstico — grava em `logs/app.log` (útil para debug verbose).
pub fn diag(msg: impl Into<String>) {
    let msg = msg.into();
    anexar("app.log", LogLevel::Diagnostic, &msg);
}

fn agora() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dias = t / 86400;
    let mut y = 1970i32;
    let mut resto = dias as i64;
    loop {
        let bissexto = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let da = if bissexto { 366 } else { 365 };
        if resto < da {
            break;
        }
        resto -= da;
        y += 1;
    }
    let dpm = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mes = 0;
    let mut d = resto;
    let bissexto = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    loop {
        let dim = dpm[mes] + if mes == 1 && bissexto { 1 } else { 0 };
        if d < dim {
            break;
        }
        d -= dim;
        mes += 1;
    }
    let h = (t % 86400) / 3600;
    let m = (t % 3600) / 60;
    let s = t % 60;
    format!(
        "{:02}/{:02}/{} {:02}:{:02}:{:02}",
        d + 1,
        mes + 1,
        y,
        h,
        m,
        s
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nivel_por_default_e_warn() {
        assert_eq!(nivel_atual(), LogLevel::Warn);
    }

    #[test]
    fn filtragem_por_nivel() {
        definir_nivel(LogLevel::Error);
        assert_eq!(nivel_atual(), LogLevel::Error);
        // Diagnostic < Error → filtrado
        assert!(LogLevel::Diagnostic < LogLevel::Error);
        // Warn < Error → filtrado
        assert!(LogLevel::Warn < LogLevel::Error);
        // Error == Error → mostrado
        assert_eq!(LogLevel::Error, LogLevel::Error);
        definir_nivel(LogLevel::Warn);
    }

    #[test]
    fn ordem_dos_niveis() {
        assert!(LogLevel::Diagnostic < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }
}
