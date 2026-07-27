# Plano Geral de Refatoração Profissional do Loryventoy

## 1. Objetivo

Este documento define o caminho completo para transformar o código atual do Loryventoy em uma base profissional, organizada, previsível e sustentável, mantendo os recursos existentes.

O objetivo não é criar novas funcionalidades. O objetivo é:

- separar responsabilidades;
- reduzir acoplamento;
- melhorar a segurança das alterações;
- tornar o projeto testável;
- preservar o comportamento atual;
- preparar o código para crescer sem virar um bloco monolítico;
- melhorar a clareza para manutenção futura;
- aproximar a qualidade estrutural de editores profissionais como After Effects, Cavalry e ferramentas node-based maduras.

O trabalho deve ser feito de maneira incremental. Cada etapa precisa deixar o projeto compilando e testável.

---

## 2. Estado atual conhecido

O projeto é uma aplicação Rust com `eframe/egui`, editor de grafo, timeline, preview procedural, DSL própria e serialização JSON.

Comandos verificados no estado analisado:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

Os principais pontos de concentração encontrados:

| Arquivo | Situação atual | Risco |
|---|---|---|
| `src/app.rs` | Janela principal, menus, playback, projeto, DSL e layout | Médio |
| `src/graph_editor/mod.rs` | Coordenador de módulos especializados (~535 linhas) | Médio |
| `src/ui/preview.rs` | Renderização, texto, Pen DSL e conversões visuais | Médio |
| `src/procedural/domain.rs` | Lógica pura de avaliação (sem egui) | Baixo |
| `src/procedural/render.rs` | Adaptador domain → egui | Baixo |
| `src/dsl/pen.rs` | Parser e avaliador grande, com boa cobertura de testes | Médio |
| `src/domain/` | Camada de domínio consolidada | Baixo (concluída) |

A existência de testes é uma boa base. A refatoração deve preservar essa base e aumentá-la antes de mover código crítico.

### Progresso da execução

Em 26/07/2026 foi concluída a primeira fatia de implementação deste plano:

- criada a camada inicial `src/domain/`;
- criado o tipo `domain::Color`, independente de `egui`;
- criado o tipo `domain::ProjectConfig`, independente de `egui`;
- `nodes::ProjetoConfig` passou a ser uma compatibilidade temporária para `domain::ProjectConfig`;
- preview recebeu conversão explícita de cor de domínio para `egui::Color32`;
- inspector recebeu conversão temporária para editar cor sem contaminar o domínio;
- DSL de projeto passou a converter a cor para o tipo de domínio;
- persistência JSON passou a usar conversão explícita RGBA;
- adicionados dois testes de proteção do domínio;
- nenhum recurso novo foi criado;
- o formato `.lory` não foi alterado.

Validação realizada após a alteração:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

Observação: `cargo fmt --check` ainda acusa diferenças de formatação preexistentes em vários arquivos do projeto. A formatação global não foi aplicada nesta etapa para evitar um diff grande e não relacionado à migração.

Próximo passo seguro: migrar os tipos básicos de parâmetros de nó e `LayerEntry` para o domínio, mantendo adaptadores temporários na UI e sem alterar o comportamento do editor.

Em seguida, foi concluída a extração estrutural de `NodeParams`:

- criado `src/nodes/params.rs`;
- `NodeParams` e seu construtor de valores padrão foram retirados do módulo principal de nós;
- `src/nodes/mod.rs` passou a reexportar `NodeParams` para preservar as chamadas existentes;
- os módulos de cada nó continuam funcionando através do mesmo contrato público;
- nenhuma variante, campo ou valor padrão foi alterado;
- a definição antiga foi mantida apenas como bloco temporário de compatibilidade durante a migração e deve ser removida na limpeza da próxima etapa;
- nenhum formato de arquivo ou comportamento visual foi alterado.

Validação adicional:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

O próximo trabalho deve remover definitivamente o bloco legado e migrar `LayerEntry` para uma estrutura de domínio sem o campo visual `renomeando`. Esse campo deverá viver no estado da apresentação, não nos dados persistentes do projeto.

### Extração de `LayerEntry`

Também foi concluída a extração estrutural de `LayerEntry` para `src/nodes/layer_entry.rs`.

Nesta etapa:

- `LayerEntry` deixou de ficar definido em `src/nodes/mod.rs`;
- `src/nodes/mod.rs` passou a reexportar o tipo para preservar os imports atuais;
- criação padrão de layers, criação de layers pelo graph editor e persistência continuam usando o mesmo tipo público;
- nenhum campo persistido foi removido;
- o campo `renomeando` ainda permanece temporariamente no tipo para não alterar o fluxo atual da UI;
- a próxima subetapa deve mover `renomeando` para o estado de apresentação;
- a definição antiga duplicada foi removida.

Validação após a extração:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Separação do estado de renomeação

Foi concluída a primeira separação entre dados persistentes e estado visual da layer:

- removido `renomeando` de `LayerEntry`;
- removida a inicialização desse campo nos nós padrão e no graph editor;
- removida a reconstrução desse campo durante o carregamento JSON;
- o estado ativo de renomeação passou a ser controlado por `UserState.renaming_layer` e `GraphPanel.renaming_layer`;
- o inspector agora recebe o estado de apresentação explicitamente;
- o comportamento de duplo clique, Enter e Escape foi preservado;
- não há mais ocorrências de `renomeando` no código Rust;
- o formato JSON existente não foi alterado, pois esse campo já não era persistido.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

Com isso, `LayerEntry` representa somente dados do projeto. O próximo passo é aplicar o mesmo princípio aos demais parâmetros que ainda usam tipos de UI diretamente, começando pelas cores de `Texto`, `Shape` e `Pen`.

### Migração das cores de Texto, Shape e Pen

Foi concluída a migração das cores persistentes dos nós `Texto`, `Shape` e `Pen`:

- os campos `cor` e `cor_fill` agora usam `domain::Color`;
- os valores padrão dos nós foram convertidos para o tipo de domínio;
- o inspector passou a editar cores por meio de um adaptador explícito para `egui::Color32`;
- o preview converte cores de domínio somente ao montar dados de renderização;
- a DSL converte cores parseadas para o tipo de domínio antes de alterar os parâmetros;
- a persistência continua usando exatamente `[u8; 4]`, sem alteração de formato;
- arquivos `.lory` antigos continuam compatíveis;
- a camada de Pen DSL continua livre para usar seu tipo de cor de execução, sem contaminar os parâmetros persistentes;
- não foram adicionados recursos novos.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

