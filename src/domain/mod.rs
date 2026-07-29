//! Tipos centrais do domínio, independentes da camada visual.

mod animation;
mod color;
mod layer_entry;
mod math;
mod node_type;
mod project_config;

// ── Parâmetros de nós ────────────────────────────────────────────
mod anim_params;
mod cena_params;
mod layer_params;
mod params;
mod pen_params;
mod ruido_params;
mod saida_params;
mod shape_params;
mod text_params;
mod transform_params;

// ── Projeto ──────────────────────────────────────────────────────
mod project;

pub use animation::{AnimSeg, Easing, LoopMode};
pub use color::{Color, Pos2, Vec2};
pub use layer_entry::LayerEntry;
pub use math::*;
pub use node_type::TipoNo;
pub use project_config::ProjectConfig;

pub use anim_params::AnimParams;
pub use cena_params::CenaParams;
pub use layer_params::LayerParams;
pub use params::NodeParams;
pub use pen_params::PenParams;
pub use ruido_params::RuidoParams;
pub use saida_params::SaidaParams;
pub use shape_params::ShapeParams;
pub use text_params::TextParams;
pub use transform_params::TransformParams;

pub use project::{Project, ProjectEdge, ProjectNode};

#[cfg(test)]
mod tests {
    use super::{
        AnimSeg, Color, Easing, LayerEntry, LoopMode, NodeParams, Project,
        ProjectConfig, ProjectEdge, ProjectNode, TipoNo, TransformParams,
        anim_params,
        cena_params,
        layer_params,
        math::{elipse_rot, estrela, poligono_regular, retangulo_rot},
        ruido_params,
        saida_params,
        shape_params,
        text_params,
    };

    // ── Config ──

    #[test]
    fn configuracao_padrao_preserva_contrato_atual() {
        let cfg = ProjectConfig::default();
        assert_eq!(cfg.largura, 1920);
        assert_eq!(cfg.altura, 1080);
        assert_eq!(cfg.fps, 30.0);
        assert_eq!(cfg.duracao_seg, 5.0);
        assert_eq!(cfg.fundo, Color::WHITE);
    }

    // ── Color ──

    #[test]
    fn cor_preserva_rgba() {
        let cor = Color::from_rgba(10, 20, 30, 40);
        assert_eq!(cor.to_rgba(), [10, 20, 30, 40]);
    }

    #[test]
    fn cor_from_rgb_presume_alpha_255() {
        let cor = Color::from_rgb(255, 0, 128);
        assert_eq!(cor.to_rgba(), [255, 0, 128, 255]);
    }

    #[test]
    fn cor_white_const() {
        assert_eq!(Color::WHITE.to_rgba(), [255, 255, 255, 255]);
    }

    #[test]
    fn cor_metodos_individuais() {
        let cor = Color::from_rgba(10, 20, 30, 40);
        assert_eq!(cor.r(), 10);
        assert_eq!(cor.g(), 20);
        assert_eq!(cor.b(), 30);
        assert_eq!(cor.a(), 40);
    }

    // ── Easing ──

