use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Versao {
    pub numero: &'static str,
    pub titulo: &'static str,
    pub itens: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct ProjetoInfo {
    pub nome: String,
    pub caminho: String,
    pub modified: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjetoArquivo {
    pub versao: u32,
    pub script_text: String,
    pub nos: Vec<serde_json::Value>,
    pub arestas: Vec<serde_json::Value>,
}

pub const VERSAO_ATUAL: &str = "12.5";

pub const VERSOES: &[Versao] = &[
    Versao {
        numero: "12.5",
        titulo: "Hub de Projetos",
        itens: &[
            "Tela inicial com gerenciador de projetos",
            "Criar, abrir, excluir projetos",
            "Escolha manual da pasta de projetos",
        ],
    },
    Versao {
        numero: "12.4",
        titulo: "Exportação",
        itens: &[
            "Melhorias no pipeline de exportação",
            "Ajustes de performance no preview",
        ],
    },
    Versao {
        numero: "12.3",
        titulo: "Ícones e UI",
        itens: &[
            "Novos ícones SVG",
            "Painel de grupos de nós",
        ],
    },
    Versao {
        numero: "12.2",
        titulo: "Timeline",
        itens: &[
            "Scrub de keyframes",
            "Loop configurável por trecho",
        ],
    },
    Versao {
        numero: "12.1",
        titulo: "Editor de Script (DSL)",
        itens: &[
            "Janela de script com exemplos",
            "Aplicar via Ctrl+Enter",
        ],
    },
    Versao {
        numero: "12",
        titulo: "Grafo de Nós",
        itens: &[
            "Editor de grafo baseado em nós",
            "Cenas, formas, textos e canetas",
        ],
    },
    Versao {
        numero: "11.9",
        titulo: "Preview em Tempo Real",
        itens: &["Renderização ao vivo sincronizada"],
    },
    Versao {
        numero: "11.8",
        titulo: "Motor Procedural",
        itens: &["Sistema de ruído e animação paramétrica"],
    },
    Versao {
        numero: "10.9",
        titulo: "Primeira Versão Estável",
        itens: &["Lançamento inicial do Loryventoy"],
    },
];

#[allow(dead_code)]
pub enum HubAction {
    SelectPage(Page),
    SetQuery(String),
    SetSort(SortBy),
    Refresh,
    OpenProject(String),
    DeleteProject(String),
    NewProject,
    CreateProject(String),
    ConfirmDelete,
    CancelDelete,
    InstallVersion(String),
    UninstallVersion(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum Page {
    Projetos,
    Instalacoes,
    Sobre,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum SortBy {
    Nome,
    Data,
    Tamanho,
}

#[allow(dead_code)]
pub struct HubState {
    pub current_page: Page,
    pub projetos: Vec<ProjetoInfo>,
    pub installed_versions: Vec<String>,
    pub query: String,
    pub sort: SortBy,
    pub show_new_modal: bool,
    pub new_project_name: String,
    pub pasta_projetos: String,
    pub pasta_instalacoes: String,
    pub delete_target: Option<String>,
    pub show_toast: Option<String>,
    pub open_project: Option<ProjetoArquivo>,
    pub install_status: HashMap<String, String>,
    pub install_progress: HashMap<String, f32>,
    pub install_size: HashMap<String, String>,
}

impl HubState {
    pub fn new() -> Self {
        let mut s = Self {
            current_page: Page::Projetos,
            projetos: Vec::new(),
            installed_versions: vec![VERSAO_ATUAL.to_string()],
            query: String::new(),
            sort: SortBy::Data,
            show_new_modal: false,
            new_project_name: String::new(),
            pasta_projetos: String::from("."),
            pasta_instalacoes: String::from("."),
            delete_target: None,
            show_toast: None,
            open_project: None,
            install_status: HashMap::new(),
            install_progress: HashMap::new(),
            install_size: HashMap::new(),
        };
        s.refresh_projects();
        s
    }

    pub fn refresh_projects(&mut self) {
        let pasta = self.pasta_projetos.clone();
        self.projetos.clear();
        let dir = PathBuf::from(pasta);
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut nomes: Vec<ProjetoInfo> = Vec::new();
        for ent in entries.flatten() {
            let path = ent.path();
            if path.extension().and_then(|e| e.to_str()) == Some("lory") {
                if let Some(nome) = path.file_name().and_then(|f| f.to_str()) {
                    let meta = fs::metadata(&path).ok();
                    let modified = meta
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .map(fmt_time)
                        .unwrap_or_else(|| "---".to_string());
                    let size = meta.map(|m| m.len()).unwrap_or(0);
                    nomes.push(ProjetoInfo {
                        nome: nome.to_string(),
                        caminho: path.to_string_lossy().to_string(),
                        modified,
                        size_bytes: size,
                    });
                }
            }
        }
        match self.sort {
            SortBy::Nome => nomes.sort_by(|a, b| a.nome.cmp(&b.nome)),
            SortBy::Data => nomes.sort_by(|a, b| b.modified.cmp(&a.modified)),
            SortBy::Tamanho => nomes.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes)),
        }
        self.projetos = nomes;
    }

    pub fn open_project(&mut self, caminho: &str) {
        let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("lory-hub.exe"));

        // Procura Loryventoy.exe caminhando para cima a partir do exe do hub.
        // Funciona tanto para execução standalone (lory-hub/target/debug/)
        // quanto como workspace member (target/debug/).
        let mut dir = current_exe.parent();
        let mut exe = PathBuf::from("Loryventoy.exe");
        for _ in 0..6 {
            if let Some(d) = dir {
                for target_dir in &["target/debug", "target/release"] {
                    let candidate = d.join(target_dir).join("Loryventoy.exe");
                    if candidate.exists() {
                        exe = candidate;
                        break;
                    }
                }
                if exe != PathBuf::from("Loryventoy.exe") {
                    break;
                }
                dir = d.parent();
            } else {
                break;
            }
        }

        let child = std::process::Command::new(&exe)
            .arg(caminho)
            .spawn();

        if child.is_ok() {
            std::process::exit(0);
        }
    }

    pub fn create_project(&mut self, nome: &str) {
        let base = if nome.trim().is_empty() { "projeto" } else { nome.trim() };
        let nome_file = format!("{base}.lory");
        let path = PathBuf::from(&self.pasta_projetos).join(&nome_file);
        if path.exists() {
            return;
        }
        let data = ProjetoArquivo {
            versao: 1,
            script_text: format!("project \"{base}\" {{ width 1920 height 1080 fps 30 duration 8 background #1c191e }}\n"),
            nos: vec![],
            arestas: vec![],
        };
        let json = match serde_json::to_string_pretty(&data) {
            Ok(j) => j,
            Err(_) => return,
        };
        let _ = fs::write(&path, json);
        
        // Abre o projeto e sai
        self.open_project(path.to_str().unwrap_or(""));
    }

    pub fn delete_project(&mut self, caminho: &str) {
        let _ = fs::remove_file(caminho);
        self.delete_target = None;
        self.refresh_projects();
    }

    pub fn install_version(&mut self, numero: &str) {
        if !self.installed_versions.contains(&numero.to_string()) {
            self.installed_versions.push(numero.to_string());
        }
    }

    pub fn uninstall_version(&mut self, numero: &str) {
        self.installed_versions.retain(|x| x != numero);
    }
}

fn fmt_time(t: std::time::SystemTime) -> String {
    let dur = match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let s = dur.as_secs();
    let d = s / 86400;
    let mut y = 1970i32;
    let mut r = d as i64;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        if r < if leap { 366 } else { 365 } { break; }
        r -= if leap { 366 } else { 365 };
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    let mut d = r;
    loop {
        let max = dim[m] + if m == 1 && leap { 1 } else { 0 };
        if d < max { break; }
        d -= max;
        m += 1;
    }
    let h = (s % 86400) / 3600;
    let min = (s % 3600) / 60;
    format!("{:02}/{:02}/{} {:02}:{:02}", d + 1, m + 1, y, h, min)
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{bytes} B") }
    else if bytes < 1024 * 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
}