Neste ponto, as cores persistentes de Canvas, Layer, Shape, Texto e Pen já possuem representação de domínio. A próxima etapa recomendada é separar os demais parâmetros de `NodeParams` em estruturas específicas (`CanvasParams`, `ShapeParams`, `TextParams`, `PenParams` etc.), preservando as variantes públicas durante a transição.

### Primeira variante de `NodeParams` separada: Shape

Foi concluída a primeira migração de variante estruturada:

- criado `src/nodes/shape_params.rs`;
- criada a estrutura `ShapeParams` com os dados persistentes do nó Shape;
- `NodeParams::Shape` passou a encapsular `ShapeParams`;
- defaults do nó Shape foram adaptados;
- inspector, DSL, preview, normalização de cena e persistência foram adaptados;
- o JSON continua com os mesmos campos e o mesmo formato;
- o comportamento visual e os valores padrão foram preservados;
- nenhum recurso novo foi adicionado.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Segunda variante de `NodeParams` separada: Texto

Foi concluída a migração da variante `Texto`:

- criado `src/nodes/text_params.rs`;
- criada a estrutura `TextParams` com os dados persistentes do nó Texto;
- `NodeParams::Texto` passou a encapsular `TextParams`;
- defaults do nó Texto foram adaptados;
- inspector, DSL, preview, normalização de cena e persistência foram adaptados;
- o JSON continua com os mesmos campos e o mesmo formato;
- o comportamento visual e os valores padrão foram preservados;
- nenhum recurso novo foi adicionado.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Terceira variante de `NodeParams` separada: Pen

Foi concluída a migração da variante `Pen`:

- criado `src/nodes/pen_params.rs`;
- criada a estrutura `PenParams` com os dados persistentes do nó Pen;
- `NodeParams::Pen` passou a encapsular `PenParams`;
- defaults do nó Pen foram adaptados;
- inspector, DSL, preview, normalização de cena e persistência foram adaptados;
- o JSON continua com os mesmos campos e o mesmo formato;
- o comportamento visual e os valores padrão foram preservados;
- nenhum recurso novo foi adicionado.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Variantes restantes: Transform, Cena, Layer, Ruído, Animação e Saída

Foi concluída a migração de todas as variantes restantes de `NodeParams` para estruturas específicas em arquivos próprios:

| Variante | Arquivo | Estrutura |
|---|---|---|
| Transform | `src/nodes/transform_params.rs` | `TransformParams` |
| Cena | `src/nodes/cena_params.rs` | `CenaParams` |
| Layer | `src/nodes/layer_params.rs` | `LayerParams` |
| Ruído | `src/nodes/ruido_params.rs` | `RuidoParams` |
| Animação | `src/nodes/anim_params.rs` | `AnimParams` |
| Saída | `src/nodes/saida_params.rs` | `SaidaParams` |

Nesta etapa:

- criados 6 arquivos de parâmetros;
- todas as variantes de `NodeParams` passaram a ser tuplas com struct específica;
- `NodeParams` agora não tem mais variantes com campos inline — todas encapsulam uma struct;
- defaults, inspector, DSL, preview, normalização de cena e persistência foram adaptados;
- o JSON continua com os mesmos campos e o mesmo formato;
- o comportamento visual e os valores padrão foram preservados;
- nenhum recurso novo foi adicionado.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Estado atual do `NodeParams`

Agora `NodeParams` possui exclusivamente variantes tuplas com structs específicas:

```rust
pub enum NodeParams {
    Transform(TransformParams),
    Cena(CenaParams),
    Layer(LayerParams),
    Texto(TextParams),
    Shape(ShapeParams),
    Pen(PenParams),
    Ruido(RuidoParams),
    Anim(AnimParams),
    Saida(SaidaParams),
    Canvas(ProjetoConfig),
}
```

### Migração de AnimSeg, Easing e LoopMode para o domínio

Foi concluída a migração dos tipos de animação para o domínio:

- criado `src/domain/animation.rs`;
- criados os tipos `Easing`, `LoopMode` e `AnimSeg` no domínio;
- `src/procedural.rs` passou a reexportar esses tipos do domínio para compatibilidade;
- `anim_params.rs` agora usa `crate::domain::AnimSeg` em vez de `crate::procedural::AnimSeg`;
- todos os consumidores foram atualizados para referenciar os tipos do domínio;
- nenhum formato de arquivo ou comportamento foi alterado.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Remoção da dependência egui de TipoNo e migração para o domínio

Foi concluída a remoção da dependência egui de `TipoNo` e sua migração para o domínio:

- criado `src/domain/node_type.rs` com `TipoNo` sem o método `cor()`;
- `cor()` foi extraído para `graph_editor::types::cor_tipo_no()` como adaptador UI;
- `src/nodes/mod.rs` passou a reexportar `TipoNo` do domínio;
- os 5 pontos de uso de `tipo.cor()` foram atualizados para `cor_tipo_no(tipo)`;
- `TipoNo::pode_conectar()` permanece no domínio como método do enum;
- todos os consumidores continuam funcionando pelo mesmo nome público.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Migração de todos os param structs e NodeParams para o domínio

Foi concluída a migração de todos os parâmetros de nó para `src/domain/`:

- `LayerEntry` migrado de `egui::Color32` para `domain::Color` e movido para `src/domain/layer_entry.rs`;
- adicionado `Color::from_rgb()` em `src/domain/color.rs` para atender à paleta de layers;
- `node_component.rs` passou a converter `domain::Color` → `egui::Color32` no ponto de desenho;
- os 9 arquivos `*_params.rs` movidos de `src/nodes/` para `src/domain/`;
- `params.rs` (com `NodeParams`) movido para `src/domain/`, removido o método `padrao()`;
- `nodes::node_params_padrao()` criada como função livre na camada de nós (depende de egui);
- `src/nodes/mod.rs` reexporta todos os tipos do domínio para compatibilidade;
- removidos os arquivos originais de `src/nodes/`.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Fase 4 — Separar GraphPanel em módulos menores

Foi concluída a divisão do `GraphPanel` em módulos especializados:

