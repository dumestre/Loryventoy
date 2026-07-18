# Exemplos da DSL (Pen + Projeto)

Este arquivo é **documentação** da linguagem do Movimento. O código-fonte
dos exemplos que aparecem na aba **Exemplos** do editor DSL vive em
[`exemplos.md`](exemplos.md) — aquele arquivo é lido em tempo de compilação
pela função `carregar_exemplos()` e alimenta a UI. **Não duplique o código
aqui**: para ver/copiar os exemplos prontos, abra `exemplos.md` ou a própria
aba Exemplos no app.

Abaixo, apenas explicamos a sintaxe e damos dicas de como usar cada exemplo
que está em `exemplos.md`.

---

## Como abrir as janelas (Passo a passo)

### Abrir o editor de código do nó Pen ("Código DSL")

1. No **editor de grafos**, clique no botão **Pen** (roxo) na **barra de
   ferramentas** para criar um nó Pen. (Ou use **Arquivo → Script (DSL)** e
   aplique um projeto que já contenha um `pen { ... }`.)
2. **Selecione** o nó Pen no canvas (um clique).
3. No **inspector** (painel à direita), role até a caixa **Código DSL** — é
   onde se escreve o PenDSL puro (`stroke`, `circle`, `repeat`…).
4. Erros aparecem em vermelho abaixo da caixa (`linha:coluna`); o preview
   atualiza a cada edição válida.
5. Conecte a saída de uma **Cena** na entrada *Cena* do Pen, senão nada é
   desenhado.

### Abrir a janela de Script (DSL de Projeto)

1. No menu superior, clique em **Arquivo → Script (DSL)**.
2. A janela mostra o projeto TODO em texto. Um nó `pen { ... }` tem o bloco
   `codigo { ... }` com o PenDSL puro.
3. Clique em **Aplicar** (ou **Ctrl+Enter**) para reconstruir o grafo. O
   botão **Exemplos** carrega projetos prontos.
4. ⚠️ Aplicar é **destrutivo** (reconstrói o grafo inteiro) — veja as
   observações na seção *DSL de Projeto* do `documentacao_dsl.md`.

### Abrir a aba Exemplos

Na janela **Script (DSL)** há um botão **Exemplos** que lista os projetos
prontos (de `exemplos.md`). Clicar num exemplo de **Projeto** reconstrói o
grafo; clicar num exemplo de **Caneta** carrega o código num nó Pen (use
"Copiar" para levá-lo ao *Código DSL* de um Pen existente).

---

## Como os exemplos chegam ao app

- **Exemplos de Projeto** (começam com `project "..."`): ao clicar na aba
  Exemplos, reconstróem o grafo inteiro (Canvas → Cena → nós → Master).
  > Atenção: aplicar um projeto **apaga** os nós existentes — o script
  > descreve o projeto TODO.
- **Exemplos de Caneta** (código puro: `stroke`, `circle`, `repeat`...): ao
  clicar, são envolvidos num nó `pen` e aplicados. Use o botão "Copiar" para
  levar só o código para o campo *Código DSL* de um nó Pen que você já tem.

---

## Sintaxe da Caneta (mini-linguagem do nó Pen)

Comandos disponíveis (coordenadas em unidades de projeto, centro em
`largura/2, altura/2`):

```
let nome = expr          # variável
move x y                 # move "caneta" para (x, y)
point x y               # igual move, mas lê como "define ponto atual"
line x y                # linha até (x, y)
line_to x y            # igual line, parte do ponto atual (atalho lt)
curve_to c1x c1y c2x c2y x y  # bezier do ponto atual (atalho ct)
bezier c1x c1y c2x c2y x y
rect x y w h            # retângulo (canto em x y, tamanho w h)
circle x y r            # círculo
text "str" x y [size] [bold] [italic] [align left|center|right] [rot graus]
polygon n cx cy r        # polígono regular de n lados
star n cx cy r1 r2       # estrela de n pontas
arc a0 a1 r cx cy        # arco de a0° a a1°
round_rect x y w h r     # retângulo com cantos arredondados
grid cols rows x y w h pr  # grade de cruzes
translate x y           # translada o sistema de coordenadas
rotate graus            # rotaciona o sistema de coordenadas (atalho rot)
scale sx sy             # escala o sistema de coordenadas
push                    # salva estado (transform + cor + estilo)
pop                     # restaura estado salvo por push
snake x y length segments  # linha serpenteante (cobra)
close                    # fecha o path
fill on | off
stroke w
color r g b | r g b a | nome   # 0..1, ou nome (red, azul…)
stroke_color ... / fill_color ...
nome = expr             # atribuição direta (ex.: px = px + 10)
repeat n { ... }          # i = 0..n-1
for v in a..b { ... }
while cond { ... }
if cond { ... } else { ... }   # else if também
fn nome(a, b) { return ... }   # funções definidas pelo usuário
return expr
```

Funções de interpolação/reescala: `lerp(a, b, t)`,
`map(v, fromA, toA, fromB, toB)`. Vetores: `vec2(x, y)` acessado por `.x`/`.y`.
Animação: `ease(x, "tipo")` (suaviza) e `osc(freq, amp, offset)` (oscila).
Veja exemplos 15–18 e 19–20 em `documentacao_dsl.md`.

