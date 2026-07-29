# Loryventoy — Agent Memory

## Project Overview

Rust application with `eframe/egui`, node-based graph editor, procedural timeline, custom DSL, JSON serialization.

## Status by Phase

| Fase | Descrição | Status |
|---|---|---|---|
| 0 | Proteção e inventário | ✅ Concluída |
| 1 | Padronização de qualidade | ✅ Concluída |
| 2 | Criar domínio independente da UI | ✅ Concluída |
| 3 | Criar `Project` como fonte de verdade | ✅ Concluída |
| 4 | Dividir `GraphPanel` em módulos menores | ✅ Concluída |
| 5 | Refatorar undo/redo | ✅ Concluída (`History<Project>` integrado em `save.rs`) |
| 6 | Separar persistência e migrações | ✅ Concluída |
| 7 | Separar DSL de aplicação | ✅ Concluída |
| 8 | Separar avaliação procedural e renderização | ✅ Concluída |
| 9 | Dividir o inspector | ✅ Concluída |
| 10 | Refatorar `Loryventoy` | ✅ Concluída (`PlaybackState`, rename `MovimentoApp` → `Loryventoy`) |
| 11 | Padronizar erros, logs e diagnósticos | ✅ Concluída |
| 12 | Testes de regressão adicionais | ✅ Concluída |
| 13 | Limpeza de dead code, warnings e formatação | ✅ Concluída |

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

### Fase 13 — Limpeza de dead code, warnings e formatação (CONCLUÍDA ✅)
- Removido `AppError::Evaluation` (variant) e `is_validation()` de `src/error.rs` (2 warnings eliminados)
- Removidos métodos mortos `from_rgba_unmultiplied`/`from_rgba_premultiplied` de `src/domain/color.rs`
- Removido campo `erro_eval` de `PenPath` em `src/procedural/domain.rs` e de construtores em `preview.rs` e `export.rs`
- Removida função `generate_shape_egui` de `src/procedural/render.rs`
- Removido pipeline patch DSL completo: `aplicar_patch`, `conectar_patch`, `desconectar_patch`, `resolver_conexao` + métodos mortos da `Application` trait
- Removidos `proxima_pos_livre` e `remover_aresta_entre` de `src/graph_editor/mod.rs`
- `cargo fmt` executado (zero diffs restantes)
- `cargo check` — 0 warnings

## In Progress Phases

### Fase 9 — Dividir o inspector (CONCLUÍDA ✅)
- `src/ui/node_component.rs` reduced to 6-line re-export wrapper
- `src/ui/inspector/` split into per-node-type files:
  - `canvas.rs` (76 lines) — resolução/presets
  - `scene.rs` (39 lines) — parâmetros de cena
  - `layer.rs` (207 lines) — layers, header, rows, rename
  - `shape.rs` (74 lines) — parâmetros de forma
  - `text.rs` (70 lines) — parâmetros de texto
  - `pen.rs` (131 lines) — editor Pen DSL
  - `noise.rs` (11 lines) — parâmetros de ruído
  - `animation.rs` (80 lines) — segmentos de animação
  - `transform.rs` (15 lines) — transform/output
- `mod.rs` (527 lines) — `show_content()` dispatch + helpers compartilhados

### Fase 11 — Padronizar erros, logs e diagnósticos (CONCLUÍDA ✅)
- `thiserror` added to `Cargo.toml`
- `src/error.rs` — `AppError` enum unificado com `#[derive(Error)]`:
  - `Io(std::io::Error)`, `Parse(String)`, `InvalidProject(String)`, `Dsl(String)`, `DslParse { msg, linha }`, `Export(String)`
- `src/log.rs` refactored with log levels, file output in `logs/app.log`, level filtering
- `eprintln!` eliminated from `app.rs` and `export.rs`
- **Tipos eliminados**: `PersistenceError` (repository.rs) e `ScriptError` (project_dsl.rs) removidos; `AppError` usado em todo lugar. `AppError::Evaluation` removido na Fase 13 (nunca construído).
- **`export.rs`**: `Result<(), String>` → `Result<(), AppError>`
- **`app.rs`**: usa `AppError` diretamente via `load_project`/`save_project`/`aplicar_script`
- **Testes**: 89/89 passam (2 novos testes de erro)

## Pending Phases

### Fase 12 — Testes de regressão adicionais (CONCLUÍDA ✅)
- **Domain** (27 testes): Color (RGBA, from_rgb, from_rgba, White, métodos), Easing (5 easing types, clamp, from_u8 roundtrip), LoopMode (from_u8), LayerEntry (palette), TipoNo (label roundtrip, conexões válidas/inválidas), geometria (retângulo/elipse/polígono/estrela), NodeParams (10 variantes), Project (config padrão, edge)
- **Persistence** (4 testes): JSON roundtrip, save/load com arquivo temp, JSON inválido, cor persiste
- **DSL Application** (3 testes): script projeto simples, script com pen, erro tipo desconhecido
- **DSL Project** (7 testes adicionais): script vazio, só comentários, hex curto (3 chars), hex com 3 números RGB, bloco vazio, edge para master, color RGB
- **Procedural Domain** (3 testes): geração retângulo, geração elipse, trim cria Path
- **Total**: **133 testes passando** (era 89)

## Current Metrics
- `cargo test --all` — **133/133 PASS**
- `cargo check` — **0 warnings**
- `cargo fmt --check` — **0 diffs**
- `src/app.rs` — 884 lines
- `src/graph_editor/mod.rs` — 851 lines
- `src/ui/node_component.rs` — 6 lines (re-export)
- `src/dsl/pen.rs` — 2,766 lines (largest file)
- `src/dsl/application.rs` — 561 lines (aplicar_campos restored)
- **Total**: 84 `.rs` files, ~15,791 lines of Rust

## Error Type Inventory

Fully unified under `AppError` — `PersistenceError`, `ScriptError`, and `AppError::Evaluation` have been eliminated. All code paths (persistence, DSL, export) use `AppError` with `?` propagation.

## Infraestrutura Pronta (Não Integrada)

The following infrastructure exists but is not yet wired into the application flow (silenced with `allow(dead_code)`):

### Dead code — never called in production

| Item | Location | Notes |
|---|---|---|
| `indice_porto()` | `src/dsl/project_dsl.rs:507` | Only used in tests |
| `alias_porto()` | `src/dsl/project_dsl.rs:527` | Only called from `indice_porto()` (also test-only) |
| `History::stack_json()` | `src/history.rs:67` | Serializes history for debug; unused |

### Dead code — conditional or test-only

| Item | Location | Notes |
|---|---|---|
| `icon_ico` | `build.rs:37` | Used on Windows (`#[cfg(target_os = "windows")]`); unused on other platforms |
| `History::len()` | `src/history.rs:55` | Called in tests |
| `History::lim()` | `src/history.rs:60` | Called in tests |

## Key Type Relationships