- `node_factory.rs` — criação de nós (`criar_nos_padrao`, `adicionar_no_em`, `adicionar_no`);
- `layer_ops.rs` — operações de cenas e layers (`cenas_disponiveis`, `normalizar_cena`, `sync_layer_ports`, CRUD de layers);
- `layout.rs` — coordenadas, hit test e portas espaciais (6 métodos de consulta espacial);
- `search.rs` — busca textual de nós por nome/tipo;
- `mod.rs` caiu de **1106 → 535 linhas**, mantendo apenas o coordenador `show()`, conexões, queries básicas e estrutura `GraphPanel`.

Validação:

```text
cargo check       OK — sem warnings
cargo test --all  OK — 68 testes
```

### Fase 6 — Separar persistência em infrastructure/persistence

Foi concluída a separação da camada de persistência:

- `src/projeto_arquivo.rs` deletado, conteúdo redistribuído:
  - `src/infrastructure/persistence/format.rs` — tipos-espelho JSON e conversões `From`/`TryFrom` (~350 linhas);
  - `src/infrastructure/persistence/migrations.rs` — sistema de migração por versão (`VERSAO_ATUAL = 1`);
  - `src/infrastructure/persistence/repository.rs` — `load_project()`, `save_project()`, `load_from_str()` com `PersistenceError`;
  - `src/infrastructure/persistence/mod.rs` — reexporta apenas a API pública.
- `src/main.rs` declara `mod infrastructure`;
- `src/app.rs` usa o repositório ao invés de `ProjetoArquivo` diretamente.

Validação:

```text
cargo check       OK — sem warnings
cargo test --all  OK — 68 testes
```

### Fase 7 — Separar DSL de aplicação

Foi concluída a separação da avaliação DSL da camada visual:

- `src/graph_editor/dsl.rs` (641 linhas) deletado, lógica movida para `src/dsl/`;
- `src/dsl/application.rs` — trait `Application` + funções `aplicar_script`/`aplicar_patch`;
- `src/dsl/evaluator.rs` — re-exporta `aplicar_script` e `aplicar_patch`;
- `GraphPanel` implementa `Application` (`type NodeId = NodeId`);
- `app.rs` chama `crate::dsl::evaluator::aplicar_script(...)`.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

### Fase 8 — Separar avaliação procedural e renderização

Foi concluída a separação da lógica procedural do renderer egui:

- `src/procedural.rs` deletado; substituído por `src/procedural/mod.rs`;
- `src/procedural/domain.rs` — lógica pura de avaliação (sem egui): `ShapeGenerator`, `PenPath`, `TextoItem`, `PreviewData`, `CenaPreview`, `LayerPreview`, `Shape`, `AnimDriver`, `RuidoDriver`; funções `generate()`, `trim_path_pts()`, `fbm()`, `ruido_offset()`; `Shape` usa campos próprios (não egui); `Pos2`/`Vec2` = `glam::Vec2`;
- `src/procedural/render.rs` — adaptador domain → egui: `shape_to_egui()`, `generate_shape_egui()`, `color_to_color32()`;
- `src/graph_editor/preview.rs` usa `shape_to_egui()` antes de `aplicar_opacidade()`/`translate_shape()`;
- `src/ui/preview.rs` converte cores via `color_to_color32()`; usa `domain::Vec2` para `trim_path_pts()`; `GVec2` migrado para `domain::Vec2`;
- `src/export.rs` usa `shape_to_egui()` + `traduzir()` e `color_to_color32()` para todas as cores de domínio;
- `src/dsl/pen.rs`: `GVec2` → `crate::domain::Vec2` (~15 ocorrências);
- `procedural/mod.rs` removeu re-export `GVec2` inexistente.

Validação:

```text
cargo check       OK (warnings only)
cargo test --all  OK — 68 testes
```

---

## 3. Regras obrigatórias da refatoração

### Criação do `domain::Project` como fonte de verdade (Fase 3)

Foi concluída a criação do `Project` no domínio como representação pura do projeto:

- criado `src/domain/project.rs` com `ProjectNode`, `ProjectEdge` e `Project`;
- `GraphPanel::to_project()` extrai o estado do grafo para `Project`;
- `GraphPanel::load_project()` reconstrói o grafo a partir de `Project`;
- `ProjetoArquivo::from_project()` e `to_project()` substituem `from_graph()`/`to_graph()`;
- `app.rs` (`salvar_projeto`, `carregar_projeto`, `carregar_arquivo`) atualizado para usar a nova API;
- removidos os métodos antigos `from_graph()` e `to_graph()` de `ProjetoArquivo`;
- undo/redo interno continua usando snapshots internos (`carregar_snapshot`);
- nenhum formato de arquivo ou comportamento visual foi alterado.

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

Neste ponto, todos os passos da **Fase 2** (domínio independente) e parte da **Fase 3** (Project como fonte de verdade) estão concluídos.

### Separação do GraphPanel em módulos menores (Fase 4)

Foi concluída a divisão do `GraphPanel` em módulos especializados:

- `node_factory.rs` — criação de nós (`criar_nos_padrao`, `adicionar_no_em`, `adicionar_no`);
- `layer_ops.rs` — operações de cenas e layers (`cenas_disponiveis`, `normalizar_cena`, `sync_layer_ports`, CRUD de layers);
- `layout.rs` — coordenadas, hit test e portas espaciais (6 métodos de consulta espacial);
- `search.rs` — busca textual de nós por nome/tipo;
- `mod.rs` caiu de **1106 → 535 linhas**, mantendo apenas o coordenador `show()`, conexões, queries básicas e estrutura `GraphPanel`.

Validação:

```text
cargo check       OK — sem warnings
cargo test --all  OK — 68 testes
```

### Separação da persistência em infrastructure/persistence (Fase 6)

Foi concluída a separação da camada de persistência:

- `src/projeto_arquivo.rs` deletado, conteúdo redistribuído:
  - `src/infrastructure/persistence/format.rs` — tipos-espelho JSON (`AnimSegJson`, `NodeParamsJson`, `NoJson`, `ArestaJson`, `ProjetoArquivo`) e conversões `From`/`TryFrom` com o domínio (~350 linhas);
  - `src/infrastructure/persistence/migrations.rs` — sistema de migração por versão (`VERSAO_ATUAL = 1`, `migrate()` sequencial);
  - `src/infrastructure/persistence/repository.rs` — `load_project()`, `save_project()`, `load_from_str()` com `PersistenceError` unificado (I/O, parse, validação);
  - `src/infrastructure/persistence/mod.rs` — reexporta apenas a API pública (`load_project`, `save_project`, `load_from_str`, `PersistenceError`).
