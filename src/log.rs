#![allow(dead_code)]

/// Registro simples de avisos/erros em arquivo, para que mensagens que
/// aparecem na UI (em vermelho) possam ser inspecionadas e copiadas fora
/// do app (já que a UI do egui não permite seleção de texto nesses rótulos).
use std::io::Write;

/// Anexa uma linha de aviso a `arquivo`, com timestamp simples.
fn anexar(arquivo: &str, nivel: &str, msg: &str) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(arquivo)
    {
        let _ = writeln!(f, "[{}] {} {}", agora(), nivel, msg);
    }
}

/// Aviso do Hub de Projetos (grava em `hub.log`).
pub fn hub(nivel: &str, msg: &str) {
    anexar("hub.log", nivel, msg);
    eprintln!("[hub] {} {}", nivel, msg);
}

/// Aviso do app / editor (grava em `app.log`).
pub fn app(nivel: &str, msg: &str) {
    anexar("app.log", nivel, msg);
    eprintln!("[app] {} {}", nivel, msg);
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
    format!("{:02}/{:02}/{} {:02}:{:02}:{:02}", d + 1, mes + 1, y, h, m, s)
}
