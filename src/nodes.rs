use eframe::egui::Color32;

/// Tipos de nó disponíveis no editor, com nome e cor em português.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoNo {
    /// Saída final do grafo (master).
    Saida,
    /// Nó de transform (posição, rotação, escala).
    Transform,
    /// Configurações do projeto (canvas, resolução, fps, duração).
    Canvas,
    /// Cena: cria/renomeia a cena e define qual é a cena ativa da sequência.
    Cena,
    /// Camadas de objetos de uma cena.
    Layer,
    /// Formas geométricas de uma cena.
    Shape,
    /// Texto procedural (rótulos/títulos), rasterizado com `cosmic-text`
    /// e desenhado no preview como elemento da cena.
    Texto,
    /// Nó de desenho procedural via DSL (caneta): o usuário escreve um
    /// programa em mini-linguagem que gera formas vetoriais no preview.
    Pen,
    /// Nó de ruído: gera um deslocamento animado (simplex/FBM) que pode ser
    /// conectado a um parâmetro de outro nó (ex.: Posição/Rotação) para
    /// animá-lo de forma orgânica.
    Ruido,
    /// Nó de animação: função por trechos (keyframes procedurais) que
    /// produz um valor no tempo `t` e pode ser conectada a um parâmetro de
    /// outro nó (Posição/Rotação/Escala/Opacidade).
    Anim,
}

impl TipoNo {
    pub fn nome(&self) -> &'static str {
        match self {
            TipoNo::Saida => "Master",
            TipoNo::Transform => "Transform",
            TipoNo::Canvas => "Canvas",
            TipoNo::Cena => "Cena",
            TipoNo::Layer => "Layers",
            TipoNo::Shape => "Shape",
            TipoNo::Texto => "Texto",
            TipoNo::Pen => "Pen",
            TipoNo::Ruido => "Ruído",
            TipoNo::Anim => "Animação",
        }
    }

    pub fn cor(&self) -> Color32 {
        match self {
            TipoNo::Saida => Color32::from_rgb(120, 220, 140),
            TipoNo::Transform => Color32::from_rgb(235, 185, 95),
            TipoNo::Canvas => Color32::from_rgb(170, 120, 235),
            TipoNo::Cena => Color32::from_rgb(90, 190, 190),
            TipoNo::Layer => Color32::from_rgb(120, 170, 235),
            TipoNo::Shape => Color32::from_rgb(235, 150, 120),
            TipoNo::Texto => Color32::from_rgb(150, 200, 120),
            TipoNo::Pen => Color32::from_rgb(200, 120, 220),
            TipoNo::Ruido => Color32::from_rgb(120, 200, 220),
            TipoNo::Anim => Color32::from_rgb(230, 130, 170),
        }
    }

    /// Recupera o tipo a partir do rótulo do nó no grafo.
    pub fn from_label(label: &str) -> Option<TipoNo> {
        match label {
            "Master" => Some(TipoNo::Saida),
            "Transform" => Some(TipoNo::Transform),
            "Canvas" => Some(TipoNo::Canvas),
            "Cena" => Some(TipoNo::Cena),
            "Layers" => Some(TipoNo::Layer),
            "Shape" => Some(TipoNo::Shape),
            "Texto" => Some(TipoNo::Texto),
            "Pen" => Some(TipoNo::Pen),
            "Ruído" | "Ruido" => Some(TipoNo::Ruido),
            "Animação" | "Animacao" => Some(TipoNo::Anim),
            _ => None,
        }
    }

    /// Retorna uma instância válida deste tipo (usado ao criar um nó a partir
    /// da toolbar). Como `TipoNo` é apenas uma etiqueta, retorna o próprio
    /// variant (os dados ficam em `NodeParams`).
    pub fn instancia(&self) -> TipoNo {
        *self
    }

    /// Regra de compatibilidade: o tipo de origem pode alimentar o de destino?
    pub fn pode_conectar(origem: TipoNo, destino: TipoNo) -> bool {
        match (origem, destino) {
            // Saída é o sumidouro final: não tem saída utilizável
            (TipoNo::Saida, _) => false,
            // Qualquer nó pode alimentar a Saída
            (_, TipoNo::Saida) => true,
            // Canvas -> Cena (fluxo do projeto; Canvas -> Saída já vale por (_ , Saída))
            (TipoNo::Canvas, TipoNo::Cena) => true,
            (TipoNo::Cena, TipoNo::Cena) => true,
            // Layers e Shape alimentam a Cena a que pertencem
            (TipoNo::Layer, TipoNo::Cena) => true,
            (TipoNo::Shape, TipoNo::Cena) => true,
            (TipoNo::Texto, TipoNo::Cena) => true,
            // Nó Pen também pertence a uma cena (geometria procedural).
            (TipoNo::Pen, TipoNo::Cena) => true,
            // Ruído modula um parâmetro (Posição/Rotação/Escala) de nós
            // dirigíveis: alimenta Transform/Shape/Texto/Pen.
            (
                TipoNo::Ruido,
                TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen,
            ) => true,
            // Animação modula um parâmetro (Posição/Rotação/Escala/Opacidade)
            // de nós dirigíveis.
            (
                TipoNo::Anim,
                TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen,
            ) => true,
            // Parâmetro -> parâmetro: nós dirigíveis (Transform/Shape/Texto/Pen)
            // podem receber a saída de outro nó desses tipos.
            (
                o @ (TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen),
                TipoNo::Transform | TipoNo::Shape | TipoNo::Texto | TipoNo::Pen,
            ) if o != TipoNo::Saida => true,
            _ => false,
        }
    }
}