- `src/main.rs` declara `mod infrastructure` (removeu `mod projeto_arquivo`);
- `src/app.rs` substituiu uso direto de `ProjetoArquivo` e I/O inline por chamadas ao repositório (`load_from_str`, `load_project`, `save_project`);
- `carregar_arquivo()` simplificado: recebe `&Project` em vez de `ProjetoArquivo`.

Validação:

```text
cargo check       OK — sem warnings
cargo test --all  OK — 68 testes
```

---

### Separação da DSL de aplicação (Fase 7)

Foi concluída a separação da avaliação DSL da camada visual:

- `src/graph_editor/dsl.rs` (641 linhas) deletado, lógica movida para `src/dsl/`:
  - `src/dsl/application.rs` — trait `Application` com associated type `NodeId` que define a interface mínima (criar/remover nós, queries, conexões, histórico, estado DSL, layers, config, utilitários de portos);
  - `src/dsl/evaluator.rs` — re-exporta `aplicar_script`, `aplicar_patch`, `Application`, `ScriptError`;
  - `src/dsl/application.rs` contém as funções genéricas `aplicar_script<A: Application>` e `aplicar_patch<A: Application>` + helpers (`aplicar_campos`, `merge_layers`, `conectar_edge`, etc.) que operam apenas via trait.
- `GraphPanel` implementa `Application` (com `type NodeId = NodeId`) expondo métodos `proxima_pos_livre`, `remover_aresta_entre` e delegando às implementações internas.
- `src/app.rs` atualizado para chamar `crate::dsl::evaluator::aplicar_script(&mut self.graph, &self.script_text)` em vez de `self.graph.aplicar_script()`.
- A DSL agora pode ser testada sem UI (mock implementando `Application`), o erro de aplicação entra no undo/redo (`empurrar_historico` chamado antes de mutar mutar mutações), e falha não gera estado parcial (validação completa antes de aplicar).

Validação:

```text
cargo check       OK
cargo test --all  OK — 68 testes
```

---

## 3. Regras obrigatórias da refatoração

Estas regras devem ser respeitadas em todas as etapas.

### 3.1 Não reescrever tudo de uma vez

Cada mudança deve ser pequena, compilável e revisável.

O fluxo padrão será:

```text
medir estado atual
→ criar teste de proteção
→ extrair uma responsabilidade
→ adaptar chamadas antigas
→ executar format/check/test
→ revisar diff
→ somente então iniciar a próxima etapa
```

### 3.2 Não misturar mudança estrutural com mudança de comportamento

Uma alteração que apenas move código não deve simultaneamente:

- mudar o formato `.lory`;
- alterar o comportamento da timeline;
- trocar regras de conexão;
- modificar o resultado do preview;
- adicionar recursos novos.

Se houver um bug descoberto durante a movimentação, registrar em uma tarefa separada e só corrigir junto se for indispensável para manter a compilação.

### 3.3 O projeto deve permanecer executável

Ao final de cada etapa:

```bash
cargo fmt --check
cargo check
cargo test --all
```

Para etapas de renderização e interface, também executar manualmente uma lista de verificação visual.

### 3.4 Não alterar o formato do projeto sem migração

Arquivos `.lory` existentes são parte do contrato do produto.

Nunca fazer uma alteração como:

```rust
versao: 1 → versao: 2
```

sem:

- definir o novo formato;
- criar leitura compatível;
- criar migração explícita;
- testar arquivos antigos;
- manter mensagens de erro compreensíveis.

### 3.5 O modelo deve ser a fonte de verdade

O editor visual não deve ser a fonte principal dos dados do projeto.

O estado correto deve ser:

```text
Projeto do domínio = fonte de verdade
Grafo egui         = representação visual
Preview            = resultado calculado do projeto
JSON               = persistência do projeto
DSL                = entrada de comandos para o projeto
```

Isso é uma mudança arquitetural essencial para reduzir problemas de salvar, carregar, undo, redo, preview e DSL.

### 3.6 Nenhuma camada inferior pode depender da UI

Dependências permitidas:

```text
presentation → application → domain
infrastructure → domain
```

Dependências que devem ser evitadas:

```text
domain → egui
domain → ui
DSL → GraphPanel
persistência → GraphPanel
renderizador → estado interno da UI
```

---

## 4. Arquitetura-alvo

A arquitetura final deve ser organizada em quatro grandes camadas.

```text
┌──────────────────────────────────────────────────────────┐
│ Presentation                                             │
│ janelas, menus, egui, graph view, timeline, inspector    │
└───────────────────────────────┬──────────────────────────┘
                                │ comandos/eventos
┌───────────────────────────────▼──────────────────────────┐
│ Application                                               │
│ casos de uso, coordenação, histórico, playback, preview   │
└───────────────────────────────┬──────────────────────────┘
                                │ opera sobre
┌───────────────────────────────▼──────────────────────────┐
│ Domain                                                   │
│ projeto, nós, conexões, animação, geometria, regras      │
└───────────────┬───────────────────────────┬──────────────┘
                │                           │
┌───────────────▼──────────────┐ ┌──────────▼──────────────┐
│ Infrastructure                │ │ Render/Adapters         │
│ JSON, DSL, arquivos, export   │ │ egui, texto, PNG        │
└──────────────────────────────┘ └─────────────────────────┘
```

### 4.1 Estrutura final sugerida