    #[test]
    fn easing_linear_mapeia_direto() {
        let e = Easing::Linear;
        assert!((e.aplicar(0.0) - 0.0).abs() < 1e-6);
        assert!((e.aplicar(0.5) - 0.5).abs() < 1e-6);
        assert!((e.aplicar(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn easing_ease_in_quadratico() {
        let e = Easing::EaseIn;
        assert!((e.aplicar(0.5) - 0.25).abs() < 1e-6);
        assert_eq!(e.aplicar(0.0), 0.0);
        assert_eq!(e.aplicar(1.0), 1.0);
    }

    #[test]
    fn easing_ease_out_quadratico() {
        let e = Easing::EaseOut;
        assert!((e.aplicar(0.5) - 0.75).abs() < 1e-6);
        assert_eq!(e.aplicar(0.0), 0.0);
        assert_eq!(e.aplicar(1.0), 1.0);
    }

    #[test]
    fn easing_ease_in_out_ponto_medio() {
        let e = Easing::EaseInOut;
        assert!((e.aplicar(0.5) - 0.5).abs() < 1e-6);
        assert_eq!(e.aplicar(0.0), 0.0);
        assert_eq!(e.aplicar(1.0), 1.0);
    }

    #[test]
    fn easing_step_salta_no_fim() {
        let e = Easing::Step;
        assert_eq!(e.aplicar(0.0), 0.0);
        assert_eq!(e.aplicar(0.999), 0.0);
        assert_eq!(e.aplicar(1.0), 1.0);
    }

    #[test]
    fn easing_clamp_fora_do_intervalo() {
        assert_eq!(Easing::Linear.aplicar(-0.5), 0.0);
        assert_eq!(Easing::Linear.aplicar(1.5), 1.0);
    }

    #[test]
    fn easing_from_u8_to_u8_roundtrip() {
        for v in 0..=4 {
            let e = Easing::from_u8(v);
            assert_eq!(e.to_u8(), v);
        }
    }

    #[test]
    fn easing_from_u8_desconhecido_vai_linear() {
        assert_eq!(Easing::from_u8(99), Easing::Linear);
    }

    // ── LoopMode ──

    #[test]
    fn loop_mode_from_u8() {
        assert_eq!(LoopMode::from_u8(0), LoopMode::Nenhum);
        assert_eq!(LoopMode::from_u8(1), LoopMode::Repetir);
        assert_eq!(LoopMode::from_u8(2), LoopMode::PingPong);
        assert_eq!(LoopMode::from_u8(99), LoopMode::Nenhum);
    }

    // ── AnimSeg ──

    #[test]
    fn anim_seg_pode_ser_construido() {
        let seg = AnimSeg {
            t_ini: 0.0,
            t_fim: 2.0,
            v_ini: [0.0, 0.0],
            v_fim: [100.0, 200.0],
            easing: Easing::Linear,
        };
        assert_eq!(seg.t_ini, 0.0);
        assert_eq!(seg.t_fim, 2.0);
        assert_eq!(seg.v_fim, [100.0, 200.0]);
    }

    // ── LayerEntry ──

    #[test]
    fn layer_entry_palette_cicla() {
        let c0 = LayerEntry::cor_por_idx(0);
        let c8 = LayerEntry::cor_por_idx(8);
        assert_eq!(c0, c8);
    }

    #[test]
    fn layer_entry_palette_nao_vazia() {
        for i in 0..8 {
            let c = LayerEntry::cor_por_idx(i);
            assert_ne!(c, Color::from_rgba(0, 0, 0, 0));
        }
    }

    // ── TipoNo ──

    #[test]
    fn tipo_no_label_roundtrip() {
        for tipo in &[
            TipoNo::Saida, TipoNo::Transform, TipoNo::Canvas, TipoNo::Cena,
            TipoNo::Layer, TipoNo::Shape, TipoNo::Texto, TipoNo::Pen,
            TipoNo::Ruido, TipoNo::Anim,
        ] {
            let label = tipo.nome();
            let parsed = TipoNo::from_label(label);
            assert_eq!(parsed, Some(*tipo), "label {label:?} não fez roundtrip");
        }
    }

    #[test]
    fn tipo_no_from_label_desconhecido() {
        assert_eq!(TipoNo::from_label("Inexistente"), None);
    }

    #[test]
    fn tipo_no_pode_conectar_saida_entrada() {
        assert!(TipoNo::pode_conectar(TipoNo::Cena, TipoNo::Saida));
        assert!(!TipoNo::pode_conectar(TipoNo::Saida, TipoNo::Cena));
    }

    #[test]
    fn tipo_no_pode_conectar_scene_para_scene() {
        assert!(TipoNo::pode_conectar(TipoNo::Cena, TipoNo::Cena));
    }

    #[test]
    fn tipo_no_pode_conectar_layer_a_shape() {
        assert!(TipoNo::pode_conectar(TipoNo::Layer, TipoNo::Shape));
    }

    #[test]
    fn tipo_no_nao_pode_conectar_invalidos() {
        assert!(!TipoNo::pode_conectar(TipoNo::Saida, TipoNo::Saida));
        assert!(!TipoNo::pode_conectar(TipoNo::Canvas, TipoNo::Shape));
    }

    // ── Geometry ──

    #[test]
    fn retangulo_rot_centro_zero_sem_rotacao() {
        let pts = retangulo_rot(glam::Vec2::ZERO, glam::Vec2::new(100.0, 50.0), 0.0);
        assert_eq!(pts.len(), 4);
        assert_eq!(pts[0], glam::Vec2::new(-50.0, -25.0));
        assert_eq!(pts[2], glam::Vec2::new(50.0, 25.0));
    }

    #[test]
    fn retangulo_rot_180_graus_inverte() {
        let pts = retangulo_rot(glam::Vec2::ZERO, glam::Vec2::new(100.0, 50.0), 180.0);
        assert!((pts[0].x - 50.0).abs() < 1e-4);
        assert!((pts[0].y - 25.0).abs() < 1e-4);
    }

    #[test]
    fn elipse_rot_numero_de_pontos() {
        let pts = elipse_rot(glam::Vec2::ZERO, 50.0, 30.0, 0.0);
        assert_eq!(pts.len(), 48);
    }

    #[test]
    fn poligono_regular_triangulo() {
        let pts = poligono_regular(glam::Vec2::new(100.0, 100.0), 50.0, 3, 0.0);
        assert_eq!(pts.len(), 3);
        for p in &pts {
            let d = (*p - glam::Vec2::new(100.0, 100.0)).length();
            assert!((d - 50.0).abs() < 1e-5);
        }
    }

    #[test]
    fn estrela_pontas_dobro() {
        let pts = estrela(glam::Vec2::ZERO, 50.0, 25.0, 5, 0.0);
        assert_eq!(pts.len(), 10);
    }

    // ── NodeParams ──

    #[test]
    fn node_params_todas_as_variantes_existem() {
        use super::PenParams;
        let _ = NodeParams::Transform(TransformParams {
            px: 0.0, py: 0.0, pz: 0.0,
            rx: 0.0, ry: 0.0, rz: 0.0,
            sx: 1.0, sy: 1.0, sz: 1.0,
        });
        let _ = NodeParams::Canvas(ProjectConfig::default());
        let _ = NodeParams::Cena(cena_params::CenaParams {
            nome_cena: String::new(),
            ativa: true,
            zoom: 1.0,
            angulo: 0.0,
            opacidade: 1.0,
        });
        let _ = NodeParams::Layer(layer_params::LayerParams {
            cena: String::new(),
            layers: vec![],
            selected: 0,
        });
        let _ = NodeParams::Shape(shape_params::ShapeParams {
            cena: String::new(),
            tipo: 0,
            px: 0.0, py: 0.0,
            largura: 100.0,
            altura: 100.0,
            rotacao: 0.0,
            cor: Color::WHITE,
            seed: 0.0,
            noise_scale: 0.0,
            amp: 0.0,
            veloc: 0.0,
            trim_inicio: 0.0,
            trim_fim: 1.0,
        });
        let _ = NodeParams::Texto(text_params::TextParams {
            cena: String::new(),
            conteudo: String::new(),
            tamanho: 32.0,
            negrito: false,
            italico: false,
            px: 0.0, py: 0.0,
            cor: Color::WHITE,
            trim_inicio: 0.0,
            trim_fim: 1.0,
        });
        let _ = NodeParams::Pen(PenParams {
            cena: String::new(),
            codigo: String::new(),
            pos_x: 0.0, pos_y: 0.0,
            espessura: 1.0,
            preenchimento: false,
            cantos: 0.0,
            ordem: 0.0,
            escala_x: 1.0, escala_y: 1.0,
            seed: 0.0,
            cor: Color::from_rgb(0, 0, 0),
            cor_fill: Color::from_rgb(0, 0, 0),
            erro: None,
            trim_inicio: 0.0,
            trim_fim: 1.0,
        });
        let _ = NodeParams::Ruido(ruido_params::RuidoParams {
            alvo: 0,
            seed: 0.0,
            freq: 0.1,
            amp: 10.0,
            veloc: 1.0,
        });
        let _ = NodeParams::Anim(anim_params::AnimParams {
            alvo: 0,
            loop_mode: 0,
            segmentos: vec![],
        });
        let _ = NodeParams::Saida(saida_params::SaidaParams {
            brilho: 1.0,
            contraste: 1.0,
            saturacao: 1.0,
        });
    }

    // ── Project ──

    #[test]
    fn project_vazio_sem_canvas_config_padrao() {
        let p = Project {
            script_text: String::new(),
            nodes: vec![],
            edges: vec![],
        };
        assert_eq!(p.config(), ProjectConfig::default());
    }

    #[test]
    fn project_config_extrai_do_canvas() {
        let mut cfg = ProjectConfig::default();
        cfg.largura = 640;
        cfg.altura = 480;
        let p = Project {
            script_text: "test".into(),
            nodes: vec![ProjectNode {
                tipo: TipoNo::Canvas,
                pos_x: 0.0,
                pos_y: 0.0,
                params: NodeParams::Canvas(cfg.clone()),
            }],
            edges: vec![],
        };
        assert_eq!(p.config().largura, 640);
        assert_eq!(p.config().altura, 480);
    }

    #[test]
    fn project_edge_pode_ser_construido() {
        let e = ProjectEdge {
            from: 0,
            to: 1,
            from_port: 0,
            from_comp: None,
            to_port: 0,
            to_comp: None,
        };
        assert_eq!(e.from, 0);
        assert_eq!(e.to, 1);
    }
}
