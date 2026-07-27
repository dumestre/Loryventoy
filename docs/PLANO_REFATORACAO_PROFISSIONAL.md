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
cargo test --all  OK — 66 testes
```

Os principais pontos de concentração encontrados são:

| Arquivo | Situação atual | Risco |
|---|---|---|
| `src/app.rs` | Janela principal, menus, playback, projeto, DSL e layout | Muito alto |
| `src/graph_editor/mod.rs` | Modelo visual, nós, conexões, histórico, DSL e UI | Muito alto |
| `src/procedural.rs` | Animação, geometria, preview e tipos ligados ao egui | Alto |
| `src/ui/node_component.rs` | Inspector de vários tipos de nó em um arquivo | Alto |
| `src/ui/preview.rs` | Renderização, texto, Pen DSL e conversões visuais | Alto |
| `src/projeto_arquivo.rs` | JSON misturado com tipos de domínio e tipos visuais | Alto |
| `src/dsl/pen.rs` | Parser e avaliador grande, porém com boa cobertura de testes | Médio |
| `src/graph_editor/dsl.rs` | Parser aplicado diretamente sobre `GraphPanel` | Muito alto |

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

O próximo passo arquitetural é criar o `Project` como fonte de verdade (Fase 3 do plano geral).

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
- criar branch específica de refatoração;
- registrar o resultado de `cargo check`;
- registrar o resultado de `cargo test --all`;
- catalogar arquivos `.lory`, `.json`, exemplos de DSL e imagens usadas no teste;
- documentar os fluxos manuais atuais;
- conferir quais diretórios são aplicativos independentes;
- verificar se `lory-hub` deve continuar separado;
- verificar se `egui-graph-edit` é dependência local intencional;
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

### Objetivo

Reduzir [src/graph_editor/mod.rs](../src/graph_editor/mod.rs) a um coordenador de interface.

### Separações

| Responsabilidade | Novo módulo |
|---|---|
| Estado do grafo visual | `graph_view_state.rs` |
| Criação de nós | `node_factory.rs` |
| Conexões | `connections.rs` |
| Seleção | `selection.rs` |
| Grupos | `groups.rs` |
| Undo/redo | `history.rs` |
| Layout e posições | `layout.rs` |
| Busca | `search.rs` |
| Renderização | `rendering.rs` |
| Integração DSL | `dsl_adapter.rs` |
| Integração preview | `preview_adapter.rs` |
| Conversões visual/modelo | `adapter.rs` |

### O que deve permanecer no coordenador

Somente:

- referências aos componentes;
- chamadas de alto nível;
- encaminhamento de eventos;
- sincronização entre view e modelo;
- composição do painel.

### O que não deve permanecer nele

- regras detalhadas de conexão;
- interpretação da DSL;
- serialização JSON;
- geração de preview;
- lógica de cada tipo de nó;
- histórico implementado manualmente dentro do painel.

### Critério de conclusão

O arquivo principal do graph editor deve ser pequeno o suficiente para ser lido integralmente em uma revisão, idealmente entre 150 e 300 linhas.

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

### Objetivo

Tornar o formato `.lory` estável e seguro.

### Estrutura recomendada

```text
infrastructure/persistence/
├── project_file.rs
├── project_json.rs
├── migrations.rs
└── validation.rs
```

### Regras

- o domínio não conhece JSON;
- o JSON não conhece `GraphPanel`;
- IDs persistentes devem ser preservados;
- erros de arquivo devem informar caminho e causa;
- o carregamento deve validar tudo antes de alterar o projeto atual;
- falha no carregamento não pode destruir o projeto aberto;
- salvar deve ser atômico sempre que possível;
- criar arquivo temporário e substituir somente após gravação bem-sucedida;
- manter versão explícita;
- criar testes com arquivos válidos, incompletos e corrompidos.

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

### Objetivo

Fazer com que parser, validação e aplicação sejam partes independentes.

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

### Objetivo

Permitir que o projeto seja avaliado sem depender de `egui`.

### Camadas

```text
Project
  ↓
Graph evaluator
  ↓
Scene evaluation
  ↓
Render primitives
  ↓
Renderer específico
```

### Tipos de saída

O avaliador deve produzir tipos neutros, por exemplo:

```rust
pub enum RenderPrimitive {
    Path(PathPrimitive),
    Shape(ShapePrimitive),
    Text(TextPrimitive),
}
```

O renderer `egui` transforma esses dados em formas visuais.

### Separar `procedural.rs`

Dividir por responsabilidade:

```text
domain/animation.rs
domain/noise.rs
domain/geometry.rs
render/shape_generator.rs
render/preview_data.rs
render/scene_evaluator.rs
```

### Regras

- preview não modifica o projeto;
- avaliação deve ser determinística para o mesmo projeto, tempo e seed;
- cache deve ser explícito;
- cache deve ser invalidado quando um nó relevante muda;
- renderizador não deve buscar parâmetros diretamente em `GraphPanel`;
- exportação e preview devem usar a mesma avaliação.

### Critério de conclusão

- preview e exportação produzem resultados equivalentes;
- avaliação pode ser testada sem abrir janela;
- alterações na UI não alteram a matemática procedural.

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