/// Tipo de um porto de conexão: escalar (1 valor, ex.: Tamanho, Cor) ou
/// vetor (vários componentes nomeados, ex.: Posição = X/Y/Z). Ao soltar um
/// fio de um porto vetor, abre-se um menu para escolher o componente.
#[derive(Clone, Copy, Debug)]
pub enum TipoPorto {
    Escalar,
    Vetor(&'static [&'static str]),
}

/// Parâmetro exposto como porto de conexão (entrada ou saída). O índice do
/// componente escolhido é guardado na aresta (`ArestaInfo.saida_comp`).
#[derive(Clone, Copy, Debug)]
pub struct ParametroPorto {
    pub nome: &'static str,
    pub tipo: TipoPorto,
}

impl ParametroPorto {
    /// Número de componentes escalares (1 para escalar, N para vetor).
    pub fn n_componentes(&self) -> usize {
        match &self.tipo {
            TipoPorto::Escalar => 1,
            TipoPorto::Vetor(c) => c.len(),
        }
    }

    /// É um porto vetorial (vários componentes)?
    pub fn is_vetor(&self) -> bool {
        matches!(self.tipo, TipoPorto::Vetor(_))
    }

    /// Nome do componente `k` (o próprio `nome` se escalar).
    pub fn componente(&self, k: usize) -> &'static str {
        match &self.tipo {
            TipoPorto::Escalar => self.nome,
            TipoPorto::Vetor(c) => c.get(k).copied().unwrap_or(self.nome),
        }
    }
}

/// Componentes vetoriais reutilizáveis.
const COMP_XYZ: &[&str] = &["X", "Y", "Z"];
const COMP_XY: &[&str] = &["X", "Y"];

