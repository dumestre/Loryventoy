use std::collections::HashMap;

use eframe::egui::Pos2;

use crate::dsl::patch_dsl::{Conexao, PatchCmd};
use crate::dsl::project_dsl::{EdgeDef, Expr, NodeDef, ProjectBlock, TopLevel};
use crate::error::AppError;
use crate::nodes::{NodeParams, TipoNo};

/// Interface que a DSL usa para mutar o projeto.
/// Implementada por `GraphPanel` (produção) e por mocks (testes).
pub trait Application {
    type NodeId: Copy + Eq + std::hash::Hash + std::fmt::Debug;

    // ===== Criação/remoção de nós =====
    fn criar_no(&mut self, tipo: TipoNo, pos: Pos2) -> Self::NodeId;
    fn remover_no(&mut self, idx: Self::NodeId);

    // ===== Queries de nó =====
    fn obter_tipo(&self, idx: Self::NodeId) -> TipoNo;
    fn obter_params_mut(&mut self, idx: Self::NodeId) -> Option<&mut NodeParams>;
    #[allow(dead_code)] // patch DSL
    fn posicao_no(&self, idx: Self::NodeId) -> Option<Pos2>;
    #[allow(dead_code)]
    fn iterar_nos(&self) -> Vec<Self::NodeId>;

    // ===== Conexões =====
    fn conectar_por_nome(
        &mut self,
        src: Self::NodeId,
        saida_nome: &str,
        dst: Self::NodeId,
        entrada_nome: &str,
    );
    fn conectar_por_idx(
        &mut self,
        src: Self::NodeId,
        saida_idx: usize,
        dst: Self::NodeId,
        entrada_idx: usize,
    );
    #[allow(dead_code)] // patch DSL
    fn remover_aresta(
        &mut self,
        src: Self::NodeId,
        saida_idx: usize,
        dst: Self::NodeId,
        entrada_idx: usize,
    );

    // ===== Histórico / transação =====
    fn empurrar_historico(&mut self);

    // ===== Estado DSL =====
    fn dsl_ids(&self) -> &HashMap<String, Self::NodeId>;
    fn dsl_ids_mut(&mut self) -> &mut HashMap<String, Self::NodeId>;

    // ===== Camadas / cenas =====
    fn sync_layer_ports(&mut self);
    #[allow(dead_code)] // patch DSL
    fn limpar_grupos(&mut self);
    #[allow(dead_code)]
    fn cena_ativa(&self) -> Option<Self::NodeId>;
    #[allow(dead_code)]
    fn definir_cena_ativa(&mut self, idx: Self::NodeId);

    // ===== Configuração do projeto =====
    fn aplicar_project_config(&mut self, bloco: &ProjectBlock);

    // ===== Utilitários =====
    #[allow(dead_code)] // patch DSL
    fn encontrar_posicao_livre(&self) -> Pos2;
    fn porto_saida_por_nome(&self, tipo: TipoNo, nome: &str) -> Option<usize>;
    fn porto_entrada_por_nome(&self, tipo: TipoNo, nome: &str) -> Option<usize>;
    fn tipo_portos(&self, tipo: TipoNo) -> crate::nodes::PortSpec;
}

/// Aplica um script DSL completo ao projeto via trait Application.
pub fn aplicar_script<A: Application>(app: &mut A, codigo: &str) -> Result<(), AppError> {
    let blocos = crate::dsl::project_dsl::parse_script(codigo)?;

    app.empurrar_historico();
    app.dsl_ids_mut().clear();

    // 1. Canvas + Master
    let canvas_idx = app.criar_no(TipoNo::Canvas, Pos2::new(0.0, 0.0));
    app.dsl_ids_mut().insert("canvas".to_string(), canvas_idx);
    let master_idx = app.criar_no(TipoNo::Saida, Pos2::new(0.0, 160.0));
    app.dsl_ids_mut().insert("master".to_string(), master_idx);

    // Apply project config if present
    for bloco in &blocos {
        if let TopLevel::Project(p) = bloco {
            app.aplicar_project_config(p);
        }
    }

    // 2. Create all nodes
    for bloco in &blocos {
        if let TopLevel::Node(n) = bloco {
            let tipo = crate::dsl::project_dsl::tipo_da_dsl(&n.tipo)
                .ok_or_else(|| AppError::Dsl(format!("tipo '{}' desconhecido", n.tipo)))?;
            if matches!(tipo, TipoNo::Canvas | TipoNo::Saida) {
                continue;
            }
            let idx = app.criar_no(tipo, Pos2::ZERO);
            app.dsl_ids_mut().insert(n.id.clone(), idx);
        }
    }

    // 3. Apply parameters
    for bloco in &blocos {
        if let TopLevel::Node(n) = bloco {
            if let Some(&idx) = app.dsl_ids().get(&n.id) {
                aplicar_campos(app, idx, n)?;
            }
        }
    }

    // 4. Merge layers from same scene
    merge_layers(app)?;

    // 5. Connect edges
    for bloco in &blocos {
        if let TopLevel::Edge(e) = bloco {
            conectar_edge(app, e)?;
        }
    }

    app.sync_layer_ports();
    Ok(())
}

