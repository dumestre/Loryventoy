# Loryventoy — Agent Memory

## Project Overview

Rust application with `eframe/egui`, node-based graph editor, procedural timeline, custom DSL, JSON serialization.

## Status by Phase

| Fase | Descrição | Status |
|---|---|---|
| 0 | Proteção e inventário | ✅ Concluída |
| 1 | Padronização de qualidade | 🟡 Parcial (formatting preexistente não resolvida) |
| 2 | Criar domínio independente da UI | ✅ Concluída |
| 3 | Criar `Project` como fonte de verdade | ✅ Concluída |
| 4 | Dividir `GraphPanel` em módulos menores | ✅ Concluída |
| 5 | Refatorar undo/redo | 🟡 Parcial (`History<T>` existe, undo ainda acoplado ao grafo) |
| 6 | Separar persistência e migrações | ✅ Concluída |
| 7 | Separar DSL de aplicação | ✅ Concluída |
| 8 | Separar avaliação procedural e renderização | ✅ Concluída |
| 9 | Dividir o inspector | 🟡 Iniciada (wrapper fino em `ui/inspector/mod.rs`) |
| 10 | Refatorar `Loryventoy` | ✅ Concluída (`PlaybackState`, rename `MovimentoApp` → `Loryventoy`) |
| 11 | Padronizar erros, logs e diagnósticos | 🟡 Parcial (`AppError`/`log.rs` existem, mas `AppError` não está no fluxo da app) |
| 12 | Testes de regressão adicionais | ❌ Pendente |

## Completed Phases in Detail

### Fase 2/3 — Domain Extraction & Project as Source of Truth

- Created `src/domain/` with domain-independent types:
  - `Color` (RGBA, no egui dependency) in `domain/color.rs`
  - `ProjectConfig` in `domain/project_config.rs`
  - `NodeParams`, 9 param structs in `domain/params.rs`
  - `Project`, `ProjectNode`, `ProjectEdge` in `domain/project.rs`
  - `TipoNo` in `domain/node_type.rs`
  - `LayerEntry` in `domain/layer_entry.rs` (colors use domain `Color`, not egui `Color32`)
  - math re-exports (`retangulo_rot`, `elipse_rot`, `poligono_regular`, `estrela`) in `domain/math.rs`
  - `Animation` types (`Easing`, `LoopMode`, `AnimSeg`) in `domain/animation.rs`
- `GraphPanel::to_project()` / `load_project()` use `domain::Project` as single source of truth
- Persistence uses `from_project()` / `to_project()` via repository API
- `app.rs` updated for new API

### Fase 4 — GraphPanel Split (`src/graph_editor/`)

Split into specialized submodules:

- `node_factory.rs` — `criar_nos_padrao`, `adicionar_no_em`, `adicionar_no`
- `layer_ops.rs` — `cenas_disponiveis`, `normalizar_cena`, `sync_layer_ports`, CRUD layers
- `layout.rs` — spatial query methods (hit test, port positions, coordinates)
- `search.rs` — text search by name/type
- `types.rs` — type aliases and constants
- `ports.rs` — port-related utilities
- `selection.rs` — selection state and operations
- `groups.rs` — group operations
- `rendering.rs` — rendering helpers
- `save.rs` — save-related logic
- `mod.rs` — coordinator `show()` + connections + basic queries (~937 lines)

### Fase 5 — Undo/Redo (Partial)
- `History<T>` generic struct exists in `src/history.rs` with `undo()`, `redo()`, `push()`, `clear()`
- `History<T>` is **not yet integrated** as the undo/redo backend for `GraphPanel`
- Undo/redo still operates on graph snapshots directly
- `len()`, `lim()`, `stack_json()` methods exist but are unused (`allow(dead_code)`)

- `src/projeto_arquivo.rs` deleted; content redistributed:
  - `src/infrastructure/persistence/format.rs` — JSON mirror types + `From`/`TryFrom` conversions (~350 lines)
  - `src/infrastructure/persistence/migrations.rs` — versioned migration system (`VERSAO_ATUAL = 1`)
  - `src/infrastructure/persistence/repository.rs` — `load_project()`, `save_project()`, `load_from_str()` with `PersistenceError`
  - `src/infrastructure/persistence/mod.rs` — public API re-export only
