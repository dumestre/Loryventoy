# Loryventoy — Agent Memory

## Project Overview
Rust application with `eframe/egui`, node-based graph editor, procedural timeline, custom DSL, JSON serialization.

## Completed Phases (Fases 2–8)

### Fase 2/3 — Domain Extraction & Project as Source of Truth
- Created `src/domain/` with domain-independent types:
  - `Color` (RGBA, no egui dependency) in `domain/color.rs`
  - `ProjectConfig` in `domain/project_config.rs`
  - `NodeParams`, 9 param structs in `domain/params.rs` (params in `domain/` subfolder)
  - `Project`, `ProjectNode`, `ProjectEdge` in `domain/project.rs`
  - `TipoNo` in `domain/node_type.rs`
  - `LayerEntry` in `domain/layer_entry.rs` (colors use domain `Color`, not egui `Color32`)
  - math re-exports (`retangulo_rot`, `elipse_rot`, `poligono_regular`, `estrela`) in `domain/math.rs`
- `GraphPanel::to_project()` / `load_project()` use `domain::Project` as single source of truth
- Removed `from_graph()`/`to_graph()` from `ProjetoArquivo` (replaced by `from_project()`/`to_project()`)
- `app.rs` updated for new API

### Fase 4 — GraphPanel Split (src/graph_editor/)
Split `mod.rs` (~1106 lines) into specialized submodules:
- `node_factory.rs` — `criar_nos_padrao`, `adicionar_no_em`, `adicionar_no`
- `layer_ops.rs` — `cenas_disponiveis`, `normalizar_cena`, `sync_layer_ports`, CRUD layers
- `layout.rs` — 6 spatial query methods (hit test, port positions, coordinates)
- `search.rs` — text search by name/type
- `mod.rs` — ~535 lines, coordinator `show()` + connections + basic queries

### Fase 6 — Persistence Separation (src/infrastructure/persistence/)
- `src/projeto_arquivo.rs` deleted; content redistributed:
  - `src/infrastructure/persistence/format.rs` — JSON mirror types + `From`/`TryFrom` conversions (~350 lines)
  - `src/infrastructure/persistence/migrations.rs` — versioned migration system (`VERSAO_ATUAL = 1`)
  - `src/infrastructure/persistence/repository.rs` — `load_project()`, `save_project()`, `load_from_str()` with `PersistenceError`
  - `src/infrastructure/persistence/mod.rs` — public API re-export only
- `src/main.rs` updated to declare `mod infrastructure`
- `src/app.rs` uses repository API instead of direct `ProjetoArquivo` usage

### Fase 7 — DSL/Application Decoupling (src/dsl/)
- `src/graph_editor/dsl.rs` (641 lines) deleted, logic moved to `src/dsl/`
- `src/dsl/application.rs` — `Application` trait (associated type `NodeId`), functions `aplicar_script<A: Application>`, `aplicar_patch<A: Application>`
- `src/dsl/evaluator.rs` — re-exports `aplicar_script`, `aplicar_patch`
- `GraphPanel implements Application` (`type NodeId = NodeId`)
- `app.rs` calls `crate::dsl::evaluator::aplicar_script(&mut self.graph, &text)`

### Fase 8 — Procedural Evaluation/Rendering Separation (src/procedural/)
- `src/procedural.rs` deleted; replaced by `src/procedural/mod.rs`
- `src/procedural/domain.rs` — pure evaluation logic, NO egui dependency
  - Types: `ShapeGenerator`, `PenPath`, `TextoItem`, `PreviewData`, `CenaPreview`, `LayerPreview`, `Shape`, `AnimDriver`, `RuidoDriver`
  - Functions: `generate()`, `trim_path_pts()`, `fbm()`, `ruido_offset()`
  - `Trim_path_pts` uses `domain::Pos2` (glam::Vec2), NOT egui's `Pos2`
- `src/procedural/render.rs` — domain → egui conversion adapter
  - `shape_to_egui()` converts `domain::Shape` → `egui::Shape`
  - `generate_shape_egui()` evaluates + converts in one step
  - `color_to_color32()` converts `domain::Color` → `Color32`
- `src/graph_editor/preview.rs` uses `preview_data()` → builds `PreviewData` (domain types), then converts for rendering
- `src/ui/preview.rs`:
  - Uses `shape_to_egui()` to convert domain shapes before `aplicar_opacidade()` and `translate_shape()`
  - Uses `color_to_color32()` for all domain Color → Color32 conversions (text, pens, background)
  - Builds `pts` buffer using `domain::Pos2` (glam::Vec2) — NOT egui's `Pos2` — for `trim_path_pts()`
  - Uses `domain::Vec2` instead of removed `GVec2` alias
- `src/export.rs`:
  - Uses `shape_to_egui()` + `traduzir()` for domain→egui conversion
  - Uses `color_to_color32()` for pen, text, and background colors
  - Test code uses `domain::Color::from_rgb()` (no `YELLOW`/`RED` constants exist on domain `Color`)
- `src/dsl/pen.rs`: all `GVec2` references migrated to `crate::domain::Vec2` (~15 occurrences)

## Current Status
- `cargo check` — OK (warnings only)
- `cargo test --all` — 68/68 PASS
- No `cargo fmt --check` fix applied (preexistent formatting differences)

## Key Type Relationships
- `domain::Color` (r/g/b/a u8) ↔ `egui::Color32` — use `procedural::render::color_to_color32()`
- `domain::Pos2` = `glam::Vec2` ↔ `egui::Pos2` — different types, same fields
- `domain::Vec2` = `glam::Vec2` ↔ `eframe::egui::Vec2` — different types, same fields
- `domain::Shape` (procedural, no egui) ↔ `egui::Shape` — use `procedural::render::shape_to_egui()`

## Fases Pendentes (9–12)
- Fase 9: Dividir inspector (`src/ui/node_component.rs`)
- Fase 10: Refatorar `MovimentoApp`
- Fase 11: Padronizar erros, logs e diagnósticos
- Fase 12: Testes de regressão adicionais