/// Aplica patch DSL incremental.
#[allow(dead_code)] // aguardando integração UI/IA (ver patch_dsl.rs)
pub fn aplicar_patch<A: Application>(app: &mut A, codigo: &str) -> Result<(), AppError> {
    let cmds = crate::dsl::patch_dsl::parse_patch(codigo)?;

    // Simulate to validate - just track keys in a HashSet
    let mut ids_sim: std::collections::HashSet<String> = app.dsl_ids().keys().cloned().collect();
    for cmd in &cmds {
        match cmd {
            PatchCmd::Add { tipo, id, .. } => {
                if crate::dsl::project_dsl::tipo_da_dsl(tipo).is_none() {
                    return Err(AppError::Dsl(format!("tipo '{tipo}' desconhecido")));
                }
                if ids_sim.contains(id) {
                    return Err(AppError::Dsl(format!(
                        "id '{id}' já existe (use 'set' para editar)"
                    )));
                }
                ids_sim.insert(id.clone());
            }
            PatchCmd::Set { id, .. } => {
                if !ids_sim.contains(id) {
                    return Err(AppError::Dsl(format!("nó '{id}' não existe")));
                }
            }
            PatchCmd::Remove { id } => {
                if !ids_sim.contains(id) {
                    return Err(AppError::Dsl(format!("nó '{id}' não existe")));
                }
                if id == "canvas" || id == "master" {
                    return Err(AppError::Dsl(format!(
                        "nó fixo '{id}' não pode ser removido"
                    )));
                }
                ids_sim.remove(id);
            }
            PatchCmd::Connect(c) | PatchCmd::Disconnect(c) => {
                if !ids_sim.contains(&c.de) {
                    return Err(AppError::Dsl(format!("nó '{}' não existe", c.de)));
                }
                if !ids_sim.contains(&c.para) {
                    return Err(AppError::Dsl(format!("nó '{}' não existe", c.para)));
                }
            }
        }
    }

    // All validations passed - apply for real
    app.empurrar_historico();

    for cmd in cmds {
        match cmd {
            PatchCmd::Add {
                tipo,
                id,
                campos,
                codigo,
            } => {
                let t = crate::dsl::project_dsl::tipo_da_dsl(&tipo).unwrap();
                let idx = app.criar_no(t, app.encontrar_posicao_livre());
                app.dsl_ids_mut().insert(id.clone(), idx);
                let ndef = NodeDef {
                    tipo,
                    id: id.clone(),
                    campos,
                    codigo,
                };
                aplicar_campos(app, idx, &ndef)?;
            }
            PatchCmd::Set {
                id,
                campo,
                valor,
                codigo,
            } => {
                let idx = *app.dsl_ids().get(&id).unwrap();
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
                aplicar_campos(app, idx, &ndef)?;
            }
            PatchCmd::Remove { id } => {
                if let Some(idx) = app.dsl_ids_mut().remove(&id) {
                    app.remover_no(idx);
                }
                app.limpar_grupos();
            }
            PatchCmd::Connect(c) => {
                conectar_patch(app, &c)?;
            }
            PatchCmd::Disconnect(c) => {
                desconectar_patch(app, &c)?;
            }
        }
    }
    Ok(())
}

// --- helpers internos ---

