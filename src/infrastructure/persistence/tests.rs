#[cfg(test)]
mod tests {
    use crate::domain::{Color, NodeParams, Project, ProjectEdge, ProjectNode, TipoNo};
    use crate::error::AppError;
    use crate::infrastructure::persistence::format::ProjetoArquivo;
    use crate::infrastructure::persistence::{load_from_str, save_project};
    use crate::nodes::{CenaParams, SaidaParams, ShapeParams};

    fn projeto_minimo() -> Project {
        Project {
            script_text: "// teste".into(),
            nodes: vec![
                ProjectNode {
                    tipo: TipoNo::Canvas,
                    pos_x: 0.0,
                    pos_y: 0.0,
                    params: NodeParams::Canvas(crate::domain::ProjectConfig::default()),
                },
                ProjectNode {
                    tipo: TipoNo::Cena,
                    pos_x: 100.0,
                    pos_y: 100.0,
                    params: NodeParams::Cena(CenaParams {
                        nome_cena: "Cena 1".into(),
                        ativa: true,
                        zoom: 1.0,
                        angulo: 0.0,
                        opacidade: 1.0,
                    }),
                },
                ProjectNode {
                    tipo: TipoNo::Saida,
                    pos_x: 200.0,
                    pos_y: 200.0,
                    params: NodeParams::Saida(SaidaParams {
                        brilho: 1.0,
                        contraste: 1.0,
                        saturacao: 1.0,
                    }),
                },
            ],
            edges: vec![ProjectEdge {
                from: 1,
                to: 2,
                from_port: 0,
                from_comp: None,
                to_port: 0,
                to_comp: None,
            }],
        }
    }

    #[test]
    fn projeto_para_json_e_volta() {
        let proj = projeto_minimo();
        let arquivo = ProjetoArquivo::from_project(&proj);
        let json = serde_json::to_string_pretty(&arquivo).expect("serializa");
        let mut de: ProjetoArquivo = serde_json::from_str(&json).expect("desserializa");
        crate::infrastructure::persistence::migrations::migrate(&mut de);
        let proj2 = de.to_project().expect("converte");
        assert_eq!(proj.nodes.len(), proj2.nodes.len());
        assert_eq!(proj.edges.len(), proj2.edges.len());
        assert_eq!(proj.script_text, proj2.script_text);
    }

    #[test]
    fn save_e_load_roundtrip() {
        let proj = projeto_minimo();
        let dir = std::env::temp_dir().join("lory_test_save_load");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("teste.lory");
        save_project(&path, &proj).expect("save");
        let loaded = crate::infrastructure::persistence::load_project(&path).expect("load");
        assert_eq!(loaded.nodes.len(), proj.nodes.len());
        assert_eq!(loaded.script_text, proj.script_text);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_from_str_parse_erro() {
        let r = load_from_str("json inválido {{}{");
        assert!(r.is_err());
        match r.unwrap_err() {
            AppError::Parse(_) => {} // ok
            other => panic!("esperado Parse, veio {other}"),
        }
    }

    #[test]
    fn projeto_com_cor_persiste() {
        let proj = Project {
            script_text: String::new(),
            nodes: vec![ProjectNode {
                tipo: TipoNo::Shape,
                pos_x: 0.0,
                pos_y: 0.0,
                params: NodeParams::Shape(ShapeParams {
                    cena: String::new(),
                    tipo: 0,
                    px: 0.0,
                    py: 0.0,
                    largura: 100.0,
                    altura: 100.0,
                    rotacao: 0.0,
                    cor: Color::from_rgb(255, 128, 64),
                    seed: 42.0,
                    noise_scale: 0.0,
                    amp: 0.0,
                    veloc: 0.0,
                    trim_inicio: 0.0,
                    trim_fim: 1.0,
                }),
            }],
            edges: vec![],
        };
        let arquivo = ProjetoArquivo::from_project(&proj);
        let json = serde_json::to_string(&arquivo).expect("serializa");
        assert!(json.contains("\"tipo\":\"Shape\""));
        assert!(json.contains("255") || json.contains("\"cor\""));
        let mut de: ProjetoArquivo = serde_json::from_str(&json).expect("desserializa");
        crate::infrastructure::persistence::migrations::migrate(&mut de);
        let proj2 = de.to_project().expect("converte");
        assert_eq!(proj2.nodes.len(), 1);
        if let NodeParams::Shape(s) = &proj2.nodes[0].params {
            assert_eq!(s.cor, Color::from_rgb(255, 128, 64));
        } else {
            panic!("esperado Shape");
        }
    }
}
