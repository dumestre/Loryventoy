use std::collections::{HashMap, HashSet};

use eframe::egui::{
    Area, Button, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Id,
    Key, Order, Painter, PointerButton, Popup, Pos2, Rect, Sense, Shape, Stroke,
    StrokeKind, Ui, UiBuilder, Vec2,
};
use eframe::egui::epaint::{CircleShape, CubicBezierShape, RectShape, TextShape};
use egui_graphs::{
    Graph, GraphView, MetadataFrame, SettingsInteraction, SettingsNavigation,
    SettingsStyle,
};
use petgraph::{
    stable_graph::StableGraph, stable_graph::NodeIndex, Directed,
    stable_graph::EdgeIndex,
};

use crate::nodes::{porto_saida, NodeParams, ProjetoConfig, TipoNo};
use crate::ui::graph_toolbar::{AcaoToolbar, GraphToolbar};
use crate::ui::node_component;
use crate::ui::scroll_delta;

/// Limites de zoom da view (fator de escala). Máximo baixo para os nós não
/// ficarem gigantes ao aproximar; mínimo para não sumirem ao afastar.
const ZOOM_MIN: f32 = 0.2;
const ZOOM_MAX: f32 = 1.2;

/// Exibição customizada dos nós (card/arestas) e a lógica de grupos.
pub mod node_display;
pub mod selection;
pub mod groups;

/// Estado do menu de escolha de componente (X/Y/Z) aberto ao soltar o fio
/// de um porto de saída vetorial. O usuário escolhe qual componente da
/// "linha" de parâmetro é enviado ao destino.
struct MenuComponentes {
    src: NodeIndex,
    saida: usize,
    drop_screen: Pos2, // posição de tela onde o popup é aberto
    drop_canvas: Pos2, // posição em canvas do soltar (p/ recomputar alvo)
    alvo: Option<(NodeIndex, usize)>, // entrada mais próxima no soltar
    escolha: Option<usize>,           // componente escolhido (255 = cancelar)
    rect: Option<Rect>,               // retângulo do popup (p/ detectar clique fora)
}

pub struct GraphPanel {
    g: Graph<
        (),
        node_display::ArestaInfo,
        Directed,
        petgraph::stable_graph::DefaultIx,
        node_display::NoDisplay,
        node_display::ArestaCurva,
    >,
    pub toolbar: GraphToolbar,
    contador: usize,
    id_grafo: Option<String>,
    // interação de conexão/remoção de arestas
    conexao: Option<(NodeIndex, usize, Pos2)>, // (nó de origem, porto de saída, cursor em canvas)
    // menu de escolha de componente (X/Y/Z) aberto ao soltar um
    // porto vetorial. Guarda a origem, o porto e o ponto de drop.
    menu_componentes: Option<MenuComponentes>,
    arrastando: Option<(NodeIndex, Vec2)>, // (nó, offset canvas)
    aresta_hover: Option<EdgeIndex>,             // aresta sob o cursor (feedback)
    master: Option<NodeIndex>,                  // nó mestre fixo (Saída)
    master_loc: Pos2,                          // posição fixa do mestre (canvas)
    canvas: Option<NodeIndex>,                  // nó Canvas (config. do projeto)
    canvas_loc: Pos2,                         // posição fixa inicial do Canvas
    cena: Option<NodeIndex>,                    // nó Cena
    cena_loc: Pos2,                           // posição fixa inicial da Cena
    liberados: HashSet<NodeIndex>,              // nós já movidos (posição solta)
    params: HashMap<NodeIndex, NodeParams>,    // parâmetros editáveis de cada nó
    pan_meio: bool,                            // pan com botão do meio ativo
    selecao: Option<(Pos2, Pos2)>,             // caixa de seleção (início, atual) em canvas
    grupos: Vec<groups::Grupo>,                // grupos (surfaces) de nós
    grupo_seq: usize,                          // sequência p/ título/cor de grupo
    clipboard: Vec<selection::NoCopia>,        // área de transferência de nós
    menu_canvas: Pos2,                         // posição (canvas) do último botão direito
    arrastando_grupo: Option<(usize, Pos2)>,   // (grupo, cursor canvas anterior)
    undo_stack: Vec<(Vec<(TipoNo, Pos2, NodeParams)>, Vec<(usize, usize, node_display::ArestaInfo)>)>,
    redo_stack: Vec<(Vec<(TipoNo, Pos2, NodeParams)>, Vec<(usize, usize, node_display::ArestaInfo)>)>,
    // ids DSL persistentes (nome -> nó), para a DSL de patch localizar nós.
    dsl_ids: HashMap<String, NodeIndex>,
}

impl GraphPanel {
    pub fn new() -> Self {
        let raw: StableGraph<(), node_display::ArestaInfo> = StableGraph::new();
        let mut panel = Self {
            g: Graph::from(&raw),
            toolbar: GraphToolbar::new(),
            contador: 0,
            id_grafo: Some("graph".into()),
            conexao: None,
            menu_componentes: None,
            arrastando: None,
            aresta_hover: None,
            master: None,
            master_loc: Pos2::ZERO,
            canvas: None,
            canvas_loc: Pos2::ZERO,
            cena: None,
            cena_loc: Pos2::ZERO,
            liberados: HashSet::new(),
            params: HashMap::new(),
            pan_meio: false,
            selecao: None,
            grupos: Vec::new(),
            grupo_seq: 0,
            clipboard: Vec::new(),
            menu_canvas: Pos2::ZERO,
            arrastando_grupo: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dsl_ids: HashMap::new(),
        };
        panel.criar_nos_padrao();
        panel
    }

    fn criar_nos_padrao(&mut self) {
        // projeto sempre abre com 3 nós conectados em sequência:
        // Canvas -> Cena -> Master (Saída)
        self.g = Graph::from(&StableGraph::<(), node_display::ArestaInfo>::new());
        self.liberados.clear();
        self.params.clear();
        self.grupos.clear();
        // Espaçamento igual entre os 3 nós (lado a lado, sem sobreposição).
        // O cartão tem o dobro da largura de `content_size`, então o
        // espaço entre centros deve ser maior que a largura do cartão.
        const ESPACO: f32 = 290.0;
        self.canvas_loc = Pos2::new(-ESPACO, 0.0);
        self.cena_loc = Pos2::new(0.0, 0.0);
        self.master_loc = Pos2::new(ESPACO, 0.0);

        let canvas = self.adicionar_no_em(TipoNo::Canvas, self.canvas_loc);
        let cena = self.adicionar_no_em(TipoNo::Cena, self.cena_loc);
        let master = self.adicionar_no_em(TipoNo::Saida, self.master_loc);

        self.g.add_edge(
            canvas,
            cena,
            node_display::ArestaInfo {
                saida: 0,
                saida_comp: None,
                entrada: 0,
                entrada_comp: None,
            },
        );
        self.g.add_edge(
            cena,
            master,
            node_display::ArestaInfo {
                saida: 0,
                saida_comp: None,
                entrada: 0,
                entrada_comp: None,
            },
        );

        self.canvas = Some(canvas);
        self.cena = Some(cena);
        self.master = Some(master);

        // ids DSL padrão para a linguagem de patch localizar os nós fixos.
        self.dsl_ids.clear();
        self.dsl_ids.insert("canvas".to_string(), canvas);
        self.dsl_ids.insert("scene".to_string(), cena);
        self.dsl_ids.insert("master".to_string(), master);
    }

    fn adicionar_no_em(&mut self, tipo: TipoNo, loc: Pos2) -> NodeIndex {
        let idx = self.g.add_node_with_label_and_location((), tipo.nome().to_string(), loc);
        if let Some(n) = self.g.node_mut(idx) {
            n.set_color(tipo.cor());
        }
        self.params.insert(idx, NodeParams::padrao(tipo));
        // Layers/Shape já nascem vinculados à primeira cena existente
        let cenas = self.cenas_disponiveis();
        self.normalizar_cena(idx, &cenas);
        idx
    }

    fn adicionar_no(&mut self, tipo: TipoNo) {
        // só pode existir um mestre
        if tipo == TipoNo::Saida && self.master.is_some() {
            return;
        }
        // Posiciona em grade: colunas de ~260px (canvas) com 3 por linha,
        // evitando empilhar todos no mesmo canto.
        let col = (self.contador % 3) as f32;
        let lin = (self.contador / 3) as f32;
        let loc = Pos2::new(40.0 + col * 260.0, 40.0 + lin * 150.0);
        self.adicionar_no_em(tipo, loc);
        self.contador += 1;
    }

    /// Cria um nó de Texto + um nó Animação já conectado, aplicando um dos
    /// modelos de animação pré-definidos (Fade In, Slide, Bounce, Zoom...).
    /// O nó Animação modula a Posição (X/Y) ou Opacidade do texto conforme o
    /// modelo. `id` seleciona o preset.
    fn aplicar_modelo_anim_texto(&mut self, id: u8) {
        let offset = self.contador as f32;
        let base = Pos2::new(20.0 + offset * 26.0, 20.0);

        // Texto
        let txt = self.adicionar_no_em(TipoNo::Texto.instancia(), base);
        if let Some(NodeParams::Texto { conteudo, tamanho, .. }) =
            self.params.get_mut(&txt)
        {
            *conteudo = "Texto".to_string();
            *tamanho = 80.0;
        }

        // Animação (posicionada à direita do texto)
        let anim = self.adicionar_no_em(
            TipoNo::Anim.instancia(),
            Pos2::new(base.x + 220.0, base.y),
        );

        // Segmentos de animação conforme o modelo.
        use crate::procedural::{AnimSeg, Easing};
        let segs: Vec<AnimSeg> = match id {
            // 0: Fade In (opacidade 0 -> 1 nos primeiros 1.5s)
            0 => vec![AnimSeg {
                t_ini: 0.0,
                t_fim: 1.5,
                v_ini: [0.0, 0.0],
                v_fim: [1.0, 0.0],
                easing: Easing::EaseOut,
            }],
            // 1: Slide da esquerda (x: -600 -> 0)
            1 => vec![AnimSeg {
                t_ini: 0.0,
                t_fim: 1.2,
                v_ini: [-600.0, 0.0],
                v_fim: [0.0, 0.0],
                easing: Easing::EaseOut,
            }],
            // 2: Slide da direita (x: 600 -> 0)
            2 => vec![AnimSeg {
                t_ini: 0.0,
                t_fim: 1.2,
                v_ini: [600.0, 0.0],
                v_fim: [0.0, 0.0],
                easing: Easing::EaseOut,
            }],
            // 3: Subir (y: 300 -> 0)
            3 => vec![AnimSeg {
                t_ini: 0.0,
                t_fim: 1.0,
                v_ini: [0.0, 300.0],
                v_fim: [0.0, 0.0],
                easing: Easing::EaseOut,
            }],
            // 4: Bounce (y: 400 -> 0 com ping-pong e várias batidas)
            4 => vec![
                AnimSeg {
                    t_ini: 0.0,
                    t_fim: 0.4,
                    v_ini: [0.0, 400.0],
                    v_fim: [0.0, 0.0],
                    easing: Easing::EaseOut,
                },
                AnimSeg {
                    t_ini: 0.4,
                    t_fim: 0.6,
                    v_ini: [0.0, 0.0],
                    v_fim: [0.0, 120.0],
                    easing: Easing::EaseIn,
                },
                AnimSeg {
                    t_ini: 0.6,
                    t_fim: 0.85,
                    v_ini: [0.0, 120.0],
                    v_fim: [0.0, 0.0],
                    easing: Easing::EaseOut,
                },
                AnimSeg {
                    t_ini: 0.85,
                    t_fim: 1.0,
                    v_ini: [0.0, 0.0],
                    v_fim: [0.0, 40.0],
                    easing: Easing::EaseIn,
                },
                AnimSeg {
                    t_ini: 1.0,
                    t_fim: 1.15,
                    v_ini: [0.0, 40.0],
                    v_fim: [0.0, 0.0],
                    easing: Easing::EaseOut,
                },
            ],
            // 5: Zoom In (escala 0.2 -> 1) — usa o alvo Escala
            5 => vec![AnimSeg {
                t_ini: 0.0,
                t_fim: 1.0,
                v_ini: [0.2, 0.2],
                v_fim: [1.0, 1.0],
                easing: Easing::EaseOut,
            }],
            _ => vec![AnimSeg {
                t_ini: 0.0,
                t_fim: 1.0,
                v_ini: [0.0, 0.0],
                v_fim: [1.0, 0.0],
                easing: Easing::EaseOut,
            }],
        };

        // alvo da animação: Opacidade p/ Fade In (0), Escala p/ Zoom In (5),
        // Posição para os demais (slide/subir/bounce).
        let alvo: u8 = match id {
            0 => 3, // Opacidade
            5 => 2, // Escala
            _ => 0, // Posição
        };
        let loop_mode: u8 = if id == 4 { 1 } else { 0 }; // Bounce faz loop

        // Índice da ENTRADA do Texto que recebe a animação (ver ENTRADAS_TEXTO
        // em nodes.rs): 1=Posição, 4=Opacidade, 5=Escala.
        let entrada_texto: usize = match alvo {
            3 => 4, // Opacidade
            2 => 5, // Escala
            _ => 1, // Posição
        };

        if let Some(NodeParams::Anim { alvo: a, loop_mode: lm, segmentos }) =
            self.params.get_mut(&anim)
        {
            *a = alvo;
            *lm = loop_mode;
            *segmentos = segs;
        }

        // Conecta a saída 0 (Anim) na entrada do alvo do Texto.
        self.conectar_parametro(anim, 0, None, txt, entrada_texto, None);

        self.contador += 2;
    }

