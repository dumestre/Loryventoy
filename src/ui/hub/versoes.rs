/// Histórico de versões do Loryventoy (software, não do projeto).
/// Ordem: mais recente primeiro. Exibido na tela inicial do Hub.
pub struct Versao {
    pub numero: &'static str,
    pub titulo: &'static str,
    pub itens: &'static [&'static str],
}

pub const VERSAO_ATUAL: &str = "12.5";

pub const VERSOES: &[Versao] = &[
    Versao {
        numero: "12.5",
        titulo: "Hub de Projetos",
        itens: &[
            "Tela inicial com gerenciador de projetos",
            "Criação, abertura, duplicação e exclusão de projetos",
            "Escolha manual da pasta de projetos",
        ],
    },
    Versao {
        numero: "12.4",
        titulo: "Exportação",
        itens: &[
            "Melhorias no pipeline de exportação de vídeo",
            "Ajustes de performance no preview",
        ],
    },
    Versao {
        numero: "12.3",
        titulo: "Ícones e UI",
        itens: &[
            "Novos ícones SVG na barra de ferramentas",
            "Painel de grupos de nós",
        ],
    },
    Versao {
        numero: "12.2",
        titulo: "Timeline",
        itens: &[
            "Scrub de keyframes na linha do tempo",
            "Loop configurável por trecho",
        ],
    },
    Versao {
        numero: "12.1",
        titulo: "Editor de Script (DSL)",
        itens: &[
            "Janela de script com exemplos embutidos",
            "Aplicar via Ctrl+Enter",
        ],
    },
    Versao {
        numero: "12",
        titulo: "Grafo de Nós",
        itens: &[
            "Editor de grafo baseado em nós",
            "Cenas, formas, textos e canetas procedurais",
        ],
    },
    Versao {
        numero: "11.9",
        titulo: "Preview em Tempo Real",
        itens: &["Renderização ao vivo sincronizada com a timeline"],
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