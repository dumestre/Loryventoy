use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

use crate::nodes::{NodeParams, LayerEntry, ProjetoConfig, TipoNo};
use crate::graph_editor::ArestaInfo;

/// Espelho serializável de um segmento de animação (`AnimSeg`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimSegJson {
    pub t_ini: f32,
    pub t_fim: f32,
    pub v_ini: [f32; 2],
    pub v_fim: [f32; 2],
    pub easing: u8,
}

/// Espelho serializável de uma entrada de layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerEntryJson {
    pub nome: String,
    pub ordem: f32,
    pub opacidade: f32,
}

/// Espelho serializável do `NodeParams` (que contém `Color32` e campos
/// não-serializáveis direto). Cada variante espelha os campos relevantes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeParamsJson {
    Transform {
        px: f32, py: f32, pz: f32,
        rx: f32, ry: f32, rz: f32,
        sx: f32, sy: f32, sz: f32,
    },
    Cena {
        nome_cena: String,
        ativa: bool,
        zoom: f32,
        angulo: f32,
        opacidade: f32,
    },
    Layer {
        cena: String,
        layers: Vec<LayerEntryJson>,
        selected: usize,
    },
    Texto {
        cena: String,
        conteudo: String,
        tamanho: f32,
        negrito: bool,
        italico: bool,
        px: f32,
        py: f32,
        cor: [u8; 4],
        #[serde(default)]
        trim_inicio: f32,
        #[serde(default)]
        trim_fim: f32,
    },
    Shape {
        cena: String,
        tipo: u8,
        px: f32, py: f32,
        largura: f32, altura: f32,
        rotacao: f32,
        cor: [u8; 4],
        seed: f32,
        noise_scale: f32,
        amp: f32,
        veloc: f32,
        #[serde(default)]
        trim_inicio: f32,
        #[serde(default)]
        trim_fim: f32,
    },
    Pen {
        cena: String,
        codigo: String,
        cor: [u8; 4],
        #[serde(default)]
        cor_fill: Option<[u8; 4]>,
        pos_x: f32,
        pos_y: f32,
        espessura: f32,
        preenchimento: bool,
        seed: f32,
        cantos: f32,
        ordem: f32,
        escala_x: f32,
        escala_y: f32,
        #[serde(default)]
        trim_inicio: f32,
        #[serde(default)]
        trim_fim: f32,
    },
    Ruido {
        seed: f32,
        freq: f32,
        amp: f32,
        veloc: f32,
        alvo: u8,
    },
    Anim {
        alvo: u8,
        loop_mode: u8,
        segmentos: Vec<AnimSegJson>,
    },
    Saida {
        brilho: f32,
        contraste: f32,
        saturacao: f32,
    },
    Canvas {
        largura: u32,
        altura: u32,
        fps: f32,
        duracao_seg: f32,
        fundo: [u8; 4],
    },
}

