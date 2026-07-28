# Loryventoy — Agent Memory

## Project Overview

Rust application with `eframe/egui`, node-based graph editor, procedural timeline, custom DSL, JSON serialization.

Plano completo: [`docs/PLANO_REFATORACAO_PROFISSIONAL.md`](docs/PLANO_REFATORACAO_PROFISSIONAL.md)

## Status das Fases

| Fase | Descrição | Status |
|------|-----------|--------|
| 0–1 | Proteção, inventário, padronização | 🟡 Parcial (`cargo fmt --check` ainda difere) |
| 2–3 | Domínio independente + `Project` como fonte de verdade | ✅ Concluída |
| 4 | Dividir `GraphPanel` em submódulos | ✅ Concluída |
| 5 | Refatorar undo/redo sobre `Project` | 🟡 Parcial — `History<T>` genérico existe; undo ainda acoplado ao grafo |
| 6 | Separar persistência (`infrastructure/persistence/`) | ✅ Concluída |
| 7 | Separar DSL de aplicação (`src/dsl/`) | ✅ Concluída |
| 8 | Separar avaliação procedural e renderização | ✅ Concluída |
| 9 | Dividir inspector (`node_component.rs` → `ui/inspector/`) | 🟡 Iniciada — wrapper fino; implementação ainda monolítica (~1200 linhas) |
| 10 | Refatorar app principal (`Loryventoy`) | ✅ Concluída — `PlaybackState` extraído; `app.rs` ainda ~995 linhas |
| 11 | Padronizar erros, logs e diagnósticos | 🟡 Parcial — `AppError` + `log.rs` criados; `AppError` ainda não integrado ao fluxo |
| 12 | Testes de regressão adicionais | ❌ Pendente |

## Current Status

```text
cargo check       OK — sem warnings
cargo test --all  OK — 87 testes
cargo fmt --check diferenças preexistentes (não aplicadas)
```

## Fases Concluídas (detalhe)

### Fase 2/3 — Domain Extraction & Project as Source of Truth

- Created `src/domain/` with domain-independent types:
  - `Color` (RGBA, no egui dependency) in `domain/color.rs`
  - `ProjectConfig` in `domain/project_config.rs`
  - `NodeParams`, 9 param structs in `domain/params.rs`
  - `Project`, `ProjectNode`, `ProjectEdge` in `domain/project.rs`
  - `TipoNo` in `domain/node_type.rs`
  - `LayerEntry` in `domain/layer_entry.rs` (colors use domain `Color`, not egui `Color32`)
  - math re-exports (`retangulo_rot`, `elipse_rot`, `poligono_regular`, `estrela`) in `domain/math.rs`
- `GraphPanel::to_project()` / `load_project()` use `domain::Project` as single source of truth
- Persistence uses `from_project()` / `to_project()` via repository API
- `app.rs` updated for new API

### Fase 4 — GraphPanel Split (`src/graph_editor/`)

Split into specialized submodules:

- `node_factory.rs` — `criar_nos_padrao`, `adicionar_no_em`, `adicionar_no`
- `layer_ops.rs` — `cenas_disponiveis`, `normalizar_cena`, `sync_layer_ports`, CRUD layers
- `layout.rs` — spatial query methods (hit test, port positions, coordinates)
- `search.rs` — text search by name/type
- `mod.rs` — coordinator `show()` + connections + basic queries (~939 linhas atuais)

### Fase 6 — Persistence Separation (`src/infrastructure/persistence/`)

- `src/projeto_arquivo.rs` deleted; content redistributed:
  - `format.rs` — JSON mirror types + `From`/`TryFrom` conversions (~350 lines)
  - `migrations.rs` — versioned migration system (`VERSAO_ATUAL = 1`)
  - `repository.rs` — `load_project()`, `save_project()`, `load_from_str()` with `PersistenceError`
  - `mod.rs` — public API re-export only
- `src/main.rs` declares `mod infrastructure`
- `src/app.rs` uses repository API

### Fase 7 — DSL/Application Decoupling (`src/dsl/`)

- `src/graph_editor/dsl.rs` deleted; logic moved to `src/dsl/`
- `src/dsl/application.rs` — `Application` trait, `aplicar_script`, `aplicar_patch`
- `src/dsl/evaluator.rs` — re-exports `aplicar_script`, `aplicar_patch`
- `src/dsl/patch_dsl.rs` — parser de patch incremental (pronto, aguardando UI/IA)
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

### Fase 10 — App Principal (`Loryventoy`)

- `MovimentoApp` renomeado para `Loryventoy` em `app.rs` e `main.rs`
- Criado `src/playback.rs` com `PlaybackState` — play/pause, FPS, acumulador de frames
- App delega playback para `self.playback.update()`
- Prefixo de log `[Movimento]` → `[Loryventoy]`

## Fases Parciais / Em Andamento

### Fase 5 — Undo/Redo

- `src/history.rs` — `History<T>` genérico com push/undo/redo
- GraphPanel usa `History<Project>` para snapshots
- Pendente: histórico 100% sobre operações de domínio, transações DSL formalizadas

### Fase 9 — Inspector

- Criado `src/ui/inspector/mod.rs` como re-export fino de `node_component.rs`
- Pendente: extrair editores por tipo de nó (`canvas.rs`, `scene.rs`, `layer.rs`, `shape.rs`, `text.rs`, `pen.rs`, etc.) e `common.rs` para helpers compartilhados

### Fase 11 — Erros, Logs e Diagnósticos

- `src/error.rs` — `AppError` enum com `thiserror` (Io, Parse, InvalidProject, Dsl, Export, Evaluation)
- `src/log.rs` — `LogLevel`, `erro!`/`aviso!`/`info!`/`diag!`, logs em `logs/app.log`
- `eprintln!` eliminado de `app.rs` e `export.rs`
- Pendente: integrar `AppError` nos fluxos de salvar/carregar/DSL/export; remover `#![allow(dead_code)]` de `error.rs` após integração

## Próximo Passo Recomendado

**Fase 9** — dividir `src/ui/node_component.rs` (~1200 linhas) em módulos em `src/ui/inspector/`, mantendo o projeto compilando a cada extração.

## Key Type Relationships

- `domain::Color` (r/g/b/a u8) ↔ `egui::Color32` — use `procedural::render::color_to_color32()`
- `domain::Pos2` = `glam::Vec2` ↔ `egui::Pos2` — different types, same fields
- `domain::Vec2` = `glam::Vec2` ↔ `eframe::egui::Vec2` — different types, same fields
- `domain::Shape` (procedural, no egui) ↔ `egui::Shape` — use `procedural::render::shape_to_egui()`

## Infraestrutura Pronta (não integrada)

Código preparado para fases futuras, silenciado com `#[allow(dead_code)]` documentado:

- `AppError` e `is_validation()` — Fase 11
- `aplicar_patch` + métodos do trait `Application` usados só pelo patch DSL — Fase 7/IA
- `parse_patch` / `patch_dsl.rs` — aguardando UI ou agente IA
- `generate_shape_egui`, `History::stack_json`, `erro_eval` em `PenPath` — conveniência/diagnóstico
