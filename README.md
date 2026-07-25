# Loryventoy

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL_3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)
[![Rust](https://img.shields.io/badge/rust-1.78%2B-orange.svg)](https://www.rust-lang.org)
[![eframe](https://img.shields.io/badge/egui-0.35-45a2e1.svg)](https://github.com/emilk/egui)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

**Editor de motion graphics procedural baseado em nós** — alternativa open-source a After Effects, Cavalry, Motion.

## Visão geral

Loryventoy é um editor de animação e motion design com paradigma **node-based** (grafo de nós) + **timeline**. Permite criar animações complexas compondo nós visuais (Canvas, Cena, Layer, Shape, Texto, Pen, Ruído, Animação, Transform, Saída) e controlando parâmetros ao longo do tempo via keyframes/marcadores.

## Principais recursos

- **Graph Editor** — Nós arrastáveis, conexões tipo cabo, zoom/pan estilo Blender (scroll = zoom, MMB = pan), caixas de seleção, agrupamento de nós
- **Timeline** — Marcadores de tempo, playhead, loop, FPS configurável, duração do projeto
- **Preview em tempo real** — Renderização GPU via `lyon` + `cosmic-text`, sincronizada com timeline
- **Nós built-in**:
  - `Canvas` — Configuração de resolução, FPS, duração, cor de fundo (presets 4K, Full HD, vertical, quadrado)
  - `Cena` — Câmera virtual (zoom, ângulo, opacidade), marcadores de tempo
  - `Layer` — Pilha de layers com ordem, visibilidade, cor, opacidade, renomeação por double-click
  - `Shape` — Retângulo, elipse, triângulo, estrela, losango, polígono, seta; parâmetros procedurais (ruído, seed, amplitude, velocidade, trim)
  - `Texto` — Fonte monoespaçada/proporcional, negrito/itálico, tamanho, cor, trim
  - `Pen` — Path DSL próprio (`move`, `line`, `rect`, `circle`, `bezier`, `close`, `fill`, `stroke`, `color`, `repeat`, `if`…), live parsing + error highlight
  - `Ruído` — Noise 1D/2D aplicável a Posição/Rotação/Escala
  - `Animação` — Segmentos com easing (Linear, Ease-in, Ease-out, Ease-in-out, Step), loop ping-pong
  - `Transform` — Posição/Rotação/Escala 3D
  - `Saída` — Brilho, contraste, saturação final
- **Inspector lateral** — Parâmetros do nó selecionado com DragValue (scroll horizontal = passo rápido), color picker, combos
- **Salvar/Carregar projetos** — JSON serializ (`.lory`) com versionamento
- **Tema escuro** customizado, ícones SVG inline

## Arquitetura

```
src/
├── app.rs              # App principal (eframe)
├── main.rs             # Entry point
├── nodes/              # Definição de nós e parâmetros
│   ├── mod.rs          # Tipos centrais (TipoNo, NodeParams, LayerEntry, PortSpec)
│   ├── canvas.rs       # Nó Canvas
│   ├── cena.rs         # Nó Cena
│   ├── layer.rs        # Nó Layer
│   ├── shape.rs        # Nó Shape
│   ├── texto.rs        # Nó Texto
│   ├── pen.rs          # Nó Pen (DSL)
│   ├── ruido.rs        # Nó Ruído
│   ├── anim.rs         # Nó Animação
│   ├── transform.rs    # Nó Transform
│   └── saida.rs        # Nó Saída
├── graph_editor/       # Wrapper sobre egui-graph-edit
│   ├── mod.rs          # GraphPanel, ações, sync de portas de Layer
│   ├── types.rs        # NodeTemplate, UserState, traits do egui-graph-edit
│   ├── selection.rs    # Copiar/colar/duplicar/deletar/agrupar
│   ├── ports.rs        # Layout de portas (estilo Blender)
│   ├── dsl.rs          # Export DSL do grafo
│   ├── preview.rs      # Render do grafo para preview
│   ├── save.rs         # Snapshots para undo/redo
│   └── groups.rs       # Agrupamento visual de nós
├── ui/
│   ├── node_component.rs  # Inspector + header/rows de Layer
│   ├── graph.rs           # Painel do graph editor
│   ├── preview.rs         # Painel de preview
│   ├── timeline.rs        # Painel de timeline
│   ├── bartool.rs         # Barra de ferramentas superior
│   ├── splitter.rs        # Splitters redimensionáveis
│   └── hub.rs             # Tela inicial (hub)
├── procedural/       # Tipos de animação (AnimSeg, Easing)
├── dsl/              # Parser/interpretador DSL do Pen
├── projeto_arquivo.rs    # Serialização JSON (.lory)
└── theme.rs          # Tema escuro customizado
```

## Build & Run

```bash
# Debug
cargo run

# Release otimizado
cargo run --release
```

Requer Rust **1.78+** (MSRV). No Windows, o ícone `app.ico` é embutido no executável via `winres`.

## Dependências principais

| Crate | Uso |
|-------|-----|
| `eframe` / `egui` | UI imediata |
| `egui-graph-edit` | Graph editor (nós, conexões, layout) |
| `lyon` | Tesselação de paths 2D (shapes, pen) |
| `cosmic-text` | Layout/shaping de texto |
| `noise` | Ruído Perlin/Simplex |
| `glam` | Matemática vetorial |
| `serde` / `serde_json` | Serialização de projetos |
| `image` | Carregamento de ícones/export |

## Atalhos úteis

| Ação | Atalho |
|------|--------|
| Zoom no graph | Scroll |
| Pan no graph | MMB (botão do meio) ou `Shift` + drag |
| Criar nó | `Tab` ou botão "+" na toolbar |
| Deletar nó(s) | `Delete` / `Backspace` |
| Duplicar | `Ctrl+D` |
| Copiar / Colar | `Ctrl+C` / `Ctrl+V` |
| Agrupar selecionados | `Ctrl+G` |
| Play / Pause timeline | `Espaço` |
| Voltar ao frame 0 | `Home` |

## DSL do Pen (exemplos)

```dsl
# Retângulo 200x100 centralizado
move -100 -50
rect 200 100
fill on
color #FF6B6B

# Estrela 5 pontas animada
repeat 5 {
  move 0 0
  line 100 0
  rotate 144
}
stroke 4
stroke_color #FFFFFF
```

Comandos: `move`, `line`, `rect`, `circle`, `bezier`, `close`, `fill on/off`, `stroke <px>`, `color <hex>`, `stroke_color <hex>`, `fill_color <hex>`, `repeat N { ... }`, `if <cond> { ... }`.

## Formato de projeto (`.lory`)

JSON com estrutura:
```json
{
  "version": 2,
  "nodes": [
    { "id": 1, "type": "Canvas", "pos": [0,0], "params": { "largura": 1920, "altura": 1080, "fps": 24, "duracao_seg": 10, "fundo": [0,0,0,255] } },
    { "id": 2, "type": "Layer", "pos": [300,100], "params": { "cena": "Cena 1", "layers": [{ "nome": "Layer 1", "ordem": 0, "opacidade": 1.0, "cor": [90,170,235,255], "visivel": true }], "selected": 0 } }
  ],
  "edges": [ { "from": [1, "Canvas"], "to": [2, "Canvas"] } ],
  "groups": [],
  "timeline_markers": [ { "time": 0.0, "label": "Inicio", "color": "#FF0000" } ]
}
```

## Roadmap / TODO

- [ ] Keyframes por parâmetro (curvas de animação no inspector)
- [ ] Export de vídeo (FFmpeg) / GIF / PNG sequence
- [ ] Nós de máscara / clipping / blend modes
- [ ] Plugin system (WASM ou dynamic libs)
- [ ] Undo/Redo global (parcialmente implementado via snapshots)
- [ ] Snapping de nós no grid
- [ ] Search/fuzzy finder de nós (`Tab` já abre)

## Licença

GPL-3.0 — veja `LICENSE` (ou cabeçalho do `Cargo.toml`).

---

**Desenvolvido com Rust + egui**. Contribuições bem-vindas!+