```text
src/
├── main.rs
│
├── domain/
│   ├── mod.rs
│   ├── project.rs
│   ├── project_config.rs
│   ├── node.rs
│   ├── node_type.rs
│   ├── node_params.rs
│   ├── ports.rs
│   ├── connection.rs
│   ├── group.rs
│   ├── layer.rs
│   ├── animation.rs
│   ├── geometry.rs
│   ├── color.rs
│   ├── identifiers.rs
│   └── errors.rs
│
├── application/
│   ├── mod.rs
│   ├── project_service.rs
│   ├── graph_service.rs
│   ├── history.rs
│   ├── command.rs
│   ├── command_history.rs
│   ├── playback.rs
│   ├── preview_service.rs
│   └── state.rs
│
├── infrastructure/
│   ├── mod.rs
│   ├── persistence/
│   │   ├── mod.rs
│   │   ├── project_file.rs
│   │   ├── project_json.rs
│   │   └── migrations.rs
│   ├── dsl/
│   │   ├── mod.rs
│   │   ├── pen/
│   │   ├── project.rs
│   │   ├── patch.rs
│   │   └── validation.rs
│   ├── export/
│   │   ├── mod.rs
│   │   ├── png.rs
│   │   └── render.rs
│   └── logging.rs
│
├── presentation/
│   ├── mod.rs
│   ├── app.rs
│   ├── app_state.rs
│   ├── commands.rs
│   ├── project_window.rs
│   ├── script_window.rs
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── graph_view.rs
│   │   ├── graph_adapter.rs
│   │   ├── graph_interaction.rs
│   │   ├── graph_rendering.rs
│   │   ├── selection.rs
│   │   └── groups.rs
│   ├── inspector/
│   │   ├── mod.rs
│   │   ├── common.rs
│   │   ├── canvas.rs
│   │   ├── scene.rs
│   │   ├── layer.rs
│   │   ├── shape.rs
│   │   ├── text.rs
│   │   ├── pen.rs
│   │   ├── noise.rs
│   │   ├── animation.rs
│   │   └── transform.rs
│   ├── preview/
│   │   ├── mod.rs
│   │   ├── preview_panel.rs
│   │   ├── egui_renderer.rs
│   │   ├── pen_renderer.rs
│   │   └── text_renderer.rs
│   ├── timeline/
│   │   ├── mod.rs
│   │   ├── timeline_panel.rs
│   │   ├── timeline_state.rs
│   │   └── markers.rs
│   ├── toolbar/
│   └── theme.rs
│
└── shared/
    ├── mod.rs
    ├── result.rs
    └── validation.rs
```

Os nomes são uma sugestão. O ponto importante é a separação de responsabilidades, não a quantidade de pastas.

---

## 5. Contratos centrais

### 5.1 Projeto

O domínio deve possuir uma estrutura que represente o projeto inteiro sem conhecer `egui`.

```rust
pub struct Project {
    pub config: ProjectConfig,
    pub nodes: NodeStore,
    pub connections: ConnectionStore,
    pub groups: Vec<Group>,
    pub timeline: TimelineData,
}
```

O projeto deve ser capaz de:

- adicionar nó;
- remover nó;
- alterar parâmetros;
- conectar portas;
- desconectar portas;
- validar conexões;
- obter nós por ID;
- verificar invariantes;
- produzir uma representação para preview;
- ser salvo e carregado.

### 5.2 IDs

Não usar índices de vetor como identidade permanente.

O formato atual pode continuar sendo lido por compatibilidade, mas internamente o ideal é usar IDs estáveis:

```rust
pub struct NodeId(u64);
pub struct ConnectionId(u64);
```

Os índices podem mudar após remoções; os IDs não devem mudar.

### 5.3 Parâmetros dos nós

`NodeParams` pode continuar como enum durante a primeira parte da refatoração, desde que deixe de depender de `egui`.

Uma versão inicial aceitável:

```rust
pub enum NodeParams {
    Canvas(CanvasParams),
    Scene(SceneParams),
    Layer(LayerParams),
    Shape(ShapeParams),
    Text(TextParams),
    Pen(PenParams),
    Noise(NoiseParams),
    Animation(AnimationParams),
    Transform(TransformParams),
    Output(OutputParams),
}
```

Cada estrutura específica deve ficar em arquivo próprio.

### 5.4 Portas

As portas devem ter uma descrição declarativa única:

```rust
pub struct PortDefinition {
    pub name: PortName,
    pub direction: PortDirection,
    pub data_type: DataType,
    pub multiple: bool,
}
```

Essa definição deve ser utilizada por:

- criação visual das portas;
- validação de conexão;
- DSL;
- exportação;
- testes.

Não deve haver uma regra de portas na UI e outra regra na DSL.

### 5.5 Comandos

As ações do usuário devem ser representadas por comandos ou operações explícitas:

```rust
pub enum ProjectCommand {
    AddNode(AddNodeCommand),
    RemoveNode(RemoveNodeCommand),
    SetNodeParams(SetNodeParamsCommand),
    Connect(ConnectCommand),
    Disconnect(DisconnectCommand),
    MoveNode(MoveNodeCommand),
    UpdateLayer(UpdateLayerCommand),
}
```

O histórico deve registrar comandos ou snapshots do modelo, não estado visual do `egui_graph_edit`.

---

## 6. Plano de execução por fases

## Fase 0 — Proteção e inventário

### Objetivo

Criar uma base segura antes de mover qualquer código.

### Tarefas

- preservar alterações locais existentes;
- registrar o resultado de `cargo check`;
- registrar o resultado de `cargo test --all`;
- catalogar arquivos `.lory`, `.json`, exemplos de DSL e imagens usadas no teste;
- documentar os fluxos manuais atuais;
- revisar o `.gitignore`;
- retirar logs de debug do controle de versão ou movê-los para uma pasta de diagnóstico.

### Fluxos manuais mínimos

1. abrir a aplicação;
2. criar um projeto novo;
3. adicionar cada tipo de nó;
4. conectar nós válidos;
5. tentar conexões inválidas;
6. mover nós;
7. selecionar e remover nós;
8. duplicar e colar;
9. agrupar e desagrupar;
10. editar layers;
11. alterar parâmetros no inspector;
12. reproduzir a timeline;
13. pausar e avançar frames;
14. alterar marcadores;
15. abrir o editor DSL;
16. aplicar um projeto DSL;
17. aplicar patch DSL;
18. salvar o projeto;
19. fechar e carregar o projeto;
20. validar o preview;
21. executar exportação PNG.

### Critério de conclusão

- nenhum comportamento atual perdido;
- testes existentes passando;
- lista de arquivos de referência criada;
- estado inicial documentado.

---

## Fase 1 — Padronização de qualidade

### Objetivo

Estabelecer uma base de código consistente sem alterar arquitetura ainda.

### Tarefas

- executar `cargo fmt`;
- corrigir warnings legítimos;
- remover `allow(dead_code)` onde possível;
- padronizar nomes;
- remover comentários obsoletos;
- corrigir caracteres corrompidos na documentação e mensagens;
- eliminar código morto comprovadamente não utilizado;
- separar constantes globais de lógica;
- criar um módulo de erros comum;
- definir convenção de retorno `Result` e `Option`;
- substituir `unwrap()` em caminhos de produção por erros tratados.

### Política para `unwrap`

Pode permanecer em testes quando a falha indicar erro do próprio teste.

Em código de produção, preferir:

```rust
let value = map
    .get(&id)
    .ok_or(AppError::NodeNotFound(id))?;
```

Os pontos de parsing também devem retornar erros com linha, coluna e contexto.

### Critério de conclusão