    /// Nomes de cena definidos pelos nós Cena (para o combobox de Layers/Shape).
    fn cenas_disponiveis(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .g
            .nodes_iter()
            .filter_map(|(_, n)| {
                let idx = n.id();
                if let Some(NodeParams::Cena { nome_cena, .. }) = self.params.get(&idx) {
                    if !nome_cena.is_empty() {
                        Some(nome_cena.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        v.sort();
        v.dedup();
        v
    }

    /// Garante que um nó Layers/Shape aponte para uma cena existente (a
    /// primeira da lista), caso sua cena esteja vazia ou tenha sido removida.
    fn normalizar_cena(&mut self, idx: NodeIndex, cenas: &[String]) {
        if let Some(NodeParams::Layer { cena, .. } | NodeParams::Shape { cena, .. }) =
            self.params.get_mut(&idx)
        {
            if cenas.iter().all(|c| c != cena) {
                *cena = cenas.first().cloned().unwrap_or_default();
            }
        }
    }

    /// Coleta todas as cenas (com suas formas e textos procedurais) para o
    /// preview desenhar. Coordenadas em pixels do projeto (canvas).
    /// Procura um nó Ruído conectado a uma ENTRADA do nó `alvo` e devolve o
    /// driver correspondente. O `alvo` do driver vem do porto de entrada em
    /// que o fio chega (Posição→0, Rotação→1, Escala→2); se o porto não for
    /// reconhecido, usa o `alvo` configurado no próprio nó Ruído.
    fn ruido_para(&self, alvo: NodeIndex) -> Option<crate::procedural::RuidoDriver> {
        use crate::nodes::porto_entrada;
        for (ei, _) in self.g.edges_iter() {
            let (src, dst) = match self.g.edge_endpoints(ei) {
                Some(p) => p,
                None => continue,
            };
            if dst != alvo {
                continue;
            }
            let (seed, freq, amp, veloc, alvo_no) = match self.params.get(&src) {
                Some(NodeParams::Ruido { seed, freq, amp, veloc, alvo }) => {
                    (*seed, *freq, *amp, *veloc, *alvo)
                }
                _ => continue,
            };
            // Descobre em qual parâmetro o fio chega (porto de entrada).
            let info = self.g.edge(ei).map(|e| *e.payload());
            let alvo_final = info
                .and_then(|inf| {
                    let tipo_dst = self.tipo_do_node(dst)?;
                    let p = porto_entrada(tipo_dst, inf.entrada)?;
                    Some(match p.nome {
                        "Posição" => 0u8,
                        "Rotação" => 1,
                        "Escala" | "Largura" | "Altura" => 2,
                        _ => alvo_no,
                    })
                })
                .unwrap_or(alvo_no);
            return Some(crate::procedural::RuidoDriver {
                seed,
                freq,
                amp,
                veloc,
                alvo: alvo_final,
            });
        }
        None
    }

    /// Procura um nó Animação conectado a uma ENTRADA do nó `alvo` e devolve o
    /// driver. Igual a `ruido_para`, mas para animação. O parâmetro alvo vem
    /// do porto de entrada em que o fio chega; se não reconhecido, usa o `alvo`
    /// configurado no próprio nó Animação.
    fn anim_para(&self, alvo: NodeIndex) -> Option<crate::procedural::AnimDriver> {
        use crate::nodes::porto_entrada;
        for (ei, _) in self.g.edges_iter() {
            let (src, dst) = match self.g.edge_endpoints(ei) {
                Some(p) => p,
                None => continue,
            };
            if dst != alvo {
                continue;
            }
            let (segmentos, loop_mode, alvo_no) = match self.params.get(&src) {
                Some(NodeParams::Anim { segmentos, loop_mode, alvo }) => {
                    (segmentos.clone(), *loop_mode, *alvo)
                }
                _ => continue,
            };
            let info = self.g.edge(ei).map(|e| *e.payload());
            let alvo_final = info
                .and_then(|inf| {
                    let tipo_dst = self.tipo_do_node(dst)?;
                    let p = porto_entrada(tipo_dst, inf.entrada)?;
                    Some(match p.nome {
                        "Posição" => 0u8,
                        "Rotação" => 1,
                        "Escala" | "Largura" | "Altura" => 2,
                        "Opacidade" => 3,
                        _ => alvo_no,
                    })
                })
                .unwrap_or(alvo_no);
            return Some(crate::procedural::AnimDriver {
                segmentos,
                loop_mode: crate::procedural::LoopMode::from_u8(loop_mode),
                alvo: alvo_final,
            });
        }
        None
    }

    pub fn formas_para_preview(&self) -> crate::procedural::PreviewData {
        use crate::procedural::{PreviewData, ShapeGenerator, ShapeKind, TextoItem};

        let mut data = PreviewData {
            largura: 1920.0,
            altura: 1080.0,
            fundo: Color32::WHITE,
            cenas: Vec::new(),
        };
        if let Some(NodeParams::Canvas(c)) = self.canvas.and_then(|i| self.params.get(&i)) {
            data.largura = c.largura as f32;
            data.altura = c.altura as f32;
            data.fundo = c.fundo;
        }

        // mapa nome-da-cena -> índice na lista (preserva ordem de aparecimento)
        let mut indice: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut cenas = data.cenas;
        for (idx, _) in self.g.nodes_iter() {
            let i = match self.params.get(&idx) {
                Some(NodeParams::Shape { cena, .. })
                | Some(NodeParams::Texto { cena, .. })
                | Some(NodeParams::Pen { cena, .. }) => {
                    if let Some(&i) = indice.get(cena) {
                        i
                    } else {
                        let i = cenas.len();
                        cenas.push(crate::procedural::CenaPreview {
                            opacidade: 1.0,
                            formas: Vec::new(),
                            textos: Vec::new(),
                            pen: Vec::new(),
                        });
                        indice.insert(cena.clone(), i);
                        i
                    }
                }
                _ => continue,
            };
            match self.params.get(&idx) {
                Some(NodeParams::Shape {
                    cena: _,
                    px,
                    py,
                    largura,
                    altura,
                    rotacao,
                    cor,
                    tipo,
                    seed,
                    noise_scale,
                    amp,
                    veloc,
                    ..
                }) => {
                    cenas[i].formas.push(ShapeGenerator {
                        kind: ShapeKind::from_u8(*tipo),
                        pos: glam::Vec2::new(*px, *py),
                        tam: glam::Vec2::new(*largura, *altura),
                        rot: *rotacao,
                        cor: *cor,
                        seed: *seed,
                        noise_scale: *noise_scale,
                        amp: *amp,
                        veloc: *veloc,
                        ruido: self.ruido_para(idx),
                        anim: self.anim_para(idx),
                    });
                }
                Some(NodeParams::Texto {
                    cena: _,
                    conteudo,
                    tamanho,
                    negrito,
                    italico,
                    px,
                    py,
                    cor,
                }) => {
                    cenas[i].textos.push(TextoItem {
                        px: *px,
                        py: *py,
                        conteudo: conteudo.clone(),
                        tamanho: *tamanho,
                        negrito: *negrito,
                        italico: *italico,
                        cor: *cor,
                        escala_x: 1.0,
                        escala_y: 1.0,
                        ruido: self.ruido_para(idx),
                        anim: self.anim_para(idx),
                    });
                }
                Some(NodeParams::Pen {
                    cena: _,
                    codigo,
                    cor,
                    cor_fill,
                    pos_x,
                    pos_y,
                    espessura,
                    preenchimento,
                    seed,
                    cantos,
                    ordem,
                    escala_x,
                    escala_y,
                    ..
                }) => {
                    if let Ok(program) = crate::dsl::Program::parse(codigo) {
                        cenas[i].pen.push(crate::procedural::PenPath {
                            program,
                            pos: glam::Vec2::new(*pos_x, *pos_y),
                            cor: *cor,
                            cor_fill: *cor_fill,
                            espessura: *espessura,
                            preenchimento: *preenchimento,
                            seed: *seed as u32,
                            cantos: *cantos,
                            ordem: *ordem,
                            escala_x: *escala_x,
                            escala_y: *escala_y,
                            ruido: self.ruido_para(idx),
                            anim: self.anim_para(idx),
                        });
                    }
                }
                _ => {}
            }
        }

        // opacidade vinda dos nós Layer conectados a cada Cena
        for (idx, _) in self.g.nodes_iter() {
            if let Some(NodeParams::Layer { cena, opacidade }) = self.params.get(&idx) {
                if let Some(&i) = indice.get(cena) {
                    // primeiro Layer que aparece ganha; demais ignorados
                    if (cenas[i].opacidade - 1.0).abs() < 1e-3 {
                        cenas[i].opacidade = *opacidade;
                    }
                }
            }
        }

        data.cenas = cenas;
        data
    }

    fn buscar(&mut self, termo: &str) {
        let t = termo.to_lowercase();
        if t.is_empty() {
            return;
        }
        let alvos: Vec<(NodeIndex, bool)> = self
            .g
            .nodes_iter()
            .map(|(idx, n)| (idx, n.label().to_lowercase().contains(&t)))
            .collect();
        for (idx, sel) in alvos {
            if let Some(n) = self.g.node_mut(idx) {
                n.set_selected(sel);
            }
        }
    }

    // ---- helpers de coordenadas (igual ao desenho do GraphView) ----
    fn screen_para_canvas(&self, screen: Pos2, frame: &MetadataFrame, rect: Rect) -> Pos2 {
        (screen - rect.left_top().to_vec2() - frame.pan) / frame.zoom
    }

    fn canvas_para_screen(&self, canvas: Pos2, frame: &MetadataFrame, rect: Rect) -> Pos2 {
        canvas * frame.zoom + frame.pan + rect.left_top().to_vec2()
    }

    fn node_sob_cursor(&self, p: Pos2) -> Option<NodeIndex> {
        let mut achado = None;
        for (idx, n) in self.g.nodes_iter() {
            let half = node_display::NoDisplay::tamanho(&n.label());
            if (p.x - n.location().x).abs() <= half.x && (p.y - n.location().y).abs() <= half.y {
                achado = Some(idx);
            }
        }
        achado
    }

    /// Verdadeiro se o ponto de canvas `p` está sobre a REGIÃO DO CABEÇALHO
    /// (faixa superior do card, onde fica o nome) de algum nó. Usado para
    /// mostrar o cursor de "mover" e indicar que aquele trecho arrasta o nó.
    fn sobre_cabecalho_no(&self, p: Pos2) -> Option<NodeIndex> {
        for (idx, n) in self.g.nodes_iter() {
            let half = node_display::NoDisplay::tamanho(&n.label());
            let dx = (p.x - n.location().x).abs();
            let dy_top = p.y - (n.location().y - half.y);
            if dx <= half.x && dy_top >= 0.0 && dy_top <= node_component::CABECALHO_H {
                return Some(idx);
            }
        }
        None
    }

    /// Verdadeiro se o ponto de tela `ps` está sobre a REGIÃO DE CONTEÚDO
    /// (corpo, abaixo do cabeçalho) de algum nó — ou seja, sobre os widgets
    /// editáveis (DragValue/ComboBox/etc). Usado para não capturar scroll/arraste
    /// do canvas quando o usuário está interagindo com um campo do nó.
    fn sobre_conteudo(&self, ps: Option<Pos2>, frame: &MetadataFrame, rect: Rect) -> bool {
        let Some(ps) = ps else { return false };
        for (_, n) in self.g.nodes_iter() {
            let half = node_display::NoDisplay::tamanho(&n.label());
            let center = self.canvas_para_screen(n.location(), frame, rect);
            let node_rect = Rect::from_center_size(center, half * 2.0 * frame.zoom);
            let corpo = Rect::from_min_max(
                Pos2::new(
                    node_rect.min.x,
                    node_rect.min.y + node_component::CABECALHO_H * frame.zoom,
                ),
                node_rect.max,
            );
            if corpo.contains(ps) {
                return true;
            }
        }
        false
    }

    /// Offsets (canvas, relativos ao centro) dos portos de entrada/saída de
    /// um nó, conforme seu tipo.
    fn portos_offsets(&self, idx: NodeIndex) -> Option<(Vec<Vec2>, Vec<Vec2>)> {
        let n = self.g.node(idx)?;
        let tipo = TipoNo::from_label(n.label().as_str())?;
        let half = node_display::NoDisplay::tamanho(n.label().as_str());
        Some(node_display::port_offsets(tipo, half))
    }

    /// Porto de saída mais próximo do ponto `p` (canvas), se dentro do raio.
    /// Retorna (nó, índice do porto).
    fn porta_saida_mais_proxima(&self, p: Pos2) -> Option<(NodeIndex, usize)> {
        let mut melhor: Option<(NodeIndex, usize, f32)> = None;
        for (idx, n) in self.g.nodes_iter() {
            let Some((_, outs)) = self.portos_offsets(idx) else {
                continue;
            };
            let loc = n.location();
            for (i, off) in outs.iter().enumerate() {
                let d = (loc + *off).distance(p);
                if d <= 12.0 && melhor.map_or(true, |(_, _, md)| d < md) {
                    melhor = Some((idx, i, d));
                }
            }
        }
        melhor.map(|(i, p, _)| (i, p))
    }

    /// Tipo do nó a partir do rótulo.
    fn tipo_do_node(&self, idx: NodeIndex) -> Option<TipoNo> {
        self.g.node(idx).and_then(|n| TipoNo::from_label(n.label().as_str()))
    }

    /// É o nó mestre (fixo)?
    fn is_master(&self, idx: NodeIndex) -> bool {
        self.master == Some(idx)
    }

    fn is_canvas(&self, idx: NodeIndex) -> bool {
        self.canvas == Some(idx)
    }

    /// Nós protegidos contra exclusão/cópia (master e canvas), mas
    /// ainda arrastáveis livremente pelo usuário.
    fn is_fixo(&self, idx: NodeIndex) -> bool {
        self.is_master(idx) || self.is_canvas(idx)
    }

    /// Garante exatamente um mestre: cria se faltar, remove extras.
    fn garantir_master(&mut self) {
        let mestres: Vec<NodeIndex> = self
            .g
            .nodes_iter()
            .filter(|(idx, _)| self.is_master(*idx))
            .map(|(idx, _)| idx)
            .collect();
        match mestres.len() {
            0 => {
                let m = self.adicionar_no_em(TipoNo::Saida, self.master_loc);
                self.master = Some(m);
            }
            n if n > 1 => {
                // mantém o primeiro, remove os demais
                for extra in &mestres[1..] {
                    self.g.remove_node(*extra);
                }
                self.master = Some(mestres[0]);
            }
            _ => {}
        }
    }

    /// Mantém as posições iniciais dos 3 nós do projeto até
    /// que o usuário os mova (então passam a ser livres).
    fn reafirmar_posicoes(&mut self) {
        let fixar = |idx: Option<NodeIndex>, loc: Pos2, liberados: &HashSet<NodeIndex>, g: &mut Graph<
            (),
            node_display::ArestaInfo,
            Directed,
            petgraph::stable_graph::DefaultIx,
            node_display::NoDisplay,
            node_display::ArestaCurva,
        >| {
            if let Some(i) = idx {
                if !liberados.contains(&i) {
                    if let Some(n) = g.node_mut(i) {
                        n.set_location(loc);
                    }
                }
            }
        };
        fixar(self.canvas, self.canvas_loc, &self.liberados, &mut self.g);
        fixar(self.cena, self.cena_loc, &self.liberados, &mut self.g);
        fixar(self.master, self.master_loc, &self.liberados, &mut self.g);
    }

    /// Configurações do projeto (lidas pela timeline), guardadas
    /// no nó Canvas dentro de `params`.
    pub fn projeto(&self) -> &ProjetoConfig {
        static FALLBACK: ProjetoConfig = ProjetoConfig {
            largura: 1920,
            altura: 1080,
            fps: 30.0,
            duracao_seg: 5.0,
            fundo: Color32::WHITE,
        };
        match self.canvas.and_then(|i| self.params.get(&i)) {
            Some(NodeParams::Canvas(c)) => c,
            _ => &FALLBACK,
        }
    }

    /// Snapshot serializável do grafo: lista de nós (tipo, posição, params)
    /// e lista de arestas (índices de origem/destino + info de portos).
    pub fn snapshot(&self) -> (Vec<(TipoNo, Pos2, NodeParams)>, Vec<(usize, usize, node_display::ArestaInfo)>) {
        let mut nos = Vec::new();
        for (idx, n) in self.g.nodes_iter() {
            let Some(tipo) = self.tipo_do_node(idx) else { continue };
            let Some(p) = self.params.get(&idx) else { continue };
            nos.push((tipo, n.location(), p.clone()));
        }
        let mut arestas = Vec::new();
        for (ei, _e) in self.g.edges_iter() {
            if let Some((s, d)) = self.g.edge_endpoints(ei) {
                let info = self.g.edge(ei).map(|e| *e.payload()).unwrap_or_default();
                arestas.push((s.index(), d.index(), info));
            }
        }
        (nos, arestas)
    }

    /// Reconstrói o grafo a partir de um snapshot (usado ao carregar projeto).
    /// Limpa o atual e recria nós/arestas preservando tipo, posição e params.
    pub fn carregar_snapshot(
        &mut self,
        nos: &[(TipoNo, Pos2, NodeParams)],
        arestas: &[(usize, usize, node_display::ArestaInfo)],
    ) {
        // limpa tudo e recria o esqueleto mínimo (garante master/canvas/vars)
        self.criar_nos_padrao();

        // remove os 3 nós padrão para começar do zero (recriaremos do snapshot)
        let padrao: Vec<NodeIndex> = [self.canvas, self.cena, self.master]
            .into_iter()
            .flatten()
            .collect();
        for i in &padrao {
            self.g.remove_node(*i);
        }

        let mut mapa: HashMap<usize, NodeIndex> = HashMap::new();
        for (i, (tipo, loc, params)) in nos.iter().enumerate() {
            let idx = self.adicionar_no_em(*tipo, *loc);
            // aplica os params exatos vindos do snapshot
            if let Some(p) = self.params.get_mut(&idx) {
                *p = params.clone();
            }
            // libera a posição para não ser reafirmada (fixa nos padrões)
            self.liberados.insert(idx);
            mapa.insert(i, idx);
        }

        for (de, para, info) in arestas {
            let Some(s) = mapa.get(de) else { continue };
            let Some(d) = mapa.get(para) else { continue };
            if s == d {
                continue;
            }
            self.g.add_edge(
                *s,
                *d,
                node_display::ArestaInfo {
                    saida: info.saida,
                    saida_comp: info.saida_comp,
                    entrada: info.entrada,
                    entrada_comp: info.entrada_comp,
                },
            );
        }

        self.garantir_master();
    }

    /// Limite máximo de entradas na pilha de desfazer.
    const LIMITE_HISTORICO: usize = 50;

    /// Empurra o estado atual do grafo para a pilha de undo, limpando a de redo.
    /// Chamar ANTES de cada mutação estrutural (add/remove/move/script/menu).
    pub fn empurrar_historico(&mut self) {
        let snap = self.snapshot();
        self.undo_stack.push(snap);
        if self.undo_stack.len() > Self::LIMITE_HISTORICO {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Desfaz a última ação, movendo o estado atual para a pilha de redo.
    /// Retorna `false` se não houver histórico para desfazer.
    pub fn undo(&mut self) -> bool {
        let Some(snap) = self.undo_stack.pop() else {
            return false;
        };
        let atual = self.snapshot();
        self.carregar_snapshot(&snap.0, &snap.1);
        self.redo_stack.push(atual);
        true
    }

    /// Refaz a última ação desfeita, movendo o estado atual para a pilha de undo.
    /// Retorna `false` se não houver histórico para refazer.
    pub fn redo(&mut self) -> bool {
        let Some(snap) = self.redo_stack.pop() else {
            return false;
        };
        let atual = self.snapshot();
        self.carregar_snapshot(&snap.0, &snap.1);
        self.undo_stack.push(atual);
        if self.undo_stack.len() > Self::LIMITE_HISTORICO {
            self.undo_stack.remove(0);
        }
        true
    }

    /// Verdadeiro se há algo para desfazer.
    pub fn pode_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Verdadeiro se há algo para refazer.
    pub fn pode_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Aplica um script DSL de projeto (linguagem de autoramento): reconstrói
    /// o grafo inteiro a partir do texto. Retorna erro (com linha) se o script
    /// for inválido. Veja `src/ui/project_dsl.rs`.
    pub fn aplicar_script(&mut self, codigo: &str) -> Result<(), crate::dsl::project_dsl::ScriptError> {
        use crate::dsl::project_dsl::{
            indice_porto, parse_script, tipo_da_dsl, TopLevel,
        };

        let prog = parse_script(codigo)?;

        // 1) config do projeto (Canvas)
        let mut proj = ProjetoConfig::default();
        for tl in &prog {
            if let TopLevel::Project(p) = tl {
                aplicar_project(&mut proj, p);
            }
        }

        // 2) reconstrói o esqueleto padrão (Canvas -> Cena -> Master)
        self.criar_nos_padrao();
        if let Some(ci) = self.canvas {
            if let Some(NodeParams::Canvas(c)) = self.params.get_mut(&ci) {
                *c = proj;
            }
        }

        // 3) mapa id -> NodeIndex (registra os nós padrão por tipo)
        let mut ids: HashMap<String, NodeIndex> = HashMap::new();
        if let Some(c) = self.canvas {
            ids.insert("canvas".to_string(), c);
        }
        if let Some(c) = self.cena {
            ids.insert("scene".to_string(), c);
        }
        if let Some(m) = self.master {
            ids.insert("master".to_string(), m);
        }

        // 4) cria os nós do script (exceto canvas/master que já existem)
        for tl in &prog {
            if let TopLevel::Node(n) = tl {
                let tipo = match tipo_da_dsl(&n.tipo) {
                    Some(t) => t,
                    None => continue,
                };
                if tipo == TipoNo::Canvas || tipo == TipoNo::Saida {
                    continue;
                }
                let idx = self.adicionar_no_em(tipo, Pos2::ZERO);
                ids.insert(n.id.clone(), idx);
                self.dsl_ids.insert(n.id.clone(), idx);
                aplicar_campos(self, idx, n)?;
            }
        }

        // 5) cria as arestas
        for tl in &prog {
            if let TopLevel::Edge(e) = tl {
                let src = *ids
                    .get(&e.de)
                    .ok_or_else(|| crate::dsl::project_dsl::ScriptError::Apply(
                        format!("nó '{}' não definido", e.de),
                    ))?;
                let dst = *ids
                    .get(&e.para)
                    .ok_or_else(|| crate::dsl::project_dsl::ScriptError::Apply(
                        format!("nó '{}' não definido", e.para),
                    ))?;
                let src_tipo = self.tipo_do_no(src);
                let dst_tipo = self.tipo_do_no(dst);
                let saida_i = indice_porto(src_tipo, &e.saida, true).ok_or_else(|| {
                    crate::dsl::project_dsl::ScriptError::Apply(format!(
                        "porta de saída '{}' inválida em '{}'",
                        e.saida, e.de
                    ))
                })?;
                let entrada_i = indice_porto(dst_tipo, &e.entrada, false).ok_or_else(|| {
                    crate::dsl::project_dsl::ScriptError::Apply(format!(
                        "porta de entrada '{}' inválida em '{}'",
                        e.entrada, e.para
                    ))
                })?;
                self.conectar_parametro(src, saida_i, None, dst, entrada_i, None);
            }
        }

        Ok(())
    }

    /// Aplica um **patch** (edição incremental, NÃO destrutiva) ao grafo.
    ///
    /// É transacional: primeiro valida TODO o patch (dry-run resolvendo ids e
    /// portos, verificando conflitos) sobre um clone do estado; só se tudo for
    /// válido é que empurra o histórico (permite Ctrl+Z) e aplica de fato. Se
    /// qualquer comando for inválido, nada é alterado e o erro é retornado.
    ///
    /// Veja `src/dsl/patch_dsl.rs` para a gramática.
    #[allow(dead_code)]
    pub fn aplicar_patch(
        &mut self,
        codigo: &str,
    ) -> Result<(), crate::dsl::project_dsl::ScriptError> {
        use crate::dsl::patch_dsl::{parse_patch, PatchCmd};
        use crate::dsl::project_dsl::{indice_porto, tipo_da_dsl, ScriptError};

        let cmds = parse_patch(codigo)?;

        // ---- 1) DRY-RUN: valida tudo sobre um clone dos ids (nada muta). ----
        // Simula quais ids passarão a existir/deixar de existir para validar
        // referências futuras dentro do mesmo patch (ex.: add p1; connect p1..).
        let mut ids_sim: std::collections::HashSet<String> =
            self.dsl_ids.keys().cloned().collect();
        for cmd in &cmds {
            match cmd {
                PatchCmd::Add { tipo, id, .. } => {
                    if tipo_da_dsl(tipo).is_none() {
                        return Err(ScriptError::Apply(format!("tipo '{tipo}' desconhecido")));
                    }
                    if ids_sim.contains(id) {
                        return Err(ScriptError::Apply(format!(
                            "id '{id}' já existe (use 'set' para editar)"
                        )));
                    }
                    ids_sim.insert(id.clone());
                }
                PatchCmd::Set { id, .. } => {
                    if !ids_sim.contains(id) {
                        return Err(ScriptError::Apply(format!("nó '{id}' não existe")));
                    }
                }
                PatchCmd::Remove { id } => {
                    if !ids_sim.contains(id) {
                        return Err(ScriptError::Apply(format!("nó '{id}' não existe")));
                    }
                    if id == "canvas" || id == "master" {
                        return Err(ScriptError::Apply(format!(
                            "nó fixo '{id}' não pode ser removido"
                        )));
                    }
                    ids_sim.remove(id);
                }
                PatchCmd::Connect(c) | PatchCmd::Disconnect(c) => {
                    if !ids_sim.contains(&c.de) {
                        return Err(ScriptError::Apply(format!("nó '{}' não existe", c.de)));
                    }
                    if !ids_sim.contains(&c.para) {
                        return Err(ScriptError::Apply(format!("nó '{}' não existe", c.para)));
                    }
                }
            }
        }

        // ---- 2) COMMIT: ponto único de undo para o patch inteiro. ----
        self.empurrar_historico();

        for cmd in cmds {
            match cmd {
                PatchCmd::Add { tipo, id, campos, codigo } => {
                    let t = tipo_da_dsl(&tipo).unwrap();
                    let idx = self.adicionar_no_em(t, self.proxima_pos_livre());
                    self.dsl_ids.insert(id.clone(), idx);
                    let ndef = crate::dsl::project_dsl::NodeDef {
                        tipo,
                        id,
                        campos,
                        codigo,
                    };
                    aplicar_campos(self, idx, &ndef)?;
                }
                PatchCmd::Set { id, campo, valor, codigo } => {
                    let idx = *self.dsl_ids.get(&id).unwrap();
                    let ndef = crate::dsl::project_dsl::NodeDef {
                        tipo: String::new(),
                        id: id.clone(),
                        campos: if codigo.is_some() {
                            Vec::new()
                        } else {
                            vec![(campo, valor)]
                        },
                        codigo,
                    };
                    aplicar_campos(self, idx, &ndef)?;
                }
                PatchCmd::Remove { id } => {
                    if let Some(idx) = self.dsl_ids.remove(&id) {
                        self.g.remove_node(idx);
                        self.params.remove(&idx);
                        self.liberados.remove(&idx);
                    }
                    self.limpar_grupos();
                }
                PatchCmd::Connect(c) => {
                    let (src, dst, saida_i, entrada_i) = self.resolver_conexao(&c)?;
                    self.conectar_parametro(src, saida_i, None, dst, entrada_i, None);
                }
                PatchCmd::Disconnect(c) => {
                    let (src, dst, saida_i, entrada_i) = self.resolver_conexao(&c)?;
                    self.remover_aresta_entre(src, saida_i, dst, entrada_i);
                }
            }
        }
        let _ = indice_porto; // (usado dentro de resolver_conexao)
        Ok(())
    }

    /// Resolve os `NodeIndex` e índices de porto de uma conexão do patch.
    #[allow(dead_code)]
    fn resolver_conexao(
        &self,
        c: &crate::dsl::patch_dsl::Conexao,
    ) -> Result<(NodeIndex, NodeIndex, usize, usize), crate::dsl::project_dsl::ScriptError> {
        use crate::dsl::project_dsl::{indice_porto, ScriptError};
        let src = *self
            .dsl_ids
            .get(&c.de)
            .ok_or_else(|| ScriptError::Apply(format!("nó '{}' não existe", c.de)))?;
        let dst = *self
            .dsl_ids
            .get(&c.para)
            .ok_or_else(|| ScriptError::Apply(format!("nó '{}' não existe", c.para)))?;
        let src_tipo = self.tipo_do_no(src);
        let dst_tipo = self.tipo_do_no(dst);
        let saida_i = indice_porto(src_tipo, &c.saida, true).ok_or_else(|| {
            ScriptError::Apply(format!("porta de saída '{}' inválida em '{}'", c.saida, c.de))
        })?;
        let entrada_i = indice_porto(dst_tipo, &c.entrada, false).ok_or_else(|| {
            ScriptError::Apply(format!(
                "porta de entrada '{}' inválida em '{}'",
                c.entrada, c.para
            ))
        })?;
        Ok((src, dst, saida_i, entrada_i))
    }

    /// Remove a aresta entre `src`(saída) e `dst`(entrada), se existir.
    #[allow(dead_code)]
    fn remover_aresta_entre(&mut self, src: NodeIndex, saida: usize, dst: NodeIndex, entrada: usize) {
        let alvo = self.g.edges_iter().find_map(|(ei, _)| {
            let (s, d) = self.g.edge_endpoints(ei)?;
            if s != src || d != dst {
                return None;
            }
            let info = self.g.edge(ei).map(|e| *e.payload())?;
            if info.saida == saida && info.entrada == entrada {
                Some(ei)
            } else {
                None
            }
        });
        if let Some(ei) = alvo {
            self.g.remove_edge(ei);
        }
    }

    /// Uma posição de canvas livre para colocar um novo nó vindo do patch,
    /// abaixo dos nós existentes (evita empilhar tudo na origem).
    #[allow(dead_code)]
    fn proxima_pos_livre(&self) -> Pos2 {
        let mut max_y = 0.0f32;
        for (_, n) in self.g.nodes_iter() {
            max_y = max_y.max(n.location().y);
        }
        Pos2::new(0.0, max_y + 160.0)
    }

    /// Recupera o `TipoNo` de um nó pelo seu rótulo no grafo.
    fn tipo_do_no(&self, idx: NodeIndex) -> TipoNo {
        if let Some(n) = self.g.node(idx) {
            TipoNo::from_label(&n.label()).unwrap_or(TipoNo::Saida)
        } else {
            TipoNo::Saida
        }
    }

    /// Porta de entrada `porta` em coordenadas de canvas.
    fn porta_entrada_canvas(&self, idx: NodeIndex, porta: usize) -> Option<Pos2> {
        let n = self.g.node(idx)?;
        let (ins, _) = self.portos_offsets(idx)?;
        let off = ins
            .get(porta)
            .copied()
            .unwrap_or_else(|| Vec2::new(-node_display::NoDisplay::tamanho(n.label().as_str()).x, 0.0));
        Some(n.location() + off)
    }

    /// Cria uma aresta de parâmetro entre a saída `saida` (componente
    /// `saida_comp`) de `src` e a entrada `entrada` (componente
    /// `entrada_comp`) de `dst`.
    fn conectar_parametro(
        &mut self,
        src: NodeIndex,
        saida: usize,
        saida_comp: Option<usize>,
        dst: NodeIndex,
        entrada: usize,
        entrada_comp: Option<usize>,
    ) {
        self.g.add_edge(
            src,
            dst,
            node_display::ArestaInfo {
                saida,
                saida_comp,
                entrada,
                entrada_comp,
            },
        );
    }

    /// Porto de entrada mais próximo de `p` (canvas), até `max`.
    /// Retorna (nó, índice do porto).
    fn porta_entrada_mais_proxima(&self, p: Pos2, max: f32) -> Option<(NodeIndex, usize)> {
        let mut melhor: Option<(NodeIndex, usize, f32)> = None;
        for (idx, n) in self.g.nodes_iter() {
            let Some((ins, _)) = self.portos_offsets(idx) else {
                continue;
            };
            let loc = n.location();
            for (i, off) in ins.iter().enumerate() {
                let d = (loc + *off).distance(p);
                if d <= max && melhor.map_or(true, |(_, _, md)| d < md) {
                    melhor = Some((idx, i, d));
                }
            }
        }
        melhor.map(|(i, p, _)| (i, p))
    }

    fn aresta_sob_cursor(&self, p_screen: Pos2, frame: &MetadataFrame, rect: Rect) -> Option<EdgeIndex> {
        let mut melhor: Option<(EdgeIndex, f32)> = None;
        for (idx, _e) in self.g.edges_iter() {
            if let Some((s, d)) = self.g.edge_endpoints(idx) {
                if let (Some(ns), Some(nd)) = (self.g.node(s), self.g.node(d)) {
                    let w = self.g.edge(idx).map(|e| *e.payload());
                    let (saida, entrada) = w.map(|a| (a.saida, a.entrada)).unwrap_or((0, 0));
                    let p0c = ns.display().port_out_pos(saida);
                    let p3c = nd.display().port_in_pos(entrada);
                    let dx = ((p3c.x - p0c.x).abs() * 0.5).max(30.0);
                    let p0 = self.canvas_para_screen(p0c, frame, rect);
                    let p1 = self.canvas_para_screen(Pos2::new(p0c.x + dx, p0c.y), frame, rect);
                    let p2 = self.canvas_para_screen(Pos2::new(p3c.x - dx, p3c.y), frame, rect);
                    let p3 = self.canvas_para_screen(p3c, frame, rect);
                    for i in 0..=10 {
                        let t = i as f32 / 10.0;
                        let u = 1.0 - t;
                        let pt = Pos2::new(
                            u * u * u * p0.x + 3.0 * u * u * t * p1.x + 3.0 * u * t * t * p2.x
                                + t * t * t * p3.x,
                            u * u * u * p0.y + 3.0 * u * u * t * p1.y + 3.0 * u * t * t * p2.y
                                + t * t * t * p3.y,
                        );
                        let d = pt.distance(p_screen);
                        if melhor.map_or(true, |(_, md)| d < md) {
                            melhor = Some((idx, d));
                        }
                    }
                }
            }
        }
        // raio generoso: qualquer parte do cursor (ponta ou cabo da faca)
        // que encoste no fio já vira faca.
        melhor.filter(|(_, d)| *d <= 12.0).map(|(i, _)| i)
    }

    /// Pontos de tela (screen) da aresta, para desenhar o realce de hover.
    fn aresta_pontos_screen(
        &self,
        ei: EdgeIndex,
        frame: &MetadataFrame,
        rect: Rect,
    ) -> Option<(Pos2, Pos2, Pos2, Pos2)> {
        let (s, d) = self.g.edge_endpoints(ei)?;
        let ns = self.g.node(s)?;
        let nd = self.g.node(d)?;
        let w = self.g.edge(ei).map(|e| *e.payload());
        let (saida, entrada) = w.map(|a| (a.saida, a.entrada)).unwrap_or((0, 0));
        let p0c = ns.display().port_out_pos(saida);
        let p3c = nd.display().port_in_pos(entrada);
        let dx = ((p3c.x - p0c.x).abs() * 0.5).max(30.0);
        let p0 = self.canvas_para_screen(p0c, frame, rect);
        let p1 = self.canvas_para_screen(Pos2::new(p0c.x + dx, p0c.y), frame, rect);
        let p2 = self.canvas_para_screen(Pos2::new(p3c.x - dx, p3c.y), frame, rect);
        let p3 = self.canvas_para_screen(p3c, frame, rect);
        Some((p0, p1, p2, p3))
    }

    fn desenhar_grade(&self, painter: &Painter, rect: Rect) {
        let step = 26.0;
        let traco = Stroke::new(1.0, Color32::from_gray(46));
        let r = 3.0;
        let mut y = rect.top() + step;
        while y < rect.bottom() {
            let mut x = rect.left() + step;
            while x < rect.right() {
                painter.line_segment([Pos2::new(x - r, y), Pos2::new(x + r, y)], traco);
                painter.line_segment([Pos2::new(x, y - r), Pos2::new(x, y + r)], traco);
                x += step;
            }
            y += step;
        }
    }

    /// Centraliza e dá zoom de foco (>=1:1) em um nó.
    fn focar_no(&mut self, ui: &mut Ui, rect: Rect, idx: NodeIndex) {
        let loc = match self.g.node(idx) {
            Some(n) => n.location().to_vec2(),
            None => return,
        };
        let mut frame = MetadataFrame::new(self.id_grafo.clone()).load(ui);
        frame.zoom = frame.zoom.max(1.0).clamp(ZOOM_MIN, ZOOM_MAX);
        // o desenho soma `rect.left_top()` ao pan, então subtraímos
        // para centralizar o nó exatamente no centro do painel
        frame.pan = rect.center().to_vec2() - loc * frame.zoom - rect.left_top().to_vec2();
        frame.save(ui);
        ui.ctx().request_repaint();
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let margin = 2.0;

        let size = Vec2::new(ui.available_width(), ui.available_height() - margin);

        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());

        // fundo do canvas
        ui.painter().rect_filled(rect, 8.0, Color32::from_rgb(22, 22, 30));

        // grade de fundo (visual de "+")
        self.desenhar_grade(&ui.painter(), rect);

        // toolbar flutuante (captura cliques e define a ação)
        self.toolbar.show(ui, rect, self.pode_undo(), self.pode_redo());

        // processa a ação solicitada pela toolbar
        if let Some(acao) = self.toolbar.acao.take() {
            match acao {
                AcaoToolbar::Adicionar(t) => {
                    self.empurrar_historico();
                    self.adicionar_no(t);
                }
                AcaoToolbar::ModeloAnimTexto(id) => {
                    self.empurrar_historico();
                    self.aplicar_modelo_anim_texto(id);
                }
                AcaoToolbar::Undo => {
                    self.undo();
                }
                AcaoToolbar::Redo => {
                    self.redo();
                }
            }
        }

        // busca por nome (Enter no campo de busca)
        if self.toolbar.focus_search {
            self.toolbar.focus_search = false;
            let q = self.toolbar.search_query.clone();
            self.buscar(&q);
        }

        // ---- PAN / ZOOM igual ao canvas (touchpad 2 dedos e Alt) ----
        let alt = ui.ctx().input(|i| i.modifiers.alt);
        let mut gesture = Vec2::ZERO;

        // Botão do meio do mouse (arrastar) — lido direto do ponteiro para
        // funcionar mesmo com o cursor SOBRE um nó (a `Area` do conteúdo não
        // deve bloquear o pan). Inicia se pressionado dentro do painel e
        // continua até soltar.
        let (mid_press, mid_down, mid_delta) = ui.ctx().input(|i| {
            (
                i.pointer.button_pressed(PointerButton::Middle),
                i.pointer.button_down(PointerButton::Middle),
                i.pointer.delta(),
            )
        });
        if mid_press && ui.rect_contains_pointer(rect) {
            self.pan_meio = true;
        }
        if !mid_down {
            self.pan_meio = false;
        }
        if self.pan_meio {
            gesture += mid_delta;
        }

        // Cria o `child_ui` (viewport onde o grafo é efetivamente desenhado)
        // logo no início, para usarmos seu retângulo nas conversões de
        // coordenadas dos hit-tests (o egui_graphs posiciona nós relativo a ele).
        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect));
        child_ui.set_clip_rect(rect);
        let view_rect = child_ui.min_rect();

        // Posições do cursor (tela e canvas) usadas tanto pelo feedback de
        // cursor quanto pela edição de conexões abaixo. Definidas aqui para
        // estarem disponíveis em todo o resto da função.
        let frame = MetadataFrame::new(self.id_grafo.clone()).load(ui);
        let p_screen = ui.ctx().pointer_interact_pos();
        let p_canvas = p_screen.map(|p| self.screen_para_canvas(p, &frame, view_rect));

        let frame_tmp = MetadataFrame::new(self.id_grafo.clone()).load(ui);
        // Decide se o scroll/gesto do ponteiro deve mover o canvas. Se o
        // ponteiro está sobre um campo editável do nó, sobre o corpo de um nó,
        // ou o usuário está numa interação (arrastar nó/grupo/conexão, caixa de
        // seleção, ou com o botão primário pressionado), NÃO fazemos pan/zoom —
        // senão o canvas "foge" e o clique/arraste do nó não pega o alvo.
        let bloqueia_gesto = if ui.rect_contains_pointer(rect) {
            let sobre_campo = ui
                .ctx()
                .pointer_interact_pos()
                .map_or(false, |ps| self.sobre_conteudo(Some(ps), &frame_tmp, view_rect));
            let p_canvas_tmp = ui
                .ctx()
                .pointer_interact_pos()
                .map(|p| self.screen_para_canvas(p, &frame_tmp, view_rect));
            let sobre_no = p_canvas_tmp.map_or(false, |pc| self.node_sob_cursor(pc).is_some());
            let arrastando_algo =
                self.arrastando.is_some() || self.arrastando_grupo.is_some() || self.conexao.is_some();
            let prim_down = ui.ctx().input(|i| i.pointer.button_down(PointerButton::Primary));
            let em_interacao = arrastando_algo || self.selecao.is_some() || prim_down;
            sobre_campo || sobre_no || em_interacao
        } else {
            false
        };
        if ui.rect_contains_pointer(rect) && !bloqueia_gesto {
            // scroll do mouse / 2 dedos no trackpad
            let scroll = scroll_delta(ui.ctx());
            if alt {
                gesture.y += scroll.y; // Alt + scroll → usa Y para zoom
            } else {
                gesture += scroll; // scroll normal → pan
            }
        }

        if !bloqueia_gesto {
            if alt {
                // Alt + gesto = zoom suave com foco no cursor
                let mut frame = MetadataFrame::new(self.id_grafo.clone()).load(ui);
                let a_global = ui
                    .ctx()
                    .pointer_interact_pos()
                    .unwrap_or_else(|| rect.center())
                    .to_vec2();
                // o pan é em espaço do painel; converte o cursor p/ o mesmo espaço
                let a = a_global - rect.left_top().to_vec2();
                // passo por frame limitado p/ o zoom não "pular"
                let fator = (-gesture.y * 0.005).exp().clamp(0.9, 1.0 / 0.9);
                let novo_zoom = (frame.zoom * fator).clamp(ZOOM_MIN, ZOOM_MAX);
                let c = (a - frame.pan) / frame.zoom;
                frame.pan = a - c * novo_zoom;
                frame.zoom = novo_zoom;
                frame.save(ui);
                ui.ctx().request_repaint();
            } else if gesture != Vec2::ZERO {
                let mut frame = MetadataFrame::new(self.id_grafo.clone()).load(ui);
                frame.pan += gesture;
                frame.save(ui);
                ui.ctx().request_repaint();
            }
        }

        if self.pan_meio || self.arrastando_grupo.is_some() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        } else {
            let frame = MetadataFrame::new(self.id_grafo.clone()).load(ui);
            if response.hovered()
                && !ui.ctx().egui_is_using_pointer()
                && self.sobre_conteudo(p_screen, &frame, rect)
            {
                // sobre um nó (corpo editável): cursor padrão de texto/seleção
                ui.ctx().set_cursor_icon(CursorIcon::Default);
            } else if response.hovered() && !ui.ctx().egui_is_using_pointer() {
                // área vazia: mão aberta só enquanto o botão está pressiono (pan);
                // em repouso fica o cursor padrão para não parecer "arrastar tudo".
                let baixo = ui.ctx().input(|i| {
                    i.pointer.button_down(eframe::egui::PointerButton::Primary)
                        || i.pointer.button_down(eframe::egui::PointerButton::Middle)
                });
                // Pan de canvas (área vazia) mostra "segurando" só enquanto o
                // botão está pressiono. Sobre um NÓ não mostramos cursor de pan:
                // o arraste do nó já é tratado em outro ponto, e queremos evitar
                // que o cursor de arrastar apareça em partes do nó que não sejam
                // o nome (cabeçalho) ou o botão de cor.
                if baixo {
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                } else {
                    ui.ctx().set_cursor_icon(CursorIcon::Default);
                }
            }
        }

        // surfaces dos grupos: desenhados ATRÁS dos nós
        let frame_g = MetadataFrame::new(self.id_grafo.clone()).load(ui);
        self.limpar_grupos();
        self.desenhar_grupos_fundo(ui, rect, &frame_g);

        // conteúdo do grafo (child_ui já foi criado no início do `show`,
        // compartilhando o mesmo `view_rect` usado nos hit-tests)
        let mut view = GraphView::<
            (),
            node_display::ArestaInfo,
            Directed,
            petgraph::stable_graph::DefaultIx,
            node_display::NoDisplay,
            node_display::ArestaCurva,
        >::new(&mut self.g)
        .with_id(self.id_grafo.clone())
        .with_navigations(
            &SettingsNavigation::new()
                .with_fit_to_screen_enabled(false)
                .with_zoom_and_pan_enabled(false),
        )
        // O GraphView NÃO gerencia interação: a seleção/arraste é feita
        .with_interactions(
            &SettingsInteraction::new()
                .with_node_selection_enabled(false)
                .with_edge_selection_enabled(false)
                .with_dragging_enabled(false),
        )
        .with_styles(&SettingsStyle::new().with_labels_always(true));

        child_ui.add(&mut view);

        // garante o mestre e mantém as posições iniciais dos 3 nós
        // (o layout aleatório poderia realocá-los na primeira frame)
        self.garantir_master();
        self.reafirmar_posicoes();

        // ---- TECLA F: foca (centraliza + zoom de foco) no nó selecionado ----
        if ui.ctx().input(|i| i.key_pressed(Key::F)) {
            // foca no primeiro nó selecionado
            let alvo = self.g.nodes_iter().find(|(_, n)| n.selected()).map(|(idx, _)| idx);
            if let Some(idx) = alvo {
                self.focar_no(ui, rect, idx);
            }
        }

        // ---- CTRL+G: agrupa os nós selecionados ----
        if ui.ctx().input(|i| i.modifiers.ctrl && i.key_pressed(Key::G)) {
            self.agrupar_selecionados();
        }

        // ---- EDIÇÃO DE CONEXÕES (conectar / mover / tesoura) ----
        // `frame`, `p_screen`, `p_canvas` já foram definidos acima (após
        // `view_rect`) e são reutilizados aqui.
        let down = ui.ctx().input(|i| i.pointer.button_down(PointerButton::Primary));
        let click = ui.ctx().input(|i| i.pointer.button_pressed(PointerButton::Primary));
        let released = ui.ctx().input(|i| i.pointer.button_released(PointerButton::Primary));

        // guarda a posição do botão direito (para o menu de contexto / colar)
        if ui.ctx().input(|i| i.pointer.button_pressed(PointerButton::Secondary)) {
            if let Some(pc) = p_canvas {
                self.menu_canvas = pc;
            }
        }

        // feedback: aresta sob o cursor
        self.aresta_hover = p_screen.and_then(|ps| self.aresta_sob_cursor(ps, &frame, view_rect));

        // o ponteiro está sobre um widget interativo (seletor de cor,
        // feedback: cursor de mover ao passar no cabeçalho de um grupo
        // ou de um nó individual (o cabeçalho arrasta o nó).
        if self.arrastando_grupo.is_none() {
            let sobre_header = p_screen
                .and_then(|ps| self.grupo_header_sob(ps, &frame, view_rect))
                .is_some();
            let sobre_header_no = p_canvas
                .and_then(|pc| self.sobre_cabecalho_no(pc))
                .is_some();
            if sobre_header || sobre_header_no {
                ui.ctx().set_cursor_icon(CursorIcon::Move);
            }
        }

        // PRESS: decide o que iniciar
        if click {
            // menu de componentes aberto: não inicia interação de canvas
            // (o popup consome o clique para escolher X/Y/Z).
            if self.menu_componentes.is_some() {
                // ignora
            } else if let Some(pc) = p_canvas {
                if let Some((src, out)) = self.porta_saida_mais_proxima(pc) {
                    self.conexao = Some((src, out, pc));
                } else if let Some(ei) = self
                    .aresta_hover
                    .filter(|_| p_screen.is_some())
                    .or_else(|| {
                        p_screen.and_then(|ps| self.aresta_sob_cursor(ps, &frame, view_rect))
                    })
                {
                    // Cursor faca sobre o fio: corta JÁ no press (mesmo sem
                    // soltar), para não ser "roubado" pelo arraste de nó/canvas
                    // que o egui dispara em seguida.
                    self.g.remove_edge(ei);
                } else if let Some(ni) = self.node_sob_cursor(pc) {
                    // Corpo do nó: inicia arraste do nó e o seleciona
                    // (shift = adicionar à seleção). Não usamos a guarda
                    // `!usando_widget` aqui: o egui_graphs tem interações
                    // desativadas, e queremos selecionar/arrastar o nó mesmo
                    // quando o ponteiro está sobre seu conteúdo.
                    let loc = self.g.node(ni).unwrap().location();
                    self.empurrar_historico();
                    self.arrastando = Some((ni, pc - loc));
                    self.selecionar_no(ni, ui.ctx().input(|i| i.modifiers.shift));
                } else if let Some(gi) = p_screen
                    .and_then(|ps| self.grupo_header_sob(ps, &frame, rect))
                {
                    // arrastar pelo cabeçalho move todos os nós do grupo
                    self.empurrar_historico();
                    self.arrastando_grupo = Some((gi, pc));
                } else if ui.rect_contains_pointer(rect) {
                    // clique em área vazia: inicia caixa de seleção. Não usamos
                    // a guarda `!usando_widget` aqui: o egui_graphs mantém as
                    // interações de nó desativadas e o pan é tratado à parte,
                    // então queremos que a caixa inicie mesmo sobre a área do
                    // grafo.
                    self.selecao = Some((pc, pc));
                }
            }
        }

        // DRAG: atualiza conexão em andamento ou move nó
        if down {
            if let Some((_, _, cur)) = &mut self.conexao {
                if let Some(pc) = p_canvas {
                    *cur = pc;
                }
            }
            if let Some((ni, off)) = &self.arrastando {
                if let Some(pc) = p_canvas {
                    if let Some(n) = self.g.node_mut(*ni) {
                        n.set_location(pc - *off);
                    }
                }
            }
            if let Some((_, cur)) = &mut self.selecao {
                if let Some(pc) = p_canvas {
                    *cur = pc;
                }
            }
            if let Some((gi, prev)) = self.arrastando_grupo {
                if let Some(pc) = p_canvas {
                    let delta = pc - prev;
                    self.arrastando_grupo = Some((gi, pc));
                    let nos = self.grupos[gi].nos.clone();
                    for idx in nos {
                        let novo = self.g.node(idx).map(|n| n.location() + delta);
                        if let Some(loc) = novo {
                            if let Some(n) = self.g.node_mut(idx) {
                                n.set_location(loc);
                            }
                            self.liberados.insert(idx);
                        }
                    }
                }
            }
        }

        // RELEASE: conclui conexão / remove aresta
        if released {
            if let Some((src, src_out, pc)) = self.conexao.take() {
                // alvo: a porta de entrada mais próxima do cursor, se compatível
                let alvo = self.porta_entrada_mais_proxima(pc, 26.0);
                // porto de saída vetorial? ao soltar abre o menu de componentes
                // (X/Y/Z) — funciona tanto soltando sobre uma entrada compatível
                // quanto no vazio.
                let is_vetor = self
                    .tipo_do_node(src)
                    .and_then(|t| porto_saida(t, src_out))
                    .map_or(false, |p| p.is_vetor());
                if is_vetor {
                    self.menu_componentes = Some(MenuComponentes {
                        src,
                        saida: src_out,
                        drop_screen: p_screen.unwrap_or_default(),
                        drop_canvas: pc,
                        alvo,
                        escolha: None,
                        rect: None,
                    });
                } else if let Some((dst, in_port)) = alvo {
                    if dst != src {
                        let ok = match (self.tipo_do_node(src), self.tipo_do_node(dst)) {
                            (Some(o), Some(d)) => TipoNo::pode_conectar(o, d),
                            _ => false,
                        };
                        if ok {
                            self.conectar_parametro(src, src_out, None, dst, in_port, None);
                        }
                    }
                }
            }
            // nó solto: não reafirma mais a posição inicial
            if let Some((ni, _)) = self.arrastando.take() {
                self.liberados.insert(ni);
            }
            self.arrastando_grupo = None;
            // conclui a caixa de seleção: seleciona nós cujo retângulo
            // (centro ± meio-tamanho) intercepta a caixa — mais robusto que
            // exigir que o centro esteja contido.
            if let Some((a, b)) = self.selecao.take() {
                let sel_rect = Rect::from_two_pos(a, b);
                let caixa_util = sel_rect.width() > 3.0 || sel_rect.height() > 3.0;
                if caixa_util {
                    let alvos: Vec<(NodeIndex, bool)> = self
                        .g
                        .nodes_iter()
                        .map(|(idx, n)| {
                            let half = node_display::NoDisplay::tamanho(&n.label());
                            let node_rect =
                                Rect::from_center_size(n.location(), half * 2.0);
                            (idx, sel_rect.intersects(node_rect))
                        })
                        .collect();
                    for (idx, sel) in alvos {
                        if let Some(no) = self.g.node_mut(idx) {
                            no.set_selected(sel);
                        }
                    }
                } else {
                    // clique em área vazia (sem arrasto): limpa a seleção
                    let indices: Vec<_> = self.g.nodes_iter().map(|(i, _)| i).collect();
                    for idx in indices {
                        if let Some(no) = self.g.node_mut(idx) {
                            no.set_selected(false);
                        }
                    }
                }
            }
        }

        // Tecla Delete/Backspace remove arestas selecionadas
        if ui.ctx().input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            let tem_selecionadas = self
                .g
                .edges_iter()
                .any(|(idx, _)| self.g.edge(idx).map_or(false, |e| e.selected()));
            if tem_selecionadas {
                self.empurrar_historico();
            }
            let selecionadas: Vec<EdgeIndex> = self
                .g
                .edges_iter()
                .filter(|(idx, _)| self.g.edge(*idx).map_or(false, |e| e.selected()))
                .map(|(idx, _)| idx)
                .collect();
            for ei in selecionadas {
                self.g.remove_edge(ei);
            }
        }

        // ---- MENU DE CONTEXTO (botão direito / 2 dedos) ----
        // Aberto a partir do clique direito BRUTO (não da resposta do painel),
        // para funcionar mesmo com o cursor sobre nós/surfaces (camadas Area).
        let sel_count = self.selecionados().len();
        let tem_clip = !self.clipboard.is_empty();
        let mut acao_menu: Option<selection::AcaoMenu> = None;
        let abrir_menu = ui.ctx().input(|i| i.pointer.secondary_clicked())
            && p_screen.map_or(false, |p| rect.contains(p));
        Popup::menu(&response)
            .open_memory(if abrir_menu {
                Some(eframe::egui::SetOpenCommand::Bool(true))
            } else {
                None
            })
            .at_pointer_fixed()
            .show(|ui| {
                ui.set_min_width(120.0);
                if ui.add_enabled(sel_count >= 1, Button::new("Copiar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Copiar);
                    ui.close();
                }
                if ui.add_enabled(tem_clip, Button::new("Colar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Colar);
                    ui.close();
                }
                if ui.add_enabled(sel_count >= 1, Button::new("Duplicar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Duplicar);
                    ui.close();
                }
                if ui.add_enabled(sel_count >= 1, Button::new("Deletar")).clicked() {
                    acao_menu = Some(selection::AcaoMenu::Deletar);
                    ui.close();
                }
                if sel_count >= 1 {
                    ui.separator();
                    if ui.button("Agrupar").clicked() {
                        acao_menu = Some(selection::AcaoMenu::Agrupar);
                        ui.close();
                    }
                }
                // Submenu: adicionar nó na posição do cursor (botão direito)
                ui.separator();
                ui.menu_button("Adicionar nó", |ui| {
                    let tipos: [(TipoNo, &str, Color32); 9] = [
                        (TipoNo::Saida, "Master", Color32::from_rgb(120, 220, 140)),
                        (TipoNo::Canvas, "Canvas", Color32::from_rgb(170, 120, 235)),
                        (TipoNo::Cena, "Cena", Color32::from_rgb(90, 190, 190)),
                        (TipoNo::Shape, "Shape", Color32::from_rgb(235, 150, 120)),
                        (TipoNo::Texto, "Texto", Color32::from_rgb(150, 200, 120)),
                        (TipoNo::Pen, "Pen", Color32::from_rgb(200, 120, 220)),
                        (TipoNo::Ruido, "Ruído", Color32::from_rgb(120, 200, 220)),
                        (TipoNo::Anim, "Animação", Color32::from_rgb(230, 130, 170)),
                        (TipoNo::Layer, "Layers", Color32::from_rgb(120, 170, 235)),
                    ];
                    for (t, nome, cor) in tipos {
                        if ui
                            .button(egui::RichText::new(nome).color(cor))
                            .clicked()
                        {
                            // adiciona na posição do clique (canvas) e seleciona
                            let p = self.menu_canvas;
                            let idx = self.adicionar_no_em(t.instancia(), p);
                            self.selecionar_no(idx, false);
                            ui.close();
                        }
                    }
                });
            });
        match acao_menu {
            Some(selection::AcaoMenu::Copiar) => self.copiar_selecionados(),
            Some(selection::AcaoMenu::Colar) => {
                self.empurrar_historico();
                let p = self.menu_canvas;
                self.colar_em(p);
            }
            Some(selection::AcaoMenu::Duplicar) => {
                self.empurrar_historico();
                self.duplicar_selecionados();
            }
            Some(selection::AcaoMenu::Deletar) => {
                self.empurrar_historico();
                self.deletar_selecionados();
            }
            Some(selection::AcaoMenu::Agrupar) => self.agrupar_selecionados(),
            None => {}
        }

        // ---- MENU DE COMPONENTE (X/Y/Z) ao soltar fio de porto vetorial ----
        // Desenha um popup listando os componentes da "linha" de parâmetro de
        // origem (ex.: Posição -> X / Y / Z). O usuário escolhe qual componente
        // é enviado à entrada compatível mais próxima (ou à já capturada no
        // soltar, funcione soltando sobre entrada ou no vazio).
        if let Some(mut menu) = self.menu_componentes.take() {
            let comps: Vec<&'static str> = self
                .tipo_do_node(menu.src)
                .and_then(|t| porto_saida(t, menu.saida))
                .map(|p| (0..p.n_componentes()).map(|k| p.componente(k)).collect())
                .unwrap_or_default();
            let nome_porto = self
                .tipo_do_node(menu.src)
                .and_then(|t| porto_saida(t, menu.saida))
                .map(|p| p.nome)
                .unwrap_or("");
            let label_src = self
                .g
                .node(menu.src)
                .map(|n| n.label().to_string())
                .unwrap_or_default();
            let ar = Area::new(Id::new("menu_componentes"))
                .order(Order::Foreground)
                .fixed_pos(menu.drop_screen)
                .movable(false)
                .constrain(false)
                .show(ui.ctx(), |ui| {
                    ui.set_min_width(120.0);
                    ui.vertical(|ui| {
                        ui.label(format!("{} ▸ {}", label_src, nome_porto));
                        for (k, nome) in comps.iter().enumerate() {
                            if ui.button(*nome).clicked() {
                                menu.escolha = Some(k);
                            }
                        }
                        ui.separator();
                        if ui.button("Cancelar").clicked() {
                            menu.escolha = Some(255);
                        }
                    });
                });
            menu.rect = Some(ar.response.rect);
            // processa a escolha
            match menu.escolha {
                Some(255) => {
                    // cancelou: descarta
                }
                Some(k) => {
                    // reconstrói o alvo (pode ter sido None no soltar)
                    let alvo = menu
                        .alvo
                        .or_else(|| self.porta_entrada_mais_proxima(menu.drop_canvas, 26.0));
                    if let Some((dst, in_port)) = alvo {
                        if dst != menu.src {
                            let ok = match (self.tipo_do_node(menu.src), self.tipo_do_node(dst)) {
                                (Some(o), Some(d)) => TipoNo::pode_conectar(o, d),
                                _ => false,
                            };
                            if ok {
                                self.conectar_parametro(
                                    menu.src,
                                    menu.saida,
                                    Some(k),
                                    dst,
                                    in_port,
                                    None,
                                );
                            }
                        }
                    }
                }
                None => {
                    // ainda sem escolha: fecha se clicou fora do popup
                    let click_fora = ui.ctx().input(|i| {
                        i.pointer.primary_clicked() || i.pointer.secondary_clicked()
                    }) && menu.rect.map_or(false, |r| {
                        !r.contains(ui.ctx().pointer_interact_pos().unwrap_or_default())
                    });
                    if !click_fora {
                        self.menu_componentes = Some(menu);
                    }
                }
            }
        }

        // ---- OVERLAYS (desenho acima do grafo) ----
        // painter recortado ao painel: o anel/cadeado do mestre e o
        // cabo de conexão não "vazam" para cima do preview
        let painter = ui.painter().with_clip_rect(rect);
        // Realce da aresta sob o cursor (tesoura)
        if let Some(ei) = self.aresta_hover {
            if let Some((p0, p1, p2, p3)) = self.aresta_pontos_screen(ei, &frame, rect) {
                painter.add(CubicBezierShape::from_points_stroke(
                    [p0, p1, p2, p3],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(3.0, Color32::from_rgb(235, 90, 90)),
                ));
            }
        }

        // Caixa de seleção (retângulo tracejado enquanto arrasta)
        if let Some((a, b)) = self.selecao {
            let pa = self.canvas_para_screen(a, &frame, rect);
            let pb = self.canvas_para_screen(b, &frame, rect);
            let r = Rect::from_two_pos(pa, pb);
            painter.add(RectShape::new(
                r,
                CornerRadius::same(0),
                Color32::from_rgba_unmultiplied(90, 140, 255, 40),
                Stroke::new(1.0, Color32::from_rgb(120, 160, 255)),
                StrokeKind::Inside,
            ));
        }

        // Fio temporário da conexão em andamento
        let tipo_origem = if let Some((src, _, _)) = &self.conexao {
            self.tipo_do_node(*src)
        } else {
            None
        };
        if let Some((src, src_out, cur)) = &self.conexao {
            if let Some(ns) = self.g.node(*src) {
                let p0c = ns.display().port_out_pos(*src_out);
                let p0 = self.canvas_para_screen(p0c, &frame, rect);
                // snap: se perto de uma entrada compatível, o fio "gruda" nela
                let mut p3 = self.canvas_para_screen(*cur, &frame, rect);
                if let Some(to) = tipo_origem {
                    let alvo = self
                        .porta_entrada_mais_proxima(*cur, 26.0)
                        .filter(|&(d, _)| d != *src)
                        .and_then(|(d, p)| self.tipo_do_node(d).map(|t| (d, p, t)))
                        .filter(|(_, _, t)| TipoNo::pode_conectar(to, *t))
                        .and_then(|(d, p, _)| self.porta_entrada_canvas(d, p));
                    if let Some(c) = alvo {
                        p3 = self.canvas_para_screen(c, &frame, rect);
                    }
                }
                let dx = ((p3.x - p0.x).abs() * 0.5).max(30.0);
                let p1 = Pos2::new(p0.x + dx, p0.y);
                let p2 = Pos2::new(p3.x - dx, p3.y);
                painter.add(CubicBezierShape::from_points_stroke(
                    [p0, p1, p2, p3],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(2.0, Color32::from_rgb(120, 220, 140)),
                ));
            }
        }

        // Realce das entradas compatíveis enquanto arrasta uma conexão
        if let (Some((src, _, _)), Some(to)) = (&self.conexao, tipo_origem) {
            for (idx, _) in self.g.nodes_iter() {
                if idx == *src {
                    continue;
                }
                let Some(t) = self.tipo_do_node(idx) else {
                    continue;
                };
                let compat = TipoNo::pode_conectar(to, t);
                let cor = if compat { t.cor() } else { Color32::from_gray(90) };
                if let Some((ins, _)) = self.portos_offsets(idx) {
                    for off in &ins {
                        let pe = self.canvas_para_screen(
                            self.g.node(idx).unwrap().location() + *off,
                            &frame,
                            rect,
                        );
                        let raio = if compat { 9.0 } else { 6.0 };
                        painter.add(Shape::Circle(CircleShape {
                            center: pe,
                            radius: raio,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(2.5, cor),
                        }));
                        if compat {
                            painter.add(Shape::Circle(CircleShape {
                                center: pe,
                                radius: raio + 3.0,
                                fill: Color32::TRANSPARENT,
                                stroke: Stroke::new(1.0, cor.gamma_multiply(0.6)),
                            }));
                        }
                    }
                }
            }
        }

        // Indicadores de nós fora da view (na borda do painel) — clicáveis
        let margem = 12.0;
        let inner = rect.shrink(margem);
        let raio_dot = 6.0;
        let mut dots: Vec<(NodeIndex, Pos2, Color32)> = Vec::new();
        for (idx, n) in self.g.nodes_iter() {
            let p = self.canvas_para_screen(n.location(), &frame, rect);
            if !inner.contains(p) {
                let cx = p.x.clamp(inner.min.x, inner.max.x);
                let cy = p.y.clamp(inner.min.y, inner.max.y);
                let cor = self.tipo_do_node(idx).map_or(Color32::GRAY, |t| t.cor());
                dots.push((idx, Pos2::new(cx, cy), cor));
            }
        }
        // hover / clique nas bolinhas: cursor de mão e foco no nó
        let hovered_dot = dots
            .iter()
            .find(|(_, c, _)| p_screen.map_or(false, |pp| pp.distance(*c) <= raio_dot + 3.0))
            .map(|(idx, _, _)| *idx);
        let mut click_target: Option<NodeIndex> = None;
        if let Some(idx) = hovered_dot {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            if click {
                click_target = Some(idx);
            }
        }
        for (idx, c, cor) in &dots {
            let hov = hovered_dot == Some(*idx);
            let r = if hov { raio_dot + 2.0 } else { raio_dot };
            painter.add(Shape::Circle(CircleShape {
                center: *c,
                radius: r,
                fill: if hov { cor.gamma_multiply(1.35) } else { *cor },
                stroke: Stroke::new(1.5, Color32::from_rgb(20, 20, 26)),
            }));
            if hov {
                painter.add(Shape::Circle(CircleShape {
                    center: *c,
                    radius: r + 3.0,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(1.0, cor.gamma_multiply(0.6)),
                }));
            }
        }
        if !dots.is_empty() {
            let txt = format!("{} nó(s) fora da view", dots.len());
            let galley = ui.painter().layout_no_wrap(
                txt,
                FontId::new(11.0, FontFamily::Proportional),
                Color32::from_rgb(220, 220, 230),
            );
            let pos = Pos2::new(
                rect.center().x - galley.size().x / 2.0,
                rect.max.y - galley.size().y - 6.0,
            );
            painter.add(TextShape::new(pos, galley, Color32::from_rgb(220, 220, 230)));
        }

        // foco no nó alvo (após terminar de usar o `painter`)
        if let Some(idx) = click_target {
            self.focar_no(ui, rect, idx);
        }

        // Conteúdo dos nós (parâmetros) desenhado em uma `Area` própria por
        // nó, posicionada em coordenadas de tela via `canvas_para_screen`
        // (mesma transformação usada para desenhar o card). A `Area` vive em
        // camada própria: fica por cima dos shapes do grafo, não é recortada
        // junto com eles e intercepta o input antes do canvas — por isso os
        // widgets (DragValue/ComboBox) funcionam e acompanham pan/zoom/arraste
        // sem defasagem. Lemos o `frame` já atualizado (após `add(&mut view)`).
        // Coleta os nós e ordena para que os "ativos" (hovered, selecionado ou
        // sendo arrastado) sejam desenhados POR ÚLTIMO — assim ficam NA FRENTE
        // e cobrem completamente os nós atrás (sem "misturar" os componentes
        // de um com o outro quando se sobrepõem).
        let arrastando_idx = self.arrastando.map(|(idx, _)| idx);
        let mut infos: Vec<(usize, TipoNo, bool)> = self
            .g
            .nodes_iter()
            .filter_map(|(idx, n)| {
                self.tipo_do_node(idx)
                    .map(|t| (idx.index(), t, n.selected() || n.hovered()))
            })
            .collect();
        infos.sort_by(|a, b| {
            let a_ativo = a.2 || Some(NodeIndex::new(a.0)) == arrastando_idx;
            let b_ativo = b.2 || Some(NodeIndex::new(b.0)) == arrastando_idx;
            // false (ao fundo) antes de true (à frente)
            a_ativo.cmp(&b_ativo)
        });
        for (i, tipo, _) in infos {
            let idx = NodeIndex::new(i);
            let loc = match self.g.node(idx) {
                Some(n) => n.location(),
                None => continue,
            };
            let half = node_display::NoDisplay::tamanho(tipo.nome()) * frame.zoom;
            let center = self.canvas_para_screen(loc, &frame, rect);
            let node_rect = Rect::from_center_size(center, half * 2.0);
            // Nó totalmente fora do painel: não desenha o conteúdo
            // (o indicador de "fora da view" já avisa sua posição).
            if !rect.intersects(node_rect) {
                continue;
            }
            // corpo abaixo do cabeçalho; margens/cabeçalho em canvas escalados
            // pelo zoom, para o card escalar junto (estilo Blender).
            let body_min = Pos2::new(
                node_rect.min.x + node_component::MARGEM_X * frame.zoom,
                node_rect.min.y
                    + (node_component::CABECALHO_H + node_component::MARGEM_Y) * frame.zoom,
            );
            // recorta o conteúdo ao próprio card (interseção com o painel),
            // para o conteúdo de um nó nunca vazar para cima de outro.
            let clip_no = node_rect.intersect(rect);
            let cenas = self.cenas_disponiveis();
            let params = self.params.get_mut(&idx);
            // conteúdo em tamanho natural (sem largura forçada): a `Area`
            // se ajusta ao conteúdo e nós medimos o resultado para tornar o
            // card responsivo no próximo frame.
            let resp = Area::new(Id::new(("no_conteudo", i)))
                .order(Order::Middle)
                .fixed_pos(body_min)
                .movable(false)
                .constrain(false)
                .show(ui.ctx(), |ui| {
                    ui.set_clip_rect(clip_no);
                    node_component::escalar_estilo(ui, frame.zoom);
                    ui.push_id(i, |ui| {
                        node_component::show_content(
                            ui, tipo, params, &cenas, body_min.y, frame.zoom,
                        );
                    });
                });
            node_component::registrar_medida(tipo, resp.response.rect.size(), frame.zoom);
        }

        // cabeçalho interativo dos grupos (título + seletor de cor)
        self.desenhar_grupos_header(ui, rect, &frame);

        // ---- CURSOR FACA: quando o ponteiro está sobre um fio, o cursor do
        // sistema some e o faca.svg aparece; um clique (botão esquerdo) corta
        // a aresta na hora (corte feito no press). ----
        if self.aresta_hover.is_some() {
            ui.ctx().set_cursor_icon(CursorIcon::None);
            if let Some(pos) = p_screen {
                let sz = Vec2::splat(22.0);
                // a ponta da lâmina no SVG (viewBox 0 0 13 18) fica em
                // (2.92285, 0.5625) -> topo-esquerdo. Ancora essa ponta
                // exatamente no cursor, para cortar onde o ponteiro aponta.
                let ponta = Vec2::new(2.92285 / 13.0, 0.5625 / 18.0);
                let r = Rect::from_min_size(pos - ponta * sz, sz);
                egui::Image::new(egui::include_image!("../ui/icons/faca.svg")).paint_at(ui, r);
            }
        }

        ui.ctx().request_repaint();
    }
}

/// Aplica os campos de um `ProjectBlock` a um `ProjetoConfig`.
fn aplicar_project(proj: &mut ProjetoConfig, p: &crate::dsl::project_dsl::ProjectBlock) {
    if let Some(v) = p.largura {
        proj.largura = v as u32;
    }
    if let Some(v) = p.altura {
        proj.altura = v as u32;
    }
    if let Some(v) = p.fps {
        proj.fps = v as f32;
    }
    if let Some(v) = p.duracao {
        proj.duracao_seg = v;
    }
    if let Some(c) = p.fundo {
        proj.fundo = c;
    }
}

/// Aplica os campos de um `NodeDef` aos `NodeParams` do nó correspondente.
fn aplicar_campos(
    panel: &mut GraphPanel,
    idx: NodeIndex,
    n: &crate::dsl::project_dsl::NodeDef,
) -> Result<(), crate::dsl::project_dsl::ScriptError> {
    use crate::dsl::project_dsl::Expr;
    let params = match panel.params.get_mut(&idx) {
        Some(p) => p,
        None => return Ok(()),
    };
    match params {
        NodeParams::Cena {
            nome_cena,
            zoom,
            angulo,
            opacidade,
            ..
        } => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "name" => *nome_cena = v.as_str(),
                    "zoom" => *zoom = v.as_num(),
                    "angle" => *angulo = v.as_num(),
                    "opacity" => *opacidade = v.as_num(),
                    _ => {}
                }
            }
        }
        NodeParams::Layer { cena, opacidade } => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => *cena = v.as_str(),
                    "opacity" => *opacidade = v.as_num(),
                    _ => {}
                }
            }
        }
        NodeParams::Shape {
            tipo,
            px,
            py,
            largura,
            altura,
            rotacao,
            cor,
            seed,
            noise_scale,
            amp,
            veloc,
            cena,
            ..
        } => {
            let mut cena_nome: Option<String> = None;
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => cena_nome = Some(v.as_str()),
                    "type" => {
                        *tipo = match v.as_str().as_str() {
                            "rect" | "rectangle" => 0,
                            "ellipse" => 1,
                            "triangle" => 2,
                            "star" => 3,
                            "losango" | "diamond" => 4,
                            "polygon" => 5,
                            "arrow" => 6,
                            _ => 0,
                        }
                    }
                    "pos" => {
                        if let Expr::Vec2(a, b) = v {
                            *px = *a;
                            *py = *b;
                        }
                    }
                    "size" => {
                        if let Expr::Vec2(a, b) = v {
                            *largura = *a;
                            *altura = *b;
                        }
                    }
                    "rotation" | "rot" => *rotacao = v.as_num(),
                    "color" | "colour" => *cor = v.as_hex(),
                    "seed" => *seed = v.as_num(),
                    "noise" => *noise_scale = v.as_num(),
                    "amp" => *amp = v.as_num(),
                    "speed" => *veloc = v.as_num(),
                    _ => {}
                }
            }
            if let Some(cn) = cena_nome {
                *cena = cn;
            }
        }
        NodeParams::Texto {
            conteudo,
            tamanho,
            negrito,
            italico,
            px,
            py,
            cor,
            cena,
            ..
        } => {
            let mut cena_nome: Option<String> = None;
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => cena_nome = Some(v.as_str()),
                    "content" => *conteudo = v.as_str(),
                    "size" => *tamanho = v.as_num(),
                    "bold" => *negrito = v.as_str() == "true" || v.as_str() == "on",
                    "italic" => *italico = v.as_str() == "true" || v.as_str() == "on",
                    "pos" => {
                        if let Expr::Vec2(a, b) = v {
                            *px = *a;
                            *py = *b;
                        }
                    }
                    "color" | "colour" => *cor = v.as_hex(),
                    _ => {}
                }
            }
            if let Some(cn) = cena_nome {
                *cena = cn;
            }
        }
        NodeParams::Pen {
            codigo,
            cor,
            cor_fill,
            pos_x,
            pos_y,
            espessura,
            preenchimento,
            seed,
            cantos,
            ordem,
            escala_x,
            escala_y,
            cena,
            ..
        } => {
            let mut cena_nome: Option<String> = None;
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => cena_nome = Some(v.as_str()),
                    "pos" => {
                        if let Expr::Vec2(a, b) = v {
                            *pos_x = *a;
                            *pos_y = *b;
                        }
                    }
                    "stroke" => *espessura = v.as_num(),
                    "fill" => *preenchimento = v.as_str() != "off" && v.as_str() != "false",
                    "color" | "colour" => {
                        let h = v.as_hex();
                        *cor = h;
                        *cor_fill = h;
                    }
                    "stroke_color" | "strokecolor" => *cor = v.as_hex(),
                    "fill_color" | "fillcolor" => *cor_fill = v.as_hex(),
                    "seed" => *seed = v.as_num(),
                    "corners" => *cantos = v.as_num(),
                    "order" => *ordem = v.as_num(),
                    "scalex" => *escala_x = v.as_num(),
                    "scaley" => *escala_y = v.as_num(),
                    _ => {}
                }
            }
            if let Some(cn) = cena_nome {
                *cena = cn;
            }
            if let Some(code) = &n.codigo {
                // Valida o PenDSL do bloco `codigo { }`: se houver erro, o
                // script inteiro falha (assim o log NÃO diz "OK" com um Pen
                // quebrado). O erro inclui o id do nó pen para contexto.
                if let Err(e) = crate::dsl::Program::parse(code) {
                    return Err(crate::dsl::project_dsl::ScriptError::Apply(format!(
                        "nó pen '{}': {}",
                        n.id, e
                    )));
                }
                *codigo = code.clone();
            }
        }
        NodeParams::Ruido {
            seed,
            freq,
            amp,
            veloc,
            alvo,
        } => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "seed" => *seed = v.as_num(),
                    "freq" | "noise" => *freq = v.as_num(),
                    "amp" => *amp = v.as_num(),
                    "speed" => *veloc = v.as_num(),
                    "target" => {
                        *alvo = match v.as_str().as_str() {
                            "pos" | "position" | "Posição" => 0,
                            "rot" | "rotation" | "Rotação" => 1,
                            "scale" | "Escala" => 2,
                            _ => 0,
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(())
}