/// Portos canônicos (estáticos) para compor as especificações.
static P_CANVAS: ParametroPorto = ParametroPorto {
    nome: "Canvas",
    tipo: TipoPorto::Escalar,
};
static P_CENA: ParametroPorto = ParametroPorto {
    nome: "Cena",
    tipo: TipoPorto::Escalar,
};
static P_LAYER: ParametroPorto = ParametroPorto {
    nome: "Layer",
    tipo: TipoPorto::Escalar,
};
static P_PEN: ParametroPorto = ParametroPorto {
    nome: "Pen",
    tipo: TipoPorto::Escalar,
};
static P_POS_XYZ: ParametroPorto = ParametroPorto {
    nome: "Posição",
    tipo: TipoPorto::Vetor(COMP_XYZ),
};
static P_ROT_XYZ: ParametroPorto = ParametroPorto {
    nome: "Rotação",
    tipo: TipoPorto::Vetor(COMP_XYZ),
};
static P_ESC_XYZ: ParametroPorto = ParametroPorto {
    nome: "Escala",
    tipo: TipoPorto::Vetor(COMP_XYZ),
};
static P_POS_XY: ParametroPorto = ParametroPorto {
    nome: "Posição",
    tipo: TipoPorto::Vetor(COMP_XY),
};
static P_LARGURA: ParametroPorto = ParametroPorto {
    nome: "Largura",
    tipo: TipoPorto::Escalar,
};
static P_ALTURA: ParametroPorto = ParametroPorto {
    nome: "Altura",
    tipo: TipoPorto::Escalar,
};
static P_ROT: ParametroPorto = ParametroPorto {
    nome: "Rotação",
    tipo: TipoPorto::Escalar,
};
static P_COR: ParametroPorto = ParametroPorto {
    nome: "Cor",
    tipo: TipoPorto::Escalar,
};
static P_TAMANHO: ParametroPorto = ParametroPorto {
    nome: "Tamanho",
    tipo: TipoPorto::Escalar,
};
static P_RUIDO_OUT: ParametroPorto = ParametroPorto {
    nome: "Ruído",
    tipo: TipoPorto::Vetor(COMP_XY),
};
static P_ANIM_OUT: ParametroPorto = ParametroPorto {
    nome: "Animação",
    tipo: TipoPorto::Vetor(COMP_XY),
};
static P_OPACIDADE: ParametroPorto = ParametroPorto {
    nome: "Opacidade",
    tipo: TipoPorto::Escalar,
};
static P_ESC_XY: ParametroPorto = ParametroPorto {
    nome: "Escala",
    tipo: TipoPorto::Vetor(COMP_XY),
};

/// Especificações por tipo/lado (estáticas, referenciadas por `PortSpec`).
static SAIDAS_TRANSFORM: [ParametroPorto; 3] = [P_POS_XYZ, P_ROT_XYZ, P_ESC_XYZ];
static ENTRADAS_TRANSFORM: [ParametroPorto; 3] = [P_POS_XYZ, P_ROT_XYZ, P_ESC_XYZ];
static ENTRADAS_SHAPE: [ParametroPorto; 6] =
    [P_CANVAS, P_POS_XY, P_LARGURA, P_ALTURA, P_ROT, P_COR];
static SAIDAS_SHAPE: [ParametroPorto; 5] = [P_POS_XY, P_LARGURA, P_ALTURA, P_ROT, P_COR];
static SAIDAS_TEXTO: [ParametroPorto; 3] = [P_POS_XY, P_TAMANHO, P_COR];
static ENTRADAS_SAIDA: [ParametroPorto; 1] = [P_CENA];
static ENTRADAS_CENA: [ParametroPorto; 1] = [P_CANVAS];
static SAIDAS_CENA: [ParametroPorto; 1] = [P_CENA];
static ENTRADAS_LAYER: [ParametroPorto; 1] = [P_CENA];
static SAIDAS_LAYER: [ParametroPorto; 1] = [P_LAYER];
static ENTRADAS_PEN: [ParametroPorto; 2] = [P_CANVAS, P_POS_XY];
static SAIDAS_PEN: [ParametroPorto; 2] = [P_PEN, P_POS_XY];
static SAIDAS_RUIDO: [ParametroPorto; 1] = [P_RUIDO_OUT];
static ENTRADAS_TEXTO: [ParametroPorto; 6] =
    [P_CANVAS, P_POS_XY, P_TAMANHO, P_COR, P_OPACIDADE, P_ESC_XY];
static SAIDAS_ANIM: [ParametroPorto; 1] = [P_ANIM_OUT];

/// Especificação de portos de um nó: rótulos de entrada (esquerda) e de
/// saída (direita), na ordem vertical em que aparecem no card. Usada para
/// desenhar uma "bolinha" (porto) por parâmetro, alinhada ao corpo.
pub struct PortSpec {
    pub entradas: &'static [ParametroPorto],
    pub saidas: &'static [ParametroPorto],
}

/// Recupera o porto de saída `i` de um tipo (se existir).
pub fn porto_saida(tipo: TipoNo, i: usize) -> Option<&'static ParametroPorto> {
    portos(tipo).saidas.get(i)
}

/// Recupera o porto de entrada `i` de um tipo (se existir).
#[allow(dead_code)]
pub fn porto_entrada(tipo: TipoNo, i: usize) -> Option<&'static ParametroPorto> {
    portos(tipo).entradas.get(i)
}