- `cargo fmt --check` passa;
- warnings importantes eliminados;
- nenhuma mudança visual intencional;
- testes passando.

---

## Fase 2 — Criar o domínio independente da UI

### Objetivo

Remover a dependência direta de `egui` dos dados centrais.

### Tarefas

Criar tipos próprios para:

- cor;
- ponto;
- vetor;
- retângulo;
- tamanho;
- IDs;
- parâmetros de projeto;
- parâmetros de nós;
- animação;
- conexões;
- grupos.

Criar conversões temporárias:

```rust
impl From<DomainColor> for egui::Color32 { ... }
impl From<egui::Color32> for DomainColor { ... }
```

Essas conversões devem ficar fora do domínio, em adaptadores da apresentação.

### Estratégia de migração

Não trocar todos os usos de uma vez.

Ordem segura:

1. criar os novos tipos;
2. adicionar conversões;
3. escrever testes dos novos tipos;
4. migrar `ProjectConfig`;
5. migrar `LayerEntry`;
6. migrar animação;
7. migrar parâmetros dos nós;
8. remover tipos antigos somente quando não houver mais uso.

### Critério de conclusão

- o domínio compila sem importar `egui`;
- testes de matemática e animação funcionam sem iniciar a UI;
- preview e UI continuam funcionando por meio de conversões.

---

## Fase 3 — Criar o `Project` como fonte de verdade

### Objetivo

Separar o projeto lógico da representação visual do grafo.

### Tarefas

- criar `Project`;
- criar armazenamento de nós;
- criar armazenamento de conexões;
- criar IDs estáveis;
- implementar validação de invariantes;
- implementar operações de domínio;
- adaptar o `GraphPanel` para ler e escrever no `Project`;
- manter o `egui_graph_edit` apenas como view/adaptador.

### Invariantes do projeto

O método `validate()` deve verificar:

- todos os IDs são únicos;
- toda conexão referencia nós existentes;
- toda porta referenciada existe;
- tipos de dados são compatíveis;
- nós obrigatórios existem;
- não há conexão impossível;
- não há índice inválido;
- parâmetros numéricos não estão em estados impossíveis;
- cenas e layers referenciados existem ou possuem política explícita de fallback.

### Critério de conclusão

- salvar, carregar, DSL, undo/redo e preview usam o `Project`;
- o grafo visual pode ser reconstruído a partir do `Project`;
- o projeto continua visualmente equivalente ao comportamento anterior.

---

## Fase 4 — Dividir `GraphPanel`

### Status: CONCLUÍDA ✅

Foi concluída a divisão em módulos especializados, preservando o mesmo contrato público.

Módulos criados:

| Responsabilidade | Arquivo |
|---|---|
| Criação de nós | `node_factory.rs` |
| Operações de layers/cenas | `layer_ops.rs` |
| Layout e coordenadas espaciais | `layout.rs` |
| Busca textual | `search.rs` |
| Coordenador + conexões | `mod.rs` (~535 linhas) |

O `mod.rs` manteve apenas o coordenador `show()`, conexões entre nós e queries básicas. Todos os testes passam inalterados.

---

## Fase 5 — Refatorar undo/redo

### Objetivo

Garantir que toda alteração relevante seja desfazível e previsível.

### Tarefas

- criar `History<T>` genérico;
- fazer o histórico operar sobre `Project`;
- definir quando um comando cria entrada no histórico;
- impedir snapshots duplicados para pequenas repinturas;
- limpar redo somente após nova alteração real;
- preservar histórico ao alternar painéis;
- testar undo/redo após salvar e carregar;
- testar undo/redo após aplicação de DSL;
- testar undo/redo em grupos e conexões.

### Regra de transação

Operações complexas devem ser transacionais:

```text
validar tudo
→ criar cópia/estado temporário
→ aplicar operação
→ validar resultado
→ confirmar ou descartar
```

Se a DSL falhar no meio, o projeto não pode ficar parcialmente alterado.

---

## Fase 6 — Separar persistência e migrações

### Status: CONCLUÍDA ✅

Foi concluída a separação da camada de persistência.

### O que foi feito

- `src/projeto_arquivo.rs` deletado; conteúdo redistribuído:
  - `infrastructure/persistence/format.rs` — tipos-espelho JSON e conversões `From`/`TryFrom`;
  - `infrastructure/persistence/migrations.rs` — sistema de migração por versão (`VERSAO_ATUAL = 1`);
  - `infrastructure/persistence/repository.rs` — `load_project()`, `save_project()`, `load_from_str()` com `PersistenceError`;
  - `infrastructure/persistence/mod.rs` — reexporta apenas a API pública;
- `src/main.rs` declara `mod infrastructure`;
- `src/app.rs` usa o repositório ao invés de `ProjetoArquivo` diretamente.

### Fluxo seguro de carregar

```text
abrir arquivo
→ ler bytes
→ desserializar
→ migrar versão
→ validar formato
→ converter para domínio
→ validar domínio
→ substituir projeto atual
```

Nunca substituir o projeto atual antes da última validação.

### Critério de conclusão

- arquivos antigos continuam carregando;
- arquivos inválidos não quebram o estado atual;
- salvar/carregar possui testes automatizados;
- mensagens de erro são úteis para o usuário.

---

## Fase 7 — Separar DSL de aplicação

### Status: CONCLUÍDA ✅

Foi concluída a separação da avaliação DSL da camada visual.

### O que foi feito

- `src/graph_editor/dsl.rs` (641 linhas) deletado; lógica movida para `src/dsl/`:
  - `src/dsl/application.rs` — trait `Application` com `aplicar_script`/`aplicar_patch`;
  - `src/dsl/evaluator.rs` — re-exports das funções genéricas;
- `GraphPanel` implementa `Application` (`type NodeId = NodeId`);
- `src/app.rs` chama `crate::dsl::evaluator::aplicar_script(...)`.

### Fluxo final

```text
String
  ↓
Lexer/tokenizer
  ↓
AST
  ↓
Validador sem efeitos colaterais
  ↓
Comandos
  ↓
Aplicador transacional
  ↓
Project
```

### Pen DSL

Separar conceitualmente:

```text
pen/
├── lexer.rs
├── parser.rs
├── ast.rs
├── evaluator.rs
├── environment.rs
├── functions.rs
├── errors.rs
└── tests/
```

O parser e avaliador da Pen DSL já possuem muitos testes. A prioridade é preservar o comportamento, não alterar a linguagem.

### Project DSL