impl From<NodeParams> for NodeParamsJson {
    fn from(p: NodeParams) -> Self {
        match p {
            NodeParams::Transform { px, py, pz, rx, ry, rz, sx, sy, sz } => {
                NodeParamsJson::Transform { px, py, pz, rx, ry, rz, sx, sy, sz }
            }
            NodeParams::Cena { nome_cena, ativa, zoom, angulo, opacidade } => {
                NodeParamsJson::Cena { nome_cena, ativa, zoom, angulo, opacidade }
            }
            NodeParams::Layer { cena, layers, selected } => {
                NodeParamsJson::Layer {
                    cena,
                    layers: layers.into_iter().map(|l| LayerEntryJson {
                        nome: l.nome,
                        ordem: l.ordem,
                        opacidade: l.opacidade,
                    }).collect(),
                    selected,
                }
            }
            NodeParams::Texto { cena, conteudo, tamanho, negrito, italico, px, py, cor, trim_inicio, trim_fim, .. } => {
                NodeParamsJson::Texto {
                    cena, conteudo, tamanho, negrito, italico, px, py,
                    cor: cor.to_array(), trim_inicio, trim_fim,
                }
            }
            NodeParams::Shape {
                cena, tipo, px, py, largura, altura, rotacao, cor,
                seed, noise_scale, amp, veloc, trim_inicio, trim_fim, ..
            } => {
                NodeParamsJson::Shape {
                    cena, tipo, px, py, largura, altura, rotacao,
                    cor: cor.to_array(), seed, noise_scale, amp, veloc,
                    trim_inicio, trim_fim,
                }
            }
            NodeParams::Pen {
                cena, codigo, cor, cor_fill, pos_x, pos_y, espessura, preenchimento,
                seed, cantos, ordem, escala_x, escala_y, trim_inicio, trim_fim, ..
            } => {
                NodeParamsJson::Pen {
                    cena, codigo,
                    cor: cor.to_array(),
                    cor_fill: Some(cor_fill.to_array()),
                    pos_x, pos_y, espessura, preenchimento,
                    seed, cantos, ordem, escala_x, escala_y,
                    trim_inicio, trim_fim,
                }
            }
            NodeParams::Ruido { seed, freq, amp, veloc, alvo } => {
                NodeParamsJson::Ruido { seed, freq, amp, veloc, alvo }
            }
            NodeParams::Anim { alvo, loop_mode, segmentos } => {
                NodeParamsJson::Anim {
                    alvo,
                    loop_mode,
                    segmentos: segmentos
                        .into_iter()
                        .map(|s| AnimSegJson {
                            t_ini: s.t_ini,
                            t_fim: s.t_fim,
                            v_ini: s.v_ini,
                            v_fim: s.v_fim,
                            easing: s.easing.to_u8(),
                        })
                        .collect(),
                }
            }
            NodeParams::Saida { brilho, contraste, saturacao } => {
                NodeParamsJson::Saida { brilho, contraste, saturacao }
            }
            NodeParams::Canvas(c) => {
                NodeParamsJson::Canvas {
                    largura: c.largura,
                    altura: c.altura,
                    fps: c.fps,
                    duracao_seg: c.duracao_seg,
                    fundo: c.fundo.to_array(),
                }
            }
        }
    }
}

impl TryFrom<NodeParamsJson> for NodeParams {
    type Error = String;

    fn try_from(j: NodeParamsJson) -> Result<Self, Self::Error> {
        Ok(match j {
            NodeParamsJson::Transform { px, py, pz, rx, ry, rz, sx, sy, sz } => {
                NodeParams::Transform { px, py, pz, rx, ry, rz, sx, sy, sz }
            }
            NodeParamsJson::Cena { nome_cena, ativa, zoom, angulo, opacidade } => {
                NodeParams::Cena { nome_cena, ativa, zoom, angulo, opacidade }
            }
            NodeParamsJson::Layer { cena, layers, selected } => {
                NodeParams::Layer {
                    cena,
                    layers: layers.into_iter().map(|l| LayerEntry {
                        nome: l.nome,
                        ordem: l.ordem,
                        opacidade: l.opacidade,
                    }).collect(),
                    selected,
                }
            }
            NodeParamsJson::Texto { cena, conteudo, tamanho, negrito, italico, px, py, cor, trim_inicio, trim_fim } => {
                NodeParams::Texto {
                    cena, conteudo, tamanho, negrito, italico, px, py,
                    cor: Color32::from_rgba_unmultiplied(cor[0], cor[1], cor[2], cor[3]),
                    trim_inicio, trim_fim,
                }
            }
            NodeParamsJson::Shape {
                cena, tipo, px, py, largura, altura, rotacao, cor,
                seed, noise_scale, amp, veloc, trim_inicio, trim_fim,
            } => {
                NodeParams::Shape {
                    cena, tipo, px, py, largura, altura, rotacao,
                    cor: Color32::from_rgba_unmultiplied(cor[0], cor[1], cor[2], cor[3]),
                    seed, noise_scale, amp, veloc, trim_inicio, trim_fim,
                }
            }
            NodeParamsJson::Pen {
                cena, codigo, cor, cor_fill, pos_x, pos_y, espessura, preenchimento,
                seed, cantos, ordem, escala_x, escala_y, trim_inicio, trim_fim,
            } => {
                let cor = Color32::from_rgba_unmultiplied(cor[0], cor[1], cor[2], cor[3]);
                let cor_fill = cor_fill
                    .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
                    .unwrap_or(cor);
                NodeParams::Pen {
                    cena, codigo,
                    cor, cor_fill,
                    pos_x, pos_y, espessura, preenchimento,
                    seed, cantos, ordem, escala_x, escala_y,
                    erro: None,
                    trim_inicio, trim_fim,
                }
            }
            NodeParamsJson::Ruido { seed, freq, amp, veloc, alvo } => {
                NodeParams::Ruido { seed, freq, amp, veloc, alvo }
            }
            NodeParamsJson::Anim { alvo, loop_mode, segmentos } => {
                NodeParams::Anim {
                    alvo,
                    loop_mode,
                    segmentos: segmentos
                        .into_iter()
                        .map(|s| crate::procedural::AnimSeg {
                            t_ini: s.t_ini,
                            t_fim: s.t_fim,
                            v_ini: s.v_ini,
                            v_fim: s.v_fim,
                            easing: crate::procedural::Easing::from_u8(s.easing),
                        })
                        .collect(),
                }
            }
            NodeParamsJson::Saida { brilho, contraste, saturacao } => {
                NodeParams::Saida { brilho, contraste, saturacao }
            }
            NodeParamsJson::Canvas { largura, altura, fps, duracao_seg, fundo } => {
                NodeParams::Canvas(ProjetoConfig {
                    largura, altura, fps, duracao_seg,
                    fundo: Color32::from_rgba_unmultiplied(fundo[0], fundo[1], fundo[2], fundo[3]),
                })
            }
        })
    }
}