/// Portos de entrada/saída de cada tipo de nó. As entradas são os
/// parâmetros que o nó consome; as saídas, os parâmetros que ele produz e
/// que podem ser enviados a outro nó (ex.: Posição, Tamanho, Cor).
pub fn portos(tipo: TipoNo) -> PortSpec {
    match tipo {
        TipoNo::Saida => PortSpec {
            entradas: &ENTRADAS_SAIDA,
            saidas: &[],
        },
        TipoNo::Transform => PortSpec {
            entradas: &ENTRADAS_TRANSFORM,
            saidas: &SAIDAS_TRANSFORM,
        },
        TipoNo::Canvas => PortSpec {
            entradas: &[],
            saidas: &[],
        },
        TipoNo::Cena => PortSpec {
            entradas: &ENTRADAS_CENA,
            saidas: &SAIDAS_CENA,
        },
        TipoNo::Layer => PortSpec {
            entradas: &ENTRADAS_LAYER,
            saidas: &SAIDAS_LAYER,
        },
        TipoNo::Shape => PortSpec {
            entradas: &ENTRADAS_SHAPE,
            saidas: &SAIDAS_SHAPE,
        },
        TipoNo::Texto => PortSpec {
            entradas: &ENTRADAS_TEXTO,
            saidas: &SAIDAS_TEXTO,
        },
        TipoNo::Pen => PortSpec {
            entradas: &ENTRADAS_PEN,
            saidas: &SAIDAS_PEN,
        },
        TipoNo::Ruido => PortSpec {
            entradas: &[],
            saidas: &SAIDAS_RUIDO,
        },
        TipoNo::Anim => PortSpec {
            entradas: &[],
            saidas: &SAIDAS_ANIM,
        },
    }
}

/// Configurações do projeto, editadas pelo nó Canvas.
#[derive(Clone, Copy, Debug)]
pub struct ProjetoConfig {
    pub largura: u32,
    pub altura: u32,
    pub fps: f32,
    pub duracao_seg: f32,
    pub fundo: Color32,
}

impl Default for ProjetoConfig {
    fn default() -> Self {
        Self {
            largura: 1920,
            altura: 1080,
            fps: 30.0,
            duracao_seg: 5.0,
            fundo: Color32::WHITE,
        }
    }
}

/// Parâmetros editáveis de cada nó, em painel estilo inspector
/// (rótulos alinhados em coluna + campos), como Unity / Cavalry.
#[derive(Clone, Debug)]
pub enum NodeParams {
    /// Nó de transform (posição, rotação, escala).
    Transform {
        px: f32, py: f32, pz: f32,
        rx: f32, ry: f32, rz: f32,
        sx: f32, sy: f32, sz: f32,
    },
    /// Nó de cena: define nome da cena e se é a cena ativa da sequência.
    Cena {
        nome_cena: String,
        ativa: bool,
        zoom: f32,
        angulo: f32,
        opacidade: f32,
    },
    /// Nó de camadas de uma cena.
    Layer {
        cena: String,
        opacidade: f32,
    },
    /// Texto procedural (rótulos/títulos), rasterizado com `cosmic-text`
    /// e desenhado no preview como elemento da cena.
    Texto {
        cena: String,
        conteudo: String,
        tamanho: f32,
        negrito: bool,
        italico: bool,
        px: f32,
        py: f32,
        cor: Color32,
    },
    /// Nó de forma geométrica de uma cena (100% procedural).
    Shape {
        cena: String,
        tipo: u8, // 0=retângulo,1=elipse,2=triângulo,3=estrela,4=losango,5=polígono,6=seta
        px: f32, py: f32,
        largura: f32, altura: f32,
        rotacao: f32,
        cor: Color32,
        // parâmetros procedurais (ruído + seed)
        seed: f32,
        noise_scale: f32,
        amp: f32,
        veloc: f32,
    },
    /// Nó de desenho procedural via DSL (caneta). O usuário escreve um
    /// programa em mini-linguagem que gera formas vetoriais na cena.
    Pen {
        cena: String,
        codigo: String,
        erro: Option<String>,
        cor: Color32,
        cor_fill: Color32,
        pos_x: f32,
        pos_y: f32,
        espessura: f32,
        preenchimento: bool,
        seed: f32,
        cantos: f32,
        ordem: f32,
        escala_x: f32,
        escala_y: f32,
    },
    /// Nó de ruído: gera um deslocamento animado (FBM) para modular um
    /// parâmetro de outro nó conectado. `alvo` escolhe o parâmetro padrão
    /// (0=Posição, 1=Rotação, 2=Escala) quando conectado a um porto ambíguo.
    Ruido {
        seed: f32,
        freq: f32,
        amp: f32,
        veloc: f32,
        alvo: u8,
    },
    /// Nó de animação: função por trechos (keyframes procedurais). `alvo`
    /// (0=Posição, 1=Rotação, 2=Escala, 3=Opacidade) e a lista de segmentos.
    Anim {
        alvo: u8,
        loop_mode: u8,
        segmentos: Vec<crate::procedural::AnimSeg>,
    },
    /// Nó de saída (ajustes de pós: brilho, contraste, saturação).
    Saida { brilho: f32, contraste: f32, saturacao: f32 },
    /// Nó Canvas: configurações do projeto.
    Canvas(ProjetoConfig),
}