Separar:

- AST;
- parser;
- resolução de IDs;
- resolução de portas;
- validação;
- conversão para comandos;
- aplicação no projeto.

### Patch DSL

O patch deve ser transacional:

1. parsear todos os comandos;
2. validar todos os IDs;
3. validar todas as portas;
4. validar conflitos;
5. aplicar em cópia temporária;
6. validar resultado;
7. confirmar uma única alteração no histórico.

### Critério de conclusão

- DSL pode ser testada sem UI;
- erros têm linha e contexto;
- falha não gera estado parcial;
- aplicar DSL entra corretamente no undo/redo.

---

## Fase 8 — Separar avaliação procedural e renderização

### Status: CONCLUÍDA ✅

Foi concluída a separação da lógica procedural do renderer egui. Todos os objetivos foram atingidos.

### O que foi feito

- `src/procedural.rs` deletado; substituído por `src/procedural/mod.rs`;
- `src/procedural/domain.rs` criado com lógica pura de avaliação (sem egui):
  - tipos: `ShapeGenerator`, `PenPath`, `TextoItem`, `PreviewData`, `CenaPreview`, `LayerPreview`, `Shape`, `AnimDriver`, `RuidoDriver`;
  - funções: `generate()`, `trim_path_pts()`, `fbm()`, `ruido_offset()`;
  - `Shape` usa campos próprios (`Rect`, `Ellipse`, `Path` com coordenadas puras);
  - `Pos2` e `Vec2` são `glam::Vec2` (tipo de domínio, sem egui);
  - `trim_path_pts` opera em `&[Pos2]` (glam::Vec2) — tipo puro de domínio;
- `src/procedural/render.rs` criado como adaptador domain → egui:
  - `shape_to_egui()` converte `domain::Shape` → `egui::Shape` (precisa de conversão explícita: `Pos2::new(c.x, c.y)`, `Vec2::new(tam.x, tam.y)`);
  - `generate_shape_egui()` avalia + converte em um passo;
  - `color_to_color32()` converte `domain::Color` → `Color32`;
- Consumentes atualizados:
  - `src/graph_editor/preview.rs` usa `procedural::render::shape_to_egui()` antes de `aplicar_opacidade()`/`translate_shape()`;
  - `src/ui/preview.rs` converte cores via `color_to_color32()`; usa `domain::Vec2` para pontos do `trim_path_pts()`; `GVec2` substituído por `domain::Vec2`;
  - `src/export.rs` usa `shape_to_egui()` + `traduzir()` e `color_to_color32()` para todas as cores de domínio;
  - `src/dsl/pen.rs`: `GVec2` → `crate::domain::Vec2` (~15 ocorrências);
- `procedural/mod.rs` removeu re-export `GVec2` inexistente;
- `dsl/evaluator.rs` mantém re-exports de `aplicar_script` e `aplicar_patch`.

### Validação

```text
cargo check       OK (warnings only)
cargo test --all  OK — 68 testes
```

---

## Fase 9 — Dividir o inspector

### Objetivo

Tornar cada editor de nó independente.

### Estrutura

```text
inspector/
├── mod.rs
├── common.rs
├── canvas.rs
├── scene.rs
├── layer.rs
├── shape.rs
├── text.rs
├── pen.rs
├── noise.rs
├── animation.rs
├── transform.rs
└── output.rs
```

### Regra de cada editor

Cada editor deve:

- receber somente os dados necessários;
- desenhar controles;
- retornar uma ação explícita;
- não conhecer o estado completo do aplicativo;
- não salvar diretamente;
- não controlar undo/redo;
- não reconstruir o grafo.

Exemplo:

```rust
pub enum InspectorAction {
    None,
    Changed,
    RenameLayer { index: usize, name: String },
    ApplyPenCode(String),
}
```

O coordenador recebe a ação e decide como aplicá-la.

---

## Fase 10 — Refatorar `MovimentoApp`

### Objetivo

Deixar a aplicação principal responsável apenas pela composição da interface.

### Estado recomendado

```rust
pub struct AppState {
    pub project: Project,
    pub selection: SelectionState,
    pub playback: PlaybackState,
    pub windows: WindowState,
    pub notifications: NotificationState,
}
```

### Separar de `app.rs`

- inicialização;
- comandos de menu;
- playback;
- janela DSL;
- ações de arquivo;
- layout dos painéis;
- métricas de performance;
- notificações.

### Playback

O playback deve possuir uma unidade própria:

```rust
pub struct PlaybackState {
    pub playing: bool,
    pub current_frame: u32,
    pub fps: f32,
    pub loop_range: Option<FrameRange>,
}
```

O `App` apenas coleta o tempo do `egui` e chama o serviço de playback.

### Critério de conclusão

O `App` não deve conter regras específicas de nós, parsing de DSL ou serialização.

---

## Fase 11 — Padronizar erros, logs e diagnósticos

### Objetivo

Substituir falhas silenciosas e mensagens inconsistentes.

### Erros recomendados

```rust
pub enum AppError {
    Io(std::io::Error),
    InvalidProject(String),
    InvalidNode(String),
    InvalidConnection(String),
    Parse(ParseError),
    Evaluation(String),
    Export(String),
}
```

Usar `thiserror` se for adequado ao projeto.

### Regras de log

- não usar `eprintln!` espalhado;
- centralizar logs;
- distinguir erro, aviso, informação e diagnóstico;
- não registrar conteúdo sensível sem necessidade;
- não deixar arquivos de debug no diretório raiz;
- permitir desligar logs verbosos;
- métricas de performance devem ser opcionais.

---

## Fase 12 — Testes de regressão

### Objetivo

Garantir que a refatoração não alterou os recursos existentes.

### Testes de domínio

- criação de projeto;
- valores padrão;
- validação de parâmetros;
- regras de conexão;
- IDs;
- layers;
- cenas;
- animação;
- easing;
- ruído;
- trim;
- geometria.

### Testes de aplicação

- adicionar/remover nó;
- conectar/desconectar;
- undo/redo;
- agrupamento;
- seleção;
- alteração de parâmetros;
- aplicação de comandos;
- transações com falha.

### Testes de persistência

- salvar projeto padrão;
- carregar projeto salvo;
- carregar versão antiga;
- migrar versões;
- arquivo incompleto;
- tipo de nó desconhecido;
- conexão inválida;
- índice inválido;
- projeto corrompido;
- falha de gravação sem perda do arquivo anterior.

### Testes de DSL