Qualquer argumento aceita uma **expressão entre parênteses**, ex.:
`circle 0 0 (100 + sin(t*2 + i)*20)` ou `line (cos(a)*ra) (sin(a)*rb)`.
O sinal de menos unário funciona solto em argumentos: `move 0 -20`,
`line -50 -50`. Para subtrair *dentro* de um argumento use parênteses:
`circle 0 0 (100 - 5)`. Veja detalhes em `documentacao_dsl.md`.

Expressões: `+ - * / %`, `and`/`or`, comparações `> < >= <= == !=`
(resultam em `1.0` ou `0.0`), parênteses, e chamadas `cos, sin, tan, sqrt,
abs, floor, noise(x)` / `noise(x,y)`, `rand()` / `rand(a,b)`, `vec2(x,y)`,
`ease(x, "tipo")`, `osc(freq, amp, offset)`. Variáveis implícitas: `t` (tempo em
s.); `phase` (t·2π); `beat` (0..1 a 120bpm); `progress` (0..1 do ciclo); `i` é
o índice do `repeat` (0 fora dele). `rand()` retorna um número pseudoaleatório
determinístico (ver abaixo).

---

## Dicas por exemplo (veja o código em `exemplos.md`)

- **Coração Animado com Rotação** — coração paramétrico com rotação suave
  `cos(t*…)` que vai e volta. Bom ponto de partida para entender `repeat` +
  rotação 2D manual (`rx = x*cos - y*sin`).
- **Bolinhas em Linha com Cores Variáveis** — `color r g b` muda por
  `i` e `t` dentro do loop.
- **Circulos com Raio Pulsante / Anel / Espirais / Ondas / Orbitas** —
  variações de `repeat` + `sin/cos` para gerar formas emergentes.
- **Barra / Linhas / Triângulos / Diamantes / Grade** — `rect`, `line`,
  `close` e composição de paths. O `close` fecha o polígono.
- **Formas com Shape** — exemplo de **Projeto** (nó `shape` ligado à
  Master via `edge`).
- **Coração Animado com Partículas** — exemplo de **Projeto** com 3 nós
  `pen` (`coracao_principal`, `particulas`, `coracao_orbitando`) mostrando
  partículas orbitando e coração pulsante.
- **Espiral Mágica** — exemplo de **Projeto** com `pen espiral` desenhando
  uma espiral que cresce com `i` e gira com `t`.
- **Título com Texto** — exemplo de **Projeto** usando o nó `text`
  (`content`, `size`, `bold`, `italic`, `color`, `pos`).
- **Texto Desenhado na Caneta** — exemplo de **Projeto** com `pen` usando o
  comando `text "str" x y [size] [bold] [italic]` (cor vem do `color`).
- **Texto + Caneta** — exemplo de **Projeto** combinando nó `text` (título)
  e nó `pen` (decoração) na mesma cena, ambos na Master.
- **Primitivas de Forma na Caneta** — exemplo de **Projeto** com `pen` usando
  `polygon`, `star`, `arc`, `round_rect` e `grid` (formas prontas).
- **Texto Rico na Caneta** — exemplo de **Projeto** com `text` + `align`
  + `rot` (negrito/itálico, alinhamento e rotação).
- **Random Determinístico na Caneta** — exemplo de **Projeto** com `pen` usando
  `rand(a, b)` para gerar formas aleatórias reprodutíveis (mesma *Seed*).

### Comando `text` na Caneta

A DSL da caneta agora desenha texto direto no código:

```
text "string" x y [size] [bold] [italic] [align left|center|right] [rot graus]
```

- `x y` é o canto superior-esquerdo (coords de projeto, origem no centro);
  com `align center`/`right` o texto é deslocado horizontalmente em relação a esse ponto.
- `size` (px de projeto) é opcional; padrão 48.
- `bold` / `italic` são flags isoladas opcionais.
- `align left|center|right` alinha horizontalmente (padrão `left`).
- `rot graus` rotaciona o texto (em torno do canto superior-esquerdo).
- A cor do texto é a cor atual da caneta (definida por `color`).

Exemplo animado (o texto treme com o tempo):

```
text "OI" (sin(t)*50) 10 32 bold italic
```

### Random determinístico (`rand`)

`rand()` retorna um número pseudoaleatório em `[0, 1)` e `rand(a, b)` em
`[a, b)`. A sequência é **determinística** a partir da *Seed* do nó, então o
mesmo nó sempre gera o mesmo resultado — reprodutível no preview e no export.

```
repeat 24 {
  let px = rand(-300, 300)
  let py = rand(-200, 200)
  circle px py (rand(4, 14))
}
```

---

## Como adicionar um exemplo

1. Edite [`exemplos.md`](exemplos.md).
2. Dê um título numa linha `# Nome do Exemplo` (vira o nome na aba).
3. Cole o código (caneta pura OU projeto completo).
4. Separe de outros exemplos com uma linha `---` (ou `===`).
5. Comentários internos com `#` são ignorados pelo parser — não usam
   `---`, então não criam cortes acidentais.

Pronto: o exemplo aparece sozinho na aba Exemplos após recompilar.
