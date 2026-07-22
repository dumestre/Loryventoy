#![allow(dead_code)]

use std::collections::HashMap;

use eframe::egui::Pos2;

use crate::dsl::project_dsl::{self, Expr, NodeDef, ProjectBlock, ScriptError};
use crate::dsl::patch_dsl::{self, PatchCmd};
use crate::nodes::{NodeParams, TipoNo};

use super::types::NodeId;
use super::GraphPanel;

fn aplicar_project(proj: &mut crate::nodes::ProjetoConfig, p: &ProjectBlock) {
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

fn aplicar_campos(
    panel: &mut GraphPanel,
    idx: NodeId,
    n: &NodeDef,
) -> Result<(), ScriptError> {
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
        NodeParams::Layer {
            cena,
            nome,
            opacidade,
            ordem,
            ..
        } => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => *cena = v.as_str(),
                    "name" => *nome = v.as_str(),
                    "opacity" => *opacidade = v.as_num(),
                    "order" => *ordem = v.as_num(),
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
            trim_inicio,
            trim_fim,
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
                    "trim_start" | "trim_inicio" => *trim_inicio = v.as_num(),
                    "trim_end" | "trim_fim" => *trim_fim = v.as_num(),
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
            trim_inicio,
            trim_fim,
            cena,
            ..
        } => {
            let mut cena_nome: Option<String> = None;
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => cena_nome = Some(v.as_str()),
                    "content" => *conteudo = v.as_str(),
                    "size" => *tamanho = v.as_num(),
                    "bold" => {
                        *negrito = v.as_str() == "true" || v.as_str() == "on"
                    }
                    "italic" => {
                        *italico = v.as_str() == "true" || v.as_str() == "on"
                    }
                    "pos" => {
                        if let Expr::Vec2(a, b) = v {
                            *px = *a;
                            *py = *b;
                        }
                    }
                    "color" | "colour" => *cor = v.as_hex(),
                    "trim_start" | "trim_inicio" => *trim_inicio = v.as_num(),
                    "trim_end" | "trim_fim" => *trim_fim = v.as_num(),
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
            trim_inicio,
            trim_fim,
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
                    "fill" => {
                        *preenchimento =
                            v.as_str() != "off" && v.as_str() != "false"
                    }
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
                    "trim_start" | "trim_inicio" => *trim_inicio = v.as_num(),
                    "trim_end" | "trim_fim" => *trim_fim = v.as_num(),
                    _ => {}
                }
            }
            if let Some(cn) = cena_nome {
                *cena = cn;
            }
            if n.codigo.is_some() {
                *codigo = n.codigo.clone().unwrap_or_default();
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
                    "freq" | "frequency" => *freq = v.as_num(),
                    "amp" | "amplitude" => *amp = v.as_num(),
                    "speed" | "veloc" => *veloc = v.as_num(),
                    "target" | "alvo" => *alvo = v.as_num() as u8,
                    _ => {}
                }
            }
        }
        NodeParams::Anim {
            alvo,
            loop_mode,
            ..
        } => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "target" | "alvo" => *alvo = v.as_num() as u8,
                    "loop" | "loop_mode" => *loop_mode = v.as_num() as u8,
                    _ => {}
                }
            }
        }
        NodeParams::Transform { .. } => {}
        NodeParams::Canvas(_) => {}
        NodeParams::Saida { .. } => {}
    }

    Ok(())
}

impl GraphPanel {
    pub fn aplicar_script(
        &mut self,
        codigo: &str,
    ) -> Result<(), ScriptError> {
        use project_dsl::{indice_porto, parse_script, tipo_da_dsl, TopLevel};

        let prog = parse_script(codigo)?;

        let mut proj = crate::nodes::ProjetoConfig::default();
        for tl in &prog {
            if let TopLevel::Project(p) = tl {
                aplicar_project(&mut proj, p);
            }
        }

        self.criar_nos_padrao();
        if let Some(ci) = self.canvas {
            if let Some(NodeParams::Canvas(c)) = self.params.get_mut(&ci) {
                *c = proj;
            }
        }

        let mut ids: HashMap<String, NodeId> = HashMap::new();
        if let Some(c) = self.canvas {
            ids.insert("canvas".to_string(), c);
        }
        if let Some(c) = self.cena {
            ids.insert("scene".to_string(), c);
        }
        if let Some(m) = self.master {
            ids.insert("master".to_string(), m);
        }

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

        for tl in &prog {
            if let TopLevel::Edge(e) = tl {
                let src = *ids.get(&e.de).ok_or_else(|| {
                    ScriptError::Apply(format!(
                        "nó '{}' não definido",
                        e.de
                    ))
                })?;
                let dst = *ids.get(&e.para).ok_or_else(|| {
                    ScriptError::Apply(format!(
                        "nó '{}' não definido",
                        e.para
                    ))
                })?;
                let src_tipo = self.obter_tipo(src);
                let dst_tipo = self.obter_tipo(dst);
                let saida_i =
                    indice_porto(src_tipo, &e.saida, true).ok_or_else(|| {
                        ScriptError::Apply(format!(
                            "porta de saída '{}' inválida em '{}'",
                            e.saida, e.de
                        ))
                    })?;
                let entrada_i =
                    indice_porto(dst_tipo, &e.entrada, false).ok_or_else(|| {
                        ScriptError::Apply(format!(
                            "porta de entrada '{}' inválida em '{}'",
                            e.entrada, e.para
                        ))
                    })?;
                self.conectar_por_idx(src, saida_i, dst, entrada_i);
            }
        }

        Ok(())
    }