- parser de projeto;
- parser de patch;
- Pen DSL;
- mensagens de linha e coluna;
- porta inexistente;
- ID duplicado;
- conexão inválida;
- falha transacional;
- aplicação seguida de undo.

### Testes de renderização

- mesmo projeto gera o mesmo preview;
- tempo zero;
- tempo no meio da animação;
- fim da animação;
- loop;
- ping-pong;
- texto;
- shape;
- Pen;
- exportação PNG.

### Testes de interface

O egui pode permanecer com testes manuais para:

- layout;
- interação do mouse;
- atalhos;
- redimensionamento;
- foco de campos;
- comportamento visual.

---

## 7. Ordem exata recomendada

Para reduzir risco, executar nesta ordem:

```text
1. proteger estado atual
2. padronizar formatação e warnings
3. criar erros comuns
4. criar tipos de domínio independentes
5. criar Project
6. colocar validação de invariantes
7. adaptar GraphPanel ao Project
8. extrair histórico
9. separar persistência
10. separar DSL de aplicação
11. separar avaliação procedural
12. separar renderização egui
13. dividir inspector
14. dividir MovimentoApp
15. reforçar testes de regressão
16. limpar compatibilidade antiga
17. documentar arquitetura final
```

Não inverter essa ordem. Separar UI antes de existir um modelo independente tende a apenas espalhar o acoplamento em mais arquivos.

---

## 8. Estratégia de compatibilidade durante a migração

Durante a refatoração, podem existir adaptadores temporários:

```rust
pub struct LegacyGraphAdapter<'a> {
    pub project: &'a mut Project,
    pub editor: &'a mut LegacyGraphPanel,
}
```

O adaptador serve para migrar gradualmente.

Não criar uma segunda fonte de verdade permanente. Enquanto o adaptador existir, deve haver uma tarefa clara para removê-lo.

Cada adaptador deve ter:

- comentário explicando por que existe;
- teste de comportamento;
- responsável pela remoção;
- critério de eliminação.

---

## 9. Critérios profissionais de qualidade

O código pode ser considerado profissional quando:

- cada módulo possui uma responsabilidade clara;
- o domínio compila sem UI;
- o projeto pode ser validado sem abrir janela;
- o grafo visual é reconstruível a partir do projeto;
- salvar e carregar não dependem da UI;
- DSL não modifica estado parcialmente;
- undo/redo funciona sobre operações do projeto;
- preview e exportação compartilham avaliação;
- erros são tratados e informativos;
- arquivos antigos continuam abrindo;
- testes cobrem o comportamento crítico;
- arquivos principais não são monólitos;
- não há dependências circulares conceituais;
- o fluxo de mudança é previsível para novos desenvolvedores.

---

## 10. Métricas de acompanhamento

As métricas servem para acompanhar qualidade, não para impor números artificiais.

Indicadores úteis:

| Métrica | Situação desejada |
|---|---|
| Linhas do `app.rs` | Reduzir significativamente |
| Linhas do `graph_editor/mod.rs` | Reduzir para coordenador |
| Dependências de `domain` em `egui` | Zero |
| `unwrap()` em produção | Somente casos justificados |
| `allow(dead_code)` | Somente casos documentados |
| Testes de persistência | Cobrir versões e falhas |
| Testes de DSL | Parser, validação e aplicação |
| Operações sem undo/redo | Nenhuma alteração de projeto |
| Caminhos de erro silenciosos | Eliminar |
| Código duplicado de portas | Uma única definição |

---

## 11. Checklist de cada pull request ou etapa

### Antes de começar

- [ ] O comportamento atual foi entendido?
- [ ] Existe teste de proteção?
- [ ] A etapa tem escopo único?
- [ ] Arquivos locais modificados foram preservados?

### Durante a implementação

- [ ] Não foi criado recurso novo?
- [ ] Não houve mudança acidental no formato `.lory`?
- [ ] A nova camada tem dependências corretas?
- [ ] A alteração não duplicou a fonte de verdade?
- [ ] Os erros continuam sendo tratados?

### Antes de concluir

- [ ] `cargo fmt --check` passa?
- [ ] `cargo check` passa?
- [ ] `cargo test --all` passa?
- [ ] O diff foi revisado?
- [ ] Projetos de teste continuam abrindo?
- [ ] DSL de teste continua funcionando?
- [ ] Preview foi comparado visualmente?
- [ ] Undo/redo foi testado?
- [ ] O documento de arquitetura foi atualizado?

---

## 12. O que não faz parte desta refatoração

Para manter o escopo controlado, não fazem parte deste plano:

- criar novos nós;
- criar novos efeitos;
- criar novos formatos de exportação;
- implementar plugins;
- criar keyframes novos;
- adicionar renderização de vídeo;
- trocar a biblioteca de UI;
- trocar a biblioteca de grafo sem necessidade;
- redesenhar a interface;
- mudar a linguagem DSL;
- alterar a experiência do usuário por preferência pessoal.

Se uma melhoria de recurso for descoberta, deve ser registrada separadamente.

---

## 13. Resultado final esperado

Ao término da refatoração, o Loryventoy deve possuir:

```text
Projeto independente da interface
        ↓
Serviços de aplicação previsíveis
        ↓
DSL, JSON e exportação isolados
        ↓
Graph editor como adaptador visual
        ↓
Preview usando o mesmo avaliador da exportação
        ↓
UI dividida em painéis especializados
```

O usuário final deve perceber o mesmo produto e os mesmos recursos, mas a equipe deve ganhar:

- menor risco ao alterar um módulo;
- menor chance de quebrar outro módulo;
- facilidade para testar;
- facilidade para corrigir bugs;
- melhor desempenho de manutenção;
- maior previsibilidade do projeto;
- base adequada para futuras evoluções.

---

## 14. Primeira execução recomendada

A primeira etapa prática deve ser pequena:

1. preservar as alterações locais;
2. criar uma branch de refatoração;
3. criar testes de carregamento de três projetos reais;
4. criar `domain::Color`, `domain::Point` e `domain::ProjectConfig`;
5. adicionar conversões temporárias para `egui`;
6. migrar apenas `ProjectConfig`;
7. executar `cargo fmt --check`, `cargo check` e `cargo test --all`;
8. revisar o diff;
9. somente então iniciar a migração de `NodeParams`.

Essa primeira execução deve alterar arquitetura interna sem mudar recursos nem aparência. É o ponto de partida mais seguro para toda a refatoração.