- `src/main.rs` declares `mod infrastructure`
- `src/app.rs` uses repository API (`load_from_str`, `load_project`, `save_project`) instead of direct `ProjetoArquivo` usage

### Fase 7 — DSL/Application Decoupling (src/dsl/)
- `src/graph_editor/dsl.rs` (641 lines) deleted, logic moved to `src/dsl/`
- `src/dsl/application.rs` — `Application` trait (associated type `NodeId`), functions `aplicar_script<A: Application>`, `aplicar_patch<A: Application>`
- `src/dsl/evaluator.rs` — re-exports `aplicar_script`, `aplicar_patch`
- `src/dsl/pen.rs` — Pen DSL lexer/parser/evaluator
- `src/dsl/project_dsl.rs` — Project DSL parser and validator
- `src/dsl/patch_dsl.rs` — Patch DSL for incremental edits
- `GraphPanel implements Application` (`type NodeId = NodeId`)
- `app.rs` calls `crate::dsl::evaluator::aplicar_script(&mut self.graph, &text)`

### Fase 8 — Procedural Evaluation/Rendering Separation (`src/procedural/`)

- `src/procedural.rs` deleted; replaced by `src/procedural/mod.rs`
- `src/procedural/domain.rs` — pure evaluation logic, NO egui dependency
  - Types: `ShapeGenerator`, `PenPath`, `TextoItem`, `PreviewData`, `CenaPreview`, `LayerPreview`, `Shape`, `AnimDriver`, `RuidoDriver`
  - Functions: `generate()`, `trim_path_pts()`, `fbm()`, `ruido_offset()`
  - Uses `domain::Pos2` / `domain::Vec2` (glam), NOT egui types
- `src/procedural/render.rs` — domain → egui adapter
  - `shape_to_egui()`, `generate_shape_egui()`, `color_to_color32()`
- Consumers updated: `graph_editor/preview.rs`, `ui/preview.rs`, `export.rs`, `dsl/pen.rs`

## In Progress Phases

### Fase 9 — Dividir o inspector (INICIADA 🟡)
- Created `src/ui/inspector/mod.rs` as a thin wrapper over `node_component.rs`
- `mod.rs` re-exports `AcaoInspector` from `node_component` without behavioral change
- `node_component.rs` (1198 lines) still contains all inspector logic in a single file
- Next steps: split into `canvas.rs`, `scene.rs`, `layer.rs`, `shape.rs`, `text.rs`, `pen.rs`, `noise.rs`, `animation.rs`, `transform.rs`, `output.rs`

### Fase 10 — Refatorar `Loryventoy` (CONCLUÍDA ✅)
- Created `src/playback.rs` with `PlaybackState` — struct dedicated to playback state (play/pause, FPS, frame accumulator, timestamp) with `update()` method
- Renamed `MovimentoApp` → `Loryventoy` in `src/app.rs` and `src/main.rs`
- Renamed log prefix `[Movimento]` → `[Loryventoy]` in `src/app.rs`
- `Loryventoy` now contains only UI composition (panels, menus, DSL windows, layout), delegating playback to `self.playback.update()`

### Fase 11 — Padronizar erros, logs e diagnósticos (PARCIAL 🟡)
- `thiserror` added to `Cargo.toml`
- `src/error.rs` created with `AppError` enum using `#[derive(Error)]`:
  - `Io(std::io::Error)` via `#[from]`
  - `Parse(String)`
  - `InvalidProject(String)`
  - `Dsl(String)`
  - `Export(String)`
  - `Evaluation(String)`
- `src/log.rs` refactored with:
  - `LogLevel` enum (Error, Warn, Info, Diagnostic) with `PartialOrd`/`Ord`
  - `definir_nivel()` / `nivel_atual()` for verbosity control
  - Logs written to `logs/app.log` (not project root)
  - Functions: `erro()`, `aviso()`, `info()`, `diag()`
  - Filter by level — messages below minimum are discarded