    pub fn aplicar_patch(
        &mut self,
        codigo: &str,
    ) -> Result<(), ScriptError> {
        let cmds = patch_dsl::parse_patch(codigo)?;

        let mut ids_sim: std::collections::HashSet<String> =
            self.dsl_ids.keys().cloned().collect();
        for cmd in &cmds {
            match cmd {
                PatchCmd::Add { tipo, id, .. } => {
                    if project_dsl::tipo_da_dsl(tipo).is_none() {
                        return Err(ScriptError::Apply(format!(
                            "tipo '{tipo}' desconhecido"
                        )));
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
                        return Err(ScriptError::Apply(format!(
                            "nó '{id}' não existe"
                        )));
                    }
                }
                PatchCmd::Remove { id } => {
                    if !ids_sim.contains(id) {
                        return Err(ScriptError::Apply(format!(
                            "nó '{id}' não existe"
                        )));
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
                        return Err(ScriptError::Apply(format!(
                            "nó '{}' não existe",
                            c.de
                        )));
                    }
                    if !ids_sim.contains(&c.para) {
                        return Err(ScriptError::Apply(format!(
                            "nó '{}' não existe",
                            c.para
                        )));
                    }
                }
            }
        }

        self.empurrar_historico();

        for cmd in cmds {
            match cmd {
                PatchCmd::Add {
                    tipo,
                    id,
                    campos,
                    codigo,
                } => {
                    let t = project_dsl::tipo_da_dsl(&tipo).unwrap();
                    let idx =
                        self.adicionar_no_em(t, self.proxima_pos_livre());
                    self.dsl_ids.insert(id.clone(), idx);
                    let ndef = NodeDef {
                        tipo,
                        id,
                        campos,
                        codigo,
                    };
                    aplicar_campos(self, idx, &ndef)?;
                }
                PatchCmd::Set {
                    id,
                    campo,
                    valor,
                    codigo,
                } => {
                    let idx = *self.dsl_ids.get(&id).unwrap();
                    let ndef = NodeDef {
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
                        self.remover_no(idx);
                    }
                    self.limpar_grupos();
                }
                PatchCmd::Connect(c) => {
                    let (src, dst, saida_i, entrada_i) =
                        self.resolver_conexao(&c)?;
                    self.conectar_por_idx(src, saida_i, dst, entrada_i);
                }
                PatchCmd::Disconnect(c) => {
                    let (src, dst, saida_i, entrada_i) =
                        self.resolver_conexao(&c)?;
                    self.remover_aresta_entre(src, saida_i, dst, entrada_i);
                }
            }
        }
        Ok(())
    }

    fn resolver_conexao(
        &self,
        c: &patch_dsl::Conexao,
    ) -> Result<(NodeId, NodeId, usize, usize), ScriptError> {
        let src = *self.dsl_ids.get(&c.de).ok_or_else(|| {
            ScriptError::Apply(format!("nó '{}' não existe", c.de))
        })?;
        let dst = *self.dsl_ids.get(&c.para).ok_or_else(|| {
            ScriptError::Apply(format!("nó '{}' não existe", c.para))
        })?;
        let src_tipo = self.obter_tipo(src);
        let dst_tipo = self.obter_tipo(dst);
        let saida_i =
            project_dsl::indice_porto(src_tipo, &c.saida, true).ok_or_else(
                || {
                    ScriptError::Apply(format!(
                        "porta de saída '{}' inválida em '{}'",
                        c.saida, c.de
                    ))
                },
            )?;
        let entrada_i =
            project_dsl::indice_porto(dst_tipo, &c.entrada, false)
                .ok_or_else(|| {
                    ScriptError::Apply(format!(
                        "porta de entrada '{}' inválida em '{}'",
                        c.entrada, c.para
                    ))
                })?;
        Ok((src, dst, saida_i, entrada_i))
    }

    pub(super) fn remover_aresta_entre(
        &mut self,
        src: NodeId,
        saida: usize,
        dst: NodeId,
        entrada: usize,
    ) {
        let output_id = self.editor_state.graph[src]
            .outputs
            .get(saida)
            .map(|(_, id)| *id);

        let input_id = self.editor_state.graph[dst]
            .inputs
            .get(entrada)
            .map(|(_, id)| *id);

        if let (Some(out), Some(inp)) = (output_id, input_id) {
            let mut alvo = None;
            for (input_conn, output_conn) in self.editor_state.graph.iter_connections() {
                if input_conn == inp && output_conn == out {
                    alvo = Some(input_conn);
                    break;
                }
            }
            if let Some(input_to_remove) = alvo {
                self.editor_state.graph.remove_connection(input_to_remove);
            }
        }
    }

    pub(super) fn proxima_pos_livre(&self) -> Pos2 {
        let mut max_y = 0.0f32;
        for nid in self.editor_state.graph.iter_nodes() {
            if let Some(pos) = self.editor_state.node_positions.get(nid) {
                max_y = max_y.max(pos.y);
            }
        }
        Pos2::new(0.0, max_y + 160.0)
    }
}
