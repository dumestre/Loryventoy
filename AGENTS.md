# Loryventoy — Agent Memory

## Project Overview

Rust application with `eframe/egui`, node-based graph editor, procedural timeline, custom DSL, JSON serialization.

## Status by Phase

| Fase | Descrição | Status |
|---|---|---|
| 0 | Proteção e inventário | ✅ Concluída |
| 1 | Padronização de qualidade | 🟡 Parcial (`cargo fmt` tem 4 diffs preexistentes) |
| 2 | Criar domínio independente da UI | ✅ Concluída |
| 3 | Criar `Project` como fonte de verdade | ✅ Concluída |
| 4 | Dividir `GraphPanel` em módulos menores | ✅ Concluída |
| 5 | Refatorar undo/redo | ✅ Concluída (`History<Project>` integrado em `save.rs`) |
| 6 | Separar persistência e migrações | ✅ Concluída |
| 7 | Separar DSL de aplicação | ✅ Concluída |
| 8 | Separar avaliação procedural e renderização | ✅ Concluída |
| 9 | Dividir o inspector | 🟡 Iniciada (wrapper fino em `ui/inspector/mod.rs`) |
| 10 | Refatorar `Loryventoy` | ✅ Concluída (`PlaybackState`, rename `MovimentoApp` → `Loryventoy`) |
| 11 | Padronizar erros, logs e diagnósticos | 🟡 Parcial (`AppError` existe mas não integrada; 4 tipos de erro independentes) |
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
- `save.rs` — save + undo/redo logic (145 lines)
- `mod.rs` — coordinator `show()` + connections + basic queries (851 lines)

### Fase 5 — Undo/Redo (CONCLUÍDA ✅)

- `History<T>` generic struct in `src/history.rs` with `undo()`, `redo()`, `push()`
- **Fully integrated** in `graph_editor/save.rs`: `History<Project>` stores full-project snapshots
- `empurrar_historico()` serializes graph → `Project` via `to_project()` and pushes to history
- `undo()`/`redo()` pop/push `Project` and reconstruct graph via `load_project()`
- Limit of 50 entries (`LIMITE_HISTORICO`), deduplication on push
- Note: strategy is whole-graph-snapshot (not incremental commands) — adequate for current project size

- `src/projeto_arquivo.rs` deleted; content redistributed:
  - `src/infrastructure/persistence/format.rs` — JSON mirror types + `From`/`TryFrom` conversions (621 lines)
  - `src/infrastructure/persistence/migrations.rs` — versioned migration system (`VERSAO_ATUAL = 1`)
  - `src/infrastructure/persistence/repository.rs` — `load_project()`, `save_project()`, `load_from_str()` with `PersistenceError`
  - `src/infrastructure/persistence/mod.rs` — public API re-export only
- `src/main.rs` declares `mod infrastructure`
- `src/app.rs` uses repository API (`load_from_str`, `load_project`, `save_project`) instead of direct `ProjetoArquivo` usage

### Fase 7 — DSL/Application Decoupling (src/dsl/)
- `src/graph_editor/dsl.rs` (641 lines) deleted, logic moved to `src/dsl/`
- `src/dsl/application.rs` — `Application` trait (associated type `NodeId`), functions `aplicar_script<A: Application>`, `aplicar_patch<A: Application>`
- `src/dsl/evaluator.rs` — re-exports `aplicar_script`, `aplicar_patch`
- `src/dsl/pen.rs` — Pen DSL lexer/parser/evaluator (2,766 lines — largest file)
- `src/dsl/project_dsl.rs` — Project DSL parser and validator
- `src/dsl/patch_dsl.rs` — Patch DSL for incremental edits
- `GraphPanel implements Application` (`type NodeId = NodeId`)
- `app.rs` calls `crate::dsl::evaluator::aplicar_script(&mut self.graph, &text)`

### Fase 8 — Procedural Evaluation/Rendering Separation (`src/procedural/`)