fn aplicar_campos<A: Application>(
    app: &mut A,
    idx: A::NodeId,
    n: &NodeDef,
) -> Result<(), AppError> {
    let params = match app.obter_params_mut(idx) {
        Some(p) => p,
        None => return Ok(()),
    };
    match params {
        crate::nodes::NodeParams::Cena(cena) => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "name" => cena.nome_cena = v.as_str(),
                    "zoom" => cena.zoom = v.as_num(),
                    "angle" => cena.angulo = v.as_num(),
                    "opacity" => cena.opacidade = v.as_num(),
                    _ => {}
                }
            }
        }
        crate::nodes::NodeParams::Layer(layer) => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => layer.cena = v.as_str(),
                    "name" => {
                        if let Some(entry) = layer.layers.first_mut() {
                            entry.nome = v.as_str();
                        }
                    }
                    "opacity" => {
                        if let Some(entry) = layer.layers.first_mut() {
                            entry.opacidade = v.as_num();
                        }
                    }
                    "order" => {
                        if let Some(entry) = layer.layers.first_mut() {
                            entry.ordem = v.as_num();
                        }
                    }
                    _ => {}
                }
            }
        }
        crate::nodes::NodeParams::Shape(shape) => {
            let mut cena_nome: Option<String> = None;
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => cena_nome = Some(v.as_str()),
                    "type" => {
                        shape.tipo = match v.as_str().as_str() {
                            "rect" | "rectangle" => 0,
                            "ellipse" => 1,
                            "triangle" => 2,
                            "star" => 3,
                            "losango" | "diamond" => 4,
                            "polygon" => 5,
                            "arrow" => 6,
                            _ => 0,
                        };
                    }
                    "pos" => {
                        if let Expr::Vec2(a, b) = v {
                            shape.px = *a;
                            shape.py = *b;
                        }
                    }
                    "size" => {
                        if let Expr::Vec2(a, b) = v {
                            shape.largura = *a;
                            shape.altura = *b;
                        }
                    }
                    "rotation" | "rot" => shape.rotacao = v.as_num(),
                    "color" | "colour" => {
                        let h = v.as_hex();
                        shape.cor = crate::domain::Color::from_rgba(h.r(), h.g(), h.b(), h.a());
                    }
                    "seed" => shape.seed = v.as_num(),
                    "noise" => shape.noise_scale = v.as_num(),
                    "amp" => shape.amp = v.as_num(),
                    "speed" => shape.veloc = v.as_num(),
                    "trim_start" | "trim_inicio" => shape.trim_inicio = v.as_num(),
                    "trim_end" | "trim_fim" => shape.trim_fim = v.as_num(),
                    _ => {}
                }
            }
            if let Some(cn) = cena_nome {
                shape.cena = cn;
            }
        }
        crate::nodes::NodeParams::Texto(texto) => {
            let mut cena_nome: Option<String> = None;
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => cena_nome = Some(v.as_str()),
                    "content" => texto.conteudo = v.as_str(),
                    "size" => texto.tamanho = v.as_num(),
                    "bold" => texto.negrito = v.as_str() == "true" || v.as_str() == "on",
                    "italic" => texto.italico = v.as_str() == "true" || v.as_str() == "on",
                    "pos" => {
                        if let Expr::Vec2(a, b) = v {
                            texto.px = *a;
                            texto.py = *b;
                        }
                    }
                    "color" | "colour" => {
                        let h = v.as_hex();
                        texto.cor = crate::domain::Color::from_rgba(h.r(), h.g(), h.b(), h.a());
                    }
                    "trim_start" | "trim_inicio" => texto.trim_inicio = v.as_num(),
                    "trim_end" | "trim_fim" => texto.trim_fim = v.as_num(),
                    _ => {}
                }
            }
            if let Some(cn) = cena_nome {
                texto.cena = cn;
            }
        }
        crate::nodes::NodeParams::Pen(pen) => {
            let mut cena_nome: Option<String> = None;
            for (c, v) in &n.campos {
                match c.as_str() {
                    "scene" => cena_nome = Some(v.as_str()),
                    "pos" => {
                        if let Expr::Vec2(a, b) = v {
                            pen.pos_x = *a;
                            pen.pos_y = *b;
                        }
                    }
                    "stroke" => pen.espessura = v.as_num(),
                    "fill" => pen.preenchimento = v.as_str() != "off" && v.as_str() != "false",
                    "color" | "colour" => {
                        let h = v.as_hex();
                        let c = crate::domain::Color::from_rgba(h.r(), h.g(), h.b(), h.a());
                        pen.cor = c;
                        pen.cor_fill = c;
                    }
                    "stroke_color" | "strokecolor" => {
                        let h = v.as_hex();
                        pen.cor = crate::domain::Color::from_rgba(h.r(), h.g(), h.b(), h.a());
                    }
                    "fill_color" | "fillcolor" => {
                        let h = v.as_hex();
                        pen.cor_fill = crate::domain::Color::from_rgba(h.r(), h.g(), h.b(), h.a());
                    }
                    "seed" => pen.seed = v.as_num(),
                    "corners" => pen.cantos = v.as_num(),
                    "order" => pen.ordem = v.as_num(),
                    "scalex" => pen.escala_x = v.as_num(),
                    "scaley" => pen.escala_y = v.as_num(),
                    "trim_start" | "trim_inicio" => pen.trim_inicio = v.as_num(),
                    "trim_end" | "trim_fim" => pen.trim_fim = v.as_num(),
                    _ => {}
                }
            }
            if let Some(cn) = cena_nome {
                pen.cena = cn;
            }
            if n.codigo.is_some() {
                pen.codigo = n.codigo.clone().unwrap_or_default();
            }
        }
        crate::nodes::NodeParams::Ruido(ruido) => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "seed" => ruido.seed = v.as_num(),
                    "freq" | "frequency" => ruido.freq = v.as_num(),
                    "amp" | "amplitude" => ruido.amp = v.as_num(),
                    "speed" | "veloc" => ruido.veloc = v.as_num(),
                    "target" | "alvo" => ruido.alvo = v.as_num() as u8,
                    _ => {}
                }
            }
        }
        crate::nodes::NodeParams::Anim(anim) => {
            for (c, v) in &n.campos {
                match c.as_str() {
                    "target" | "alvo" => anim.alvo = v.as_num() as u8,
                    "loop" | "loop_mode" => anim.loop_mode = v.as_num() as u8,
                    _ => {}
                }
            }
        }
        crate::nodes::NodeParams::Transform(..) => {}
        crate::nodes::NodeParams::Canvas(_) => {}
        crate::nodes::NodeParams::Saida(..) => {}
    }
    Ok(())
}