/// Nó serializável: tipo (rótulo), posição e params em JSON.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoJson {
    pub tipo: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub params: NodeParamsJson,
}

/// Aresta serializável: índices de origem/destino e portos.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArestaJson {
    pub de: usize,
    pub para: usize,
    pub saida: usize,
    pub saida_comp: Option<usize>,
    pub entrada: usize,
    pub entrada_comp: Option<usize>,
}

/// Snapshot completo do projeto em disco.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjetoArquivo {
    pub versao: u32,
    pub script_text: String,
    pub nos: Vec<NoJson>,
    pub arestas: Vec<ArestaJson>,
}

impl ProjetoArquivo {
    /// Constrói o arquivo a partir do estado atual do grafo e do script.
    pub fn from_graph(
        nos: &[(TipoNo, eframe::egui::Pos2, NodeParams)],
        arestas: &[(usize, usize, ArestaInfo)],
        script_text: &str,
    ) -> Self {
        let nos_json: Vec<NoJson> = nos
            .iter()
            .map(|(tipo, loc, p)| NoJson {
                tipo: tipo.nome().to_string(),
                pos_x: loc.x,
                pos_y: loc.y,
                params: NodeParamsJson::from(p.clone()),
            })
            .collect();
        let arestas_json: Vec<ArestaJson> = arestas
            .iter()
            .map(|(de, para, info)| ArestaJson {
                de: *de,
                para: *para,
                saida: info.saida,
                saida_comp: info.saida_comp,
                entrada: info.entrada,
                entrada_comp: info.entrada_comp,
            })
            .collect();
        ProjetoArquivo {
            versao: 1,
            script_text: script_text.to_string(),
            nos: nos_json,
            arestas: arestas_json,
        }
    }

    /// Converte de volta para as estruturas do grafo.
    pub fn to_graph(
        &self,
    ) -> Result<
        (
            Vec<(TipoNo, eframe::egui::Pos2, NodeParams)>,
            Vec<(usize, usize, ArestaInfo)>,
        ),
        String,
    > {
        let mut nos = Vec::new();
        for n in &self.nos {
            let tipo = TipoNo::from_label(&n.tipo)
                .ok_or_else(|| format!("tipo de nó desconhecido: {}", n.tipo))?;
            let params = NodeParams::try_from(n.params.clone())?;
            nos.push((
                tipo,
                eframe::egui::Pos2::new(n.pos_x, n.pos_y),
                params,
            ));
        }
        let mut arestas = Vec::new();
        for a in &self.arestas {
            arestas.push((
                a.de,
                a.para,
                ArestaInfo {
                    saida: a.saida,
                    saida_comp: a.saida_comp,
                    entrada: a.entrada,
                    entrada_comp: a.entrada_comp,
                },
            ));
        }
        Ok((nos, arestas))
    }
}