- `eprintln!` eliminated from `app.rs` (save/load use `info!`/`erro!`; performance metrics use `diag!`)
- `eprintln!` eliminated from `export.rs` (replaced by `aviso!`)
- `src/log.rs` added to `src/main.rs`; `src/error.rs` added to `src/main.rs`
- **Not yet integrated**: `AppError` is never imported or used in `app.rs` or anywhere in the app flow — it exists as infrastructure but is not wired into the application logic

## Pending Phases

### Fase 12 — Testes de regressão adicionais (PENDENTE ❌)
- Domain tests (project creation, defaults, validation, connections, IDs, layers, scenes, animation, easing, noise, trim, geometry)
- Application tests (add/remove nodes, connect/disconnect, undo/redo, grouping, selection, parameter changes, command application, transactional failures)
- Persistence tests (save/load, version migration, corrupted files, invalid types, invalid connections)
- DSL tests (project parser, patch parser, Pen DSL, line/column messages, port validation, transactional application + undo)
- Rendering tests (deterministic preview, time points, loop modes, text, shapes, Pen, PNG export)

## Current Metrics
- `cargo test --all` — **87/87 PASS**
- `cargo check` — compiles with **18 warnings** (1 build script + 15 binary + 2 preexisting)
- `cargo fmt --check` — preexistent formatting differences not yet fixed
- `src/app.rs` — 994 lines
- `src/graph_editor/mod.rs` — 937 lines
- `src/ui/node_component.rs` — 1198 lines

## Infraestrutura Pronta (Não Integrada)

The following infrastructure exists but is not yet wired into the application flow (silenced with `allow(dead_code)`):

| Item | Location | Status |
|---|---|---|
| `AppError` enum | `src/error.rs` | Defined but never imported or used |
| `AppError::is_validation()` | `src/error.rs:25` | Never called |
| `aplicar_patch()` | `src/dsl/application.rs:127` | Never called |
| `conectar_patch()` | `src/dsl/application.rs:515` | Never called |
| `desconectar_patch()` | `src/dsl/application.rs:546` | Never called |
| `resolver_conexao()` | `src/dsl/application.rs:569` | Never called |
| `indice_porto()` | `src/dsl/project_dsl.rs:506` | Never called |
| `alias_porto()` | `src/dsl/project_dsl.rs:525` | Never called |
| `proxima_pos_livre()` | `src/graph_editor/mod.rs:789` | Never called |
| `remover_aresta_entre()` | `src/graph_editor/mod.rs:799` | Never called |
| `History::len()` / `History::lim()` | `src/history.rs:54,58` | Never called |
| `History::stack_json()` | `src/history.rs:64` | Never called |
| `PenPath.erro_eval` | `src/procedural/domain.rs:179` | Never read |
| `generate_shape_egui()` | `src/procedural/render.rs:52` | Never called |
| `Color::from_rgba_unmultiplied()` | `src/domain/color.rs:21` | Never called |
| `Color::from_rgba_premultiplied()` | `src/domain/color.rs:25` | Never called |
| `icon_ico` | `build.rs:37` | Unused variable |

## Key Type Relationships

- `domain::Color` (r/g/b/a u8) ↔ `egui::Color32` — use `procedural::render::color_to_color32()`
- `domain::Pos2` = `glam::Vec2` ↔ `egui::Pos2` — different types, same fields
- `domain::Vec2` = `glam::Vec2` ↔ `eframe::egui::Vec2` — different types, same fields
- `domain::Shape` (procedural, no egui) ↔ `egui::Shape` — use `procedural::render::shape_to_egui()`

## Architecture Target
See [`docs/PLANO_REFATORACAO_PROFISSIONAL.md`](docs/PLANO_REFATORACAO_PROFISSIONAL.md) for the full architecture target, phase plan, contracts, and quality criteria.

## Next Recommended Step
**Finish Fase 9** — split `node_component.rs` (1198 lines) into individual inspector editors per node type, keeping the project compiling at each step.