impl NodeParams {
    /// Parâmetros iniciais (em zero) para um tipo de nó.
    pub fn padrao(tipo: TipoNo) -> NodeParams {
        match tipo {
            TipoNo::Saida => NodeParams::Saida {
                brilho: 1.0,
                contraste: 1.0,
                saturacao: 1.0,
            },
            TipoNo::Transform => NodeParams::Transform {
                px: 0.0, py: 0.0, pz: 0.0,
                rx: 0.0, ry: 0.0, rz: 0.0,
                sx: 1.0, sy: 1.0, sz: 1.0,
            },
            TipoNo::Canvas => NodeParams::Canvas(ProjetoConfig::default()),
            TipoNo::Cena => NodeParams::Cena {
                nome_cena: "Cena 1".to_string(),
                ativa: true,
                zoom: 1.0,
                angulo: 0.0,
                opacidade: 1.0,
            },
            TipoNo::Layer => NodeParams::Layer {
                cena: String::new(),
                opacidade: 1.0,
            },
            TipoNo::Shape => NodeParams::Shape {
                cena: String::new(),
                tipo: 0,
                px: 960.0,
                py: 540.0,
                largura: 200.0,
                altura: 200.0,
                rotacao: 0.0,
                cor: Color32::from_rgb(235, 150, 120),
                seed: 1.0,
                noise_scale: 0.6,
                amp: 0.0,
                veloc: 0.0,
            },
            TipoNo::Texto => NodeParams::Texto {
                cena: String::new(),
                conteudo: "Texto".to_string(),
                tamanho: 48.0,
                negrito: false,
                italico: false,
                px: 960.0,
                py: 540.0,
                cor: Color32::from_rgb(20, 20, 26),
            },
            TipoNo::Ruido => NodeParams::Ruido {
                seed: 1.0,
                freq: 0.6,
                amp: 50.0,
                veloc: 1.0,
                alvo: 0,
            },
            TipoNo::Anim => NodeParams::Anim {
                alvo: 0,
                loop_mode: 0,
                segmentos: vec![crate::procedural::AnimSeg {
                    t_ini: 0.0,
                    t_fim: 1.0,
                    v_ini: [960.0, 540.0],
                    v_fim: [960.0, 540.0],
                    easing: crate::procedural::Easing::EaseInOut,
                }],
            },
            TipoNo::Pen => NodeParams::Pen {
                cena: String::new(),
                codigo: PEN_EXEMPLO.to_string(),
                erro: None,
                cor: Color32::from_rgb(200, 120, 220),
                cor_fill: Color32::from_rgb(200, 120, 220),
                pos_x: 960.0,
                pos_y: 540.0,
                espessura: 3.0,
                preenchimento: true,
                seed: 1.0,
                cantos: 0.0,
                ordem: 0.0,
                escala_x: 1.0,
                escala_y: 1.0,
            },
        }
    }
}

/// Exemplo de código DSL exibido por padrão no nó Pen (estrela de 5 pontas).
pub const PEN_EXEMPLO: &str = "\
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