- `src/procedural.rs` deleted; replaced by `src/procedural/mod.rs`
- `src/procedural/domain.rs` — pure evaluation logic, NO egui dependency (648 lines)
  - Types: `ShapeGenerator`, `PenPath`, `TextoItem`, `PreviewData`, `CenaPreview`, `LayerPreview`, `Shape`, `AnimDriver`, `RuidoDriver`
  - Functions: `generate()`, `trim_path_pts()`, `fbm()`, `ruido_offset()`
  - Uses `domain::Pos2` / `domain::Vec2` (glam), NOT egui types
- `src/procedural/render.rs` — domain → egui adapter (51 lines)
  - `shape_to_egui()`, `color_to_color32()`
- Consumers updated: `graph_editor/preview.rs`, `ui/preview.rs`, `export.rs`, `dsl/pen.rs`

### Fase 10 — Refatorar `Loryventoy` (CONCLUÍDA ✅)
- Created `src/playback.rs` with `PlaybackState` — struct dedicated to playback state (play/pause, FPS, frame accumulator, timestamp) with `update()` method
- Renamed `MovimentoApp` → `Loryventoy` in `src/app.rs` and `src/main.rs`
- Renamed log prefix `[Movimento]` → `[Loryventoy]` in `src/app.rs`
- `Loryventoy` now contains only UI composition (panels, menus, DSL windows, layout), delegating playback to `self.playback.update()`

## In Progress Phases

### Fase 9 — Dividir o inspector (INICIADA 🟡)
- Created `src/ui/inspector/mod.rs` as a thin re-export wrapper (3 lines)
- `mod.rs` re-exports `AcaoInspector` from `node_component` without behavioral change
- `node_component.rs` (1,155 lines) still contains ALL inspector logic in a single file
- Next steps: split into per-node-type files (`canvas.rs`, `scene.rs`, `layer.rs`, `shape.rs`, `text.rs`, `pen.rs`, `noise.rs`, `animation.rs`, `transform.rs`, `output.rs`), keeping the project compiling at each step

### Fase 11 — Padronizar erros, logs e diagnósticos (PARCIAL 🟡)
- `thiserror` added to `Cargo.toml`
- `src/error.rs` created with `AppError` enum using `#[derive(Error)]`:
  - `Io(std::io::Error)` via `#[from]`
  - `Parse(String)`, `InvalidProject(String)`, `Dsl(String)`, `Export(String)`, `Evaluation(String)`
- `src/log.rs` refactored with:
  - `LogLevel` enum (Error, Warn, Info, Diagnostic) with `PartialOrd`/`Ord`
  - `definir_nivel()` / `nivel_atual()` for verbosity control
  - Logs written to `logs/app.log` (not project root)
  - Functions: `erro()`, `aviso()`, `info()`, `diag()`
  - Filter by level — messages below minimum are discarded
- `eprintln!` eliminated from `app.rs` and `export.rs` (replaced by log macros)
- **NOT integrated**: `AppError` is never imported or used outside `src/error.rs`
- **4 independent error types exist** (see Error Type Inventory below) with no conversions between them

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
- `cargo fmt --check` — 4 preexistent diffs: import order (`evaluator.rs`, `export.rs`) + trailing newlines (`save.rs`, `history.rs`)
- `src/app.rs` — 885 lines
- `src/graph_editor/mod.rs` — 851 lines
- `src/ui/node_component.rs` — 1,155 lines
- `src/dsl/pen.rs` — 2,766 lines (largest file)
- **Total**: 74 `.rs` files, ~15,076 lines of Rust

## Error Type Inventory

The codebase has **4 independent error types** with overlapping variants and no conversions between them:

| Error Type | File | Variants | Used In Production |
|---|---|---|---|
| `AppError` | `src/error.rs` | `Io`, `Parse`, `InvalidProject`, `Dsl`, `Export`, `Evaluation` | **NO** — zero callers outside own file |
| `PersistenceError` | `src/infrastructure/persistence/repository.rs` | `Io`, `Parse`, `InvalidProject` | Yes — `app.rs` (save/load) |
| `ScriptError` | `src/dsl/project_dsl.rs` | `Parse { msg, linha }`, `Apply(String)` | Yes — `app.rs` (DSL evaluation) |
| `String` errors | `src/export.rs` | N/A (raw `format!` strings) | Yes — export functions |

None of these types implement `From` for each other. There is no unified `?` error propagation chain from persistence through DSL to the application layer.

## Infraestrutura Pronta (Não Integrada)

The following infrastructure exists but is not yet wired into the application flow (silenced with `allow(dead_code)`):

### Dead code — never called in production

| Item | Location | Notes |
|---|---|---|
| `AppError` enum + `is_validation()` | `src/error.rs` | Never imported outside own file; `#![allow(dead_code)]` on entire file |
| `aplicar_patch()` | `src/dsl/application.rs:135` | Entire patch DSL pipeline unreachable |
| `conectar_patch()` | `src/dsl/application.rs:524` | Called only from `aplicar_patch()` |
| `desconectar_patch()` | `src/dsl/application.rs:556` | Called only from `aplicar_patch()` |
| `resolver_conexao()` | `src/dsl/application.rs:580` | Called from `conectar_patch`/`desconectar_patch` |
| `indice_porto()` | `src/dsl/project_dsl.rs:507` | Only used in tests |
| `alias_porto()` | `src/dsl/project_dsl.rs:527` | Only called from `indice_porto()` (also test-only) |
| `generate_shape_egui()` | `src/procedural/render.rs:53` | Callers use `shape_to_egui(gen.generate(t))` directly |
| `PenPath.erro_eval` | `src/procedural/domain.rs:180` | Always set to `None`, never read |
| `Color::from_rgba_unmultiplied()` | `src/domain/color.rs:22` | Identical to `from_rgba()`; only egui equivalents used |
| `Color::from_rgba_premultiplied()` | `src/domain/color.rs:27` | Same as above |
| `History::stack_json()` | `src/history.rs:67` | Serializes history for debug; unused |

### Dead code — conditional or test-only

| Item | Location | Notes |
|---|---|---|
| `proxima_pos_livre()` | `src/graph_editor/mod.rs:790` | Used only by `Application::encontrar_posicao_livre()` impl, which is never called |
| `remover_aresta_entre()` | `src/graph_editor/mod.rs:801` | Used only by `Application::remover_aresta()` impl, which is never called |
| `icon_ico` | `build.rs:37` | Used on Windows (`#[cfg(target_os = "windows")]`); unused on other platforms |
| `History::len()` | `src/history.rs:55` | Called in tests (7 test functions in `history.rs`) |
| `History::lim()` | `src/history.rs:60` | Called in tests |

## Key Type Relationships

- `domain::Color` (r/g/b/a u8) ↔ `egui::Color32` — use `procedural::render::color_to_color32()`
- `domain::Pos2` = `glam::Vec2` ↔ `egui::Pos2` — different types, same fields
- `domain::Vec2` = `glam::Vec2` ↔ `eframe::egui::Vec2` — different types, same fields
- `domain::Shape` (procedural, no egui) ↔ `egui::Shape` — use `procedural::render::shape_to_egui()`

## Architecture Target
See [`docs/PLANO_REFATORACAO_PROFISSIONAL.md`](docs/PLANO_REFATORACAO_PROFISSIONAL.md) for the full architecture target, phase plan, contracts, and quality criteria.

## Next Recommended Step
**Finish Fase 9** — split `node_component.rs` (1,155 lines) into individual inspector editors per node type, keeping the project compiling at each step. After that, wire `AppError` into the app flow (Fase 11) to eliminate the 4 independent error types.