fn merge_layers<A: Application>(app: &mut A) -> Result<(), AppError> {
    use std::collections::HashMap;

    let mut scene_layers: HashMap<String, Vec<A::NodeId>> = HashMap::new();
    // First collect all layer indices
    let layer_indices: Vec<A::NodeId> = app.dsl_ids().values().copied().collect();
    for &idx in &layer_indices {
        if let Some(params) = app.obter_params_mut(idx) {
            if let crate::nodes::NodeParams::Layer(l) = params {
                scene_layers.entry(l.cena.clone()).or_default().push(idx);
            }
        }
    }

    for (_scene_name, indices) in scene_layers {
        if indices.len() > 1 {
            let primary = indices[0];
            // Collect entries to merge first
            let mut entries_to_merge: Vec<crate::domain::LayerEntry> = Vec::new();
            for &idx in &indices[1..] {
                if let Some(crate::nodes::NodeParams::Layer(other)) = app.obter_params_mut(idx) {
                    for entry in other.layers.drain(..) {
                        entries_to_merge.push(entry);
                    }
                }
            }
            // Now merge into primary
            if let Some(crate::nodes::NodeParams::Layer(primary_params)) =
                app.obter_params_mut(primary)
            {
                for entry in entries_to_merge {
                    if !primary_params.layers.iter().any(|e| e.nome == entry.nome) {
                        primary_params.layers.push(entry);
                    }
                }
            }
            // Remove merged nodes
            for &idx in &indices[1..] {
                app.remover_no(idx);
                app.dsl_ids_mut().retain(|_, &mut v| v != idx);
            }
        }
    }
    Ok(())
}

fn conectar_edge<A: Application>(app: &mut A, e: &EdgeDef) -> Result<(), AppError> {
    let src = *app
        .dsl_ids()
        .get(&e.de)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", e.de)))?;
    let dst = *app
        .dsl_ids()
        .get(&e.para)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", e.para)))?;

    let src_tipo = app.obter_tipo(src);
    let dst_tipo = app.obter_tipo(dst);

    if src_tipo == TipoNo::Layer {
        let entrada_i = app
            .porto_entrada_por_nome(dst_tipo, &e.entrada)
            .ok_or_else(|| {
                AppError::Dsl(format!(
                    "porta de entrada '{}' inválida em '{}'",
                    e.entrada, e.para
                ))
            })?;
        let port_name = app.tipo_portos(dst_tipo).entradas[entrada_i].nome;
        app.conectar_por_nome(src, &e.saida, dst, &port_name);
    } else {
        let saida_i = app
            .porto_saida_por_nome(src_tipo, &e.saida)
            .ok_or_else(|| {
                AppError::Dsl(format!(
                    "porta de saída '{}' inválida em '{}'",
                    e.saida, e.de
                ))
            })?;
        let entrada_i = app
            .porto_entrada_por_nome(dst_tipo, &e.entrada)
            .ok_or_else(|| {
                AppError::Dsl(format!(
                    "porta de entrada '{}' inválida em '{}'",
                    e.entrada, e.para
                ))
            })?;
        app.conectar_por_idx(src, saida_i, dst, entrada_i);
    }
    Ok(())
}

#[allow(dead_code)]
fn conectar_patch<A: Application>(app: &mut A, c: &Conexao) -> Result<(), AppError> {
    let src = *app
        .dsl_ids()
        .get(&c.de)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", c.de)))?;
    let dst = *app
        .dsl_ids()
        .get(&c.para)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", c.para)))?;

    let src_tipo = app.obter_tipo(src);
    let dst_tipo = app.obter_tipo(dst);

    if src_tipo == TipoNo::Layer {
        let entrada_i = app
            .porto_entrada_por_nome(dst_tipo, &c.entrada)
            .ok_or_else(|| {
                AppError::Dsl(format!(
                    "porta de entrada '{}' inválida em '{}'",
                    c.entrada, c.para
                ))
            })?;
        let port_name = app.tipo_portos(dst_tipo).entradas[entrada_i].nome;
        app.conectar_por_nome(src, &c.saida, dst, &port_name);
    } else {
        let (src2, dst2, saida_i, entrada_i) = resolver_conexao(app, c)?;
        app.conectar_por_idx(src2, saida_i, dst2, entrada_i);
    }
    Ok(())
}

#[allow(dead_code)]
fn desconectar_patch<A: Application>(app: &mut A, c: &Conexao) -> Result<(), AppError> {
    let src = *app
        .dsl_ids()
        .get(&c.de)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", c.de)))?;
    let dst = *app
        .dsl_ids()
        .get(&c.para)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", c.para)))?;

    let src_tipo = app.obter_tipo(src);
    let _dst_tipo = app.obter_tipo(dst);

    if src_tipo == TipoNo::Layer {
        // Layer disconnect is complex - delegate to concrete impl
        // For now, just skip
    } else {
        let (src2, dst2, saida_i, entrada_i) = resolver_conexao(app, c)?;
        app.remover_aresta(src2, saida_i, dst2, entrada_i);
    }
    Ok(())
}

#[allow(dead_code)]
fn resolver_conexao<A: Application>(
    app: &A,
    c: &Conexao,
) -> Result<(A::NodeId, A::NodeId, usize, usize), AppError> {
    let src = *app
        .dsl_ids()
        .get(&c.de)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", c.de)))?;
    let dst = *app
        .dsl_ids()
        .get(&c.para)
        .ok_or_else(|| AppError::Dsl(format!("nó '{}' não existe", c.para)))?;
    let src_tipo = app.obter_tipo(src);
    let dst_tipo = app.obter_tipo(dst);
    let saida_i = app
        .porto_saida_por_nome(src_tipo, &c.saida)
        .ok_or_else(|| {
            AppError::Dsl(format!(
                "porta de saída '{}' inválida em '{}'",
                c.saida, c.de
            ))
        })?;
    let entrada_i = app
        .porto_entrada_por_nome(dst_tipo, &c.entrada)
        .ok_or_else(|| {
            AppError::Dsl(format!(
                "porta de entrada '{}' inválida em '{}'",
                c.entrada, c.para
            ))
        })?;
    Ok((src, dst, saida_i, entrada_i))
}
