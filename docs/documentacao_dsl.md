# Nó Pen — DSL de desenho procedural

O nó **Pen** permite desenhar formas vetoriais arbitrárias escrevendo um
pequeno programa em uma linguagem própria (DSL) no inspector do nó. Diferente
do nó **Shape** (que expõe sliders de retângulo/elipse/estrela), o Pen aceita
qualquer path: linhas, curvas de Bézier, círculos, polígonos e repetições
animadas pelo tempo `t`.

## Onde encontrar

### 1. Criar um nó Pen

1. No **editor de grafos**, localize a **barra de ferramentas** (à esquerda ou
   no topo do canvas). Ela tem um botão colorido para cada tipo de nó.
2. Clique no botão **Pen** (roxo). Um novo nó Pen aparece no canvas.
3. Se preferir, no menu **Arquivo → Script (DSL)** você pode escrever um
   projeto em texto que já cria o nó Pen (veja a seção *DSL de Projeto* abaixo)
   e clicar em **Aplicar**.

### 2. Conectar a uma Cena

Conecte a saída de um nó **Cena** (ou **Canvas → Cena**) na entrada *Cena* do
Pen — assim a geometria aparece na cena certa do preview. Sem essa conexão, o
Pen não desenha nada.

### 3. Abrir o editor de código (campo "Código DSL")

1. **Selecione** o nó Pen com um clique no canvas (ele fica destacado).
2. O **inspector** (painel à direita) mostra os parâmetros do Pen.
3. Role até o campo **Código DSL** — é uma caixa de texto grande (multilinha).
   É ali que você escreve a mini-linguagem deste documento.
4. Erros de sintaxe aparecem em **vermelho logo abaixo da caixa**, no formato
   `linha:coluna`, assim que você digita. O preview atualiza a cada alteração
   válida.

> Dica: o texto digitado no *Código DSL* é o **PenDSL puro** (os comandos
> `move`, `line`, `circle`, `repeat`… listados abaixo). Ele é separado da
> *DSL de Projeto* (que descreve o app inteiro em `Arquivo → Script (DSL)`).

### 4. Abrir a janela de Script (DSL de Projeto)

1. No menu superior, clique em **Arquivo → Script (DSL)**. Uma janela se abre
   com o projeto inteiro descrito em texto (Canvas, Cenas, nós e conexões).
2. Dentro de um nó `pen { ... }`, o bloco `codigo { ... }` contém justamente o
   PenDSL puro. Edite-o e clique em **Aplicar** para reconstruir o grafo.
3. Atalho: **Ctrl+Enter** também aplica o script. Há um botão **Exemplos** na
   própria janela para carregar projetos prontos.

> ⚠️ Aplicar o Script (DSL) é **destrutivo**: reconstrói o grafo inteiro. Veja
> as observações na seção *DSL de Projeto* mais abaixo antes de usar.

> Coordenadas são em **unidades de projeto**, no mesmo espaço do Canvas. O
> centro da cena fica em `(largura/2, altura/2)` — por padrão `(960, 540)` para
> um projeto 1920×1080. O deslocamento do nó (**Posição**) é somado a todos os
> pontos.

## Referência da linguagem

Cada linha é um comando. Comentários começam com `#` e valem tanto no início
da linha quanto depois de um comando (`move 1 2  # nota`).

| Comando | Sintaxe | Descrição |
|---|---|---|
| `let` | `let nome = expr` | Define uma variável (escalar). |
| `move` | `move x y` | Move a "caneta" para `(x, y)` sem desenhar. |
| `line` | `line x y` | Desenha uma linha até `(x, y)`. |
| `point` | `point x y` | Define o ponto atual sem desenhar (atalho de `move`). Bom para iniciar um caminho antes de `line_to`/`curve_to`. |
| `line_to` / `lt` | `line_to x y` | Desenha uma linha do ponto atual até `(x, y)` (atalho de `line`). |
| `curve_to` / `ct` | `curve_to c1x c1y c2x c2y x y` | Curva de Bézier do ponto atual até `(x, y)` com pontos de controle `(c1x, c1y)` e `(c2x, c2y)` (atalho de `bezier`). |
| `rect` | `rect x y w h` | Retângulo com canto em `(x, y)` e tamanho `(w, h)`. |
| `circle` | `circle x y r` | Círculo (polígono de 48 lados) centrado em `(x, y)`. |
| `bezier` | `bezier cx1 cy1 cx2 cy2 x y` | Curva de Bézier cúbica até `(x, y)`. |
| `close` | `close` | Fecha o path atual (liga o último ponto ao primeiro). |
| `fill` | `fill on` / `fill off` | Liga/desliga o preenchimento do path. |
| `stroke` | `stroke w` | Espessura do traço (px de projeto). |
| `color` | `color nome` \| `color r g b` \| `color r g b a` | Cor do traço **e** do preenchimento (nome, rgb 0..1, ou rgba 0..1 com alpha). |
| `stroke_color` | `stroke_color nome` \| `stroke_color r g b [a]` | Cor **apenas do traço** (contorno). Mesmos formatos de `color`. |
| `fill_color` | `fill_color nome` \| `fill_color r g b [a]` | Cor **apenas do preenchimento**. Mesmos formatos de `color`. |
| `repeat` | `repeat n { ... }` | Repete o bloco `n` vezes (usa `i` = índice 0..n-1). |
| `for` | `for v in a..b { ... }` | Repete com `v` varrendo `[a, b)` (passo 1). |
| `while` | `while cond { ... }` | Repete enquanto `cond` ≠ 0 (limite de 100000 iterações). |
| `if` | `if cond { ... }` | Executa o bloco se `cond` ≠ 0. |
| `if/else` | `if cond { ... } else { ... }` | Ramo alternativo quando `cond` = 0. |
| `else if` | `if c1 { ... } else if c2 { ... } else { ... }` | Cadeia de condições. |
| `text` | `text "str" x y [size] [bold] [italic] [align left\|center\|right] [rot graus]` | Desenha texto direto na caneta (mesma fonte do nó Texto). A cor vem do `color`. `align` alinha horizontalmente; `rot` rotaciona em graus. |
| `polygon` | `polygon n cx cy r` | Polígono regular de `n` lados (n≥3), raio `r`, centrado em `(cx, cy)`. |
| `star` | `star n cx cy r1 r2` | Estrela de `n` pontas, raios `r1` (pontas) e `r2` (vales), centrada em `(cx, cy)`. |
| `arc` | `arc a0 a1 r cx cy` | Arco do ângulo `a0` ao ângulo `a1` (graus), raio `r`, centrado em `(cx, cy)`. |
| `round_rect` / `roundrect` | `round_rect x y w h r` | Retângulo com cantos arredondados (raio `r`) a partir de `(x, y)` com tamanho `(w, h)`. |
| `grid` | `grid cols rows x y w h pr` | Grade de `cols`×`rows` cruzes (raio `pr`), espaçadas `w`×`h`, a partir de `(x, y)`. |
| `translate` / `trans` | `translate x y` | Translada o sistema de coordenadas em `(x, y)`. Afeta todos os comandos seguintes. |
| `rotate` / `rot` | `rotate ang` | Rotaciona o sistema de coordenadas em `ang` graus (sentido anti-horário). |
| `scale` | `scale sx sy` | Escala o sistema de coordenadas por `sx` (x) e `sy` (y). |
| `push` | `push` | Salva o estado atual (transformação + cor + estilo) na pilha. |
| `pop` | `pop` | Restaura o estado salvo por `push` (desfaz translate/rotate/scale e mudanças de cor). |
| `snake` | `snake x y length segments` | Desenha uma "cobra": linha serpenteante iniciando em `(x, y)`, com `length` total e `segments` segmentos oscilando na direção vertical. |
| `rand` | `rand` \| `rand(a, b)` | Número pseudoaleatório **determinístico** (0..1, ou `a..b`). A mesma *Seed* do nó produz a mesma sequência — reprodutível no export. |
| `= ` (atribuição) | `nome = expr` | Atribuição direta a uma variável (cria se não existir). Ex.: `px = px + 10`. Não confunda com `==` (comparação). |

### Expressões

Operadores (da menor para a maior precedência):

- Lógica: `and`, `or`
- Aditivo: `+`, `-`
- Comparação: `>`, `<`, `>=`, `<=`, `==` (ou `=` como atalho), `!=` — resultam em **1.0** (verdade) ou **0.0** (falso)
- Multiplicativo: `*`, `/`, `%` (módulo)
- Unário: `-` (menos), parênteses `( )`

Chamadas de função: `cos`, `sin`, `tan`, `sqrt`, `abs`, `floor`, `noise`,
`rand`, `lerp`, `map`, `vec2`, `ease`, `osc`. `noise(x)` é ruído 1D e `noise(x, y)`
é ruído 2D — ambos determinísticos, baseados no *Seed* do nó, no intervalo
`[-1, 1]`. `rand()` retorna um valor pseudoaleatório em `[0, 1)` e `rand(a, b)`
retorna em `[a, b)`; a sequência é **determinística** a partir da *Seed* do nó
(xorshift de 32 bits), então o mesmo nó sempre gera o mesmo resultado —
essencial para exportações PNG/vídeo reprodutíveis. `lerp(a, b, t)` = `a +
(b-a)*t` (interpolação linear; `t` em `[0, 1]`). `map(v, fromA, toA, fromB,
toB)` reescala `v` do intervalo `[fromA, toA]` para `[fromB, toB]`. `vec2(x, y)`
cria um vetor 2D (acesse com `.x`/`.y`). `ease(x, "tipo")` aplica uma curva de
suavização a `x` (em `[0, 1]`). `osc(freq, amp, offset)` = `amp * sin(2π·freq·t +
offset)` (oscilador em `[-amp, amp]`).

Variáveis implícitas:

- `t` — tempo em segundos (anima o desenho).
- `phase` — fase contínua em radianos (`t * 2π`), pronta para `cos/sin`.
- `beat` — fração `0..1` de uma batida a 120 BPM (`(t*2) % 1`), útil para pulsar.
- `progress` — fração `0..1` do ciclo (duração do projeto; padrão 6s no loop).
- `i` — índice do `repeat` atual (0-based). Vale `0` fora de um `repeat`.
- `k` (ou o nome que você der) — variável do `for`.

Tipos de `ease` suportados (sem sufixo = ease-in-out; `in`/`out` variam o
sentido): `linear`, `quad`(`in`/`out`), `cubic`, `quart`, `quint`, `expo`,
`circ`, `sine`, `back`, `elastic`, `bounce`.

Cores por nome aceitas (case-insensitive, PT ou EN): `red/vermelho`,
`green/verde`, `blue/azul`, `yellow/amarelo`, `cyan/ciano`, `magenta`,
`white/branco`, `black/preto`, `orange/laranja`, `purple/roxo`, `pink/rosa`,
`gray/grey/cinza`.

> **Sinal de menos (`-`):** cada argumento de comando aceita um `-` unário
> diretamente — `move 0 -ra` significa **x = 0, y = -ra** (já não é mais
> necessário escrever `move 0 (-ra)`). O `+` continua valendo como operador
> binário dentro do argumento, então `line x1 + 50 y1` dá x = x1+50. Para
> **subtrair dentro de um argumento** (ex.: raio `100 - 5`), use parênteses:
> `circle 0 0 (100 - 5)` — sem os parênteses o `- 5` seria lido como um novo
> argumento negativo.

> **Parênteses e expressões:** qualquer argumento pode ser uma expressão
> arbitrária envolta em `( )`, ex.: `circle 0 0 (100 + sin(t*2 + i)*20)` ou
> `line (cos(a)*ra) (sin(a)*rb)`. Parênteses aninhados funcionam, então não é
> preciso criar variáveis com `let` só para agrupar um cálculo.

## Exemplos práticos

### 1. Estrela de 5 pontas (padrão do nó)

```
# estrela de 5 pontas
let ra = 200
let rb = 80
move 0 (-ra)
repeat 5 {
  let a = i * 72
  line (cos(a)*ra) (sin(a)*ra)
  let b = a + 36
  line (cos(b)*rb) (sin(b)*rb)
}
close
fill on
color 0.78 0.47 0.08
```

`repeat 5` desenha 5 pares de pontos (externo em raio 200, interno em 80) a
cada 72°. `close` fecha o polígono e `fill on` o preenche.

### 2. Onda senoidal de pontos

```
stroke 2
color 0.2 0.6 1
move -400 0
repeat 80 {
  let x = -400 + i * 10
  let y = sin(x * 0.02 + t) * 120
  line x y
}
```

Uma linha ondulada de 80 segmentos. O `+ t` dentro do `sin` faz a onda
"viajar" no tempo no preview.

### 3. Círculo que pulsa com o tempo

```
let s = 100 + sin(t * 2) * 40
circle 0 0 s
fill on
color (0.5 + 0.5*sin(t)) 0.3 0.6
```

O raio oscila entre 60 e 140; a cor também pulsa no canal vermelho.

### 4. Espiral com `repeat` e `i`

```
let voltas = 5
repeat 200 {
  let a = i * 0.3
  let r = i * 1.2
  line (cos(a)*r) (sin(a)*r)
}
stroke 1.5
color 0.9 0.9 0.2
```

Cada iteração aumenta o raio `r` e o ângulo `a`, produzindo uma espiral.

### 5. Curva de Bézier

```
move -200 0
bezier -100 200 100 -200 200 0
stroke 4
color 0.2 0.8 0.4
```

Desenha uma curva suave de `(-200, 0)` até `(200, 0)` com pontos de controle
`(-100, 200)` e `(100, -200)`.

### 6. Coração girando (vai e volta) com `if/else`

O exemplo abaixo usa `if/else` para desenhar o coração numa cor quente na
"ida" (metade do ciclo) e fria na "volta" — em vez de truques de `cos`:

```
stroke 2
fill on
let s = 12
# metade do ciclo (6s): quente; depois: frio
if (t < 6) {
  color red
} else {
  color blue
}
let rot = cos(t * 1.047) * 18.8439
repeat 100 {
  let a = i * 0.0628318
  let x = 16 * sin(a) * sin(a) * sin(a)
  let y = -(13 * cos(a) - 5 * cos(2*a) - 2 * cos(3*a) - cos(4*a))
  let rx = x * cos(rot) - y * sin(rot)
  let ry = x * sin(rot) + y * cos(rot)
  line (rx * s) (ry * s)
}
```

### 7. Grade de círculos com `for` e `and`

```
stroke 1.5
for gx in 0..12 {
  for gy in 0..8 {
    let px = -550 + gx * 100
    let py = -350 + gy * 100
    # pisca a cada 2s apenas nas posições pares
    if ((gx % 2 == 0) and (gy % 2 == 0)) and (floor(t / 2) % 2 == 0) {
      color 0.2 0.8 0.5
      circle px py 30
    } else {
      color 0.3 0.3 0.4
      circle px py 12
    }
  }
}
```

### 8. Rabisco com `while` (espiral controlada por condição)

```
stroke 2
color purple
let r = 0
let a = 0
while (r < 300) {
  let x = cos(a) * r
  let y = sin(a) * r
  line x y
  let r = r + 4
  let a = a + 0.3
}
```

> Nota: dentro de `while`/`repeat`/`for` você pode **redefinir** uma
> variável com `let` (como `let r = r + 4` acima) — isso cria uma nova
> "sombra" do escopo do laço, que some ao sair do bloco.

### 9. Ruído orgânico (1D e 2D)

```
stroke 2
for k in 0..120 {
  let x = -500 + k * 8
  let y = noise(x * 0.01, t * 0.5) * 160
  color 0.4 0.7 noise(k * 0.02) * 0.5 + 0.5
  line x y
}
```

`noise(x, y)` (2D) perturba a altura ao longo do tempo; `noise(z)` (1D)
perturba a cor.

### 10. Primitivas de forma prontas

```
color 0.9 0.6 0.1
polygon 6 0 -180 80        # hexágono
color 0.94 0.28 0.44
star 5 0 0 110 50          # estrela de 5 pontas
color 0.02 0.84 0.63
round_rect -160 120 320 90 24   # retângulo arredondado
color 0.07 0.54 0.7
arc 0 270 150 0 -180       # arco de 0° a 270°
```

Cada primitiva já emite o path fechado/aberto correspondente — não precisa de
`move`/`line`/`close` manual. `grid` desenha uma malha de cruzetas:

```
stroke 2
color 0.97 0.55 0.42
grid 8 1 -200 250 50 0 6
```

### 11. Texto com alinhamento e rotação

```
color 1 0.82 0.4
text "CENTRO" 0 -80 120 bold align center
color 0.94 0.28 0.44
text "inclinado" 0 40 64 italic rot 18 align center
color 0.02 0.84 0.63
text "direita" 0 130 48 align right
```

`align center`/`right` desloca o texto em relação ao `(x, y)`; `rot` gira o
texto em graus (sempre em torno do canto superior-esquerdo).

### 12. Random determinístico

```
stroke 3
repeat 24 {
  let px = rand(-300, 300)
  let py = rand(-200, 200)
  let rr = rand(4, 14)
  circle px py rr
}
```

A mesma *Seed* do nó gera exatamente a mesma nuvem de círculos, quadro a
quadro e no export — `rand` não usa o relógio do sistema.

### 13. Caminhos com `point` / `line_to` / `curve_to`

Em vez de `move`/`line`/`bezier`, você pode iniciar o caminho com `point` e
continuar com `line_to` (atalho de `line`) e `curve_to` (atalho de `bezier`,
cuja origem é o ponto atual implícito):

```
stroke 4
color 0.2 0.8 0.4
point 0 0
line_to 100 0
curve_to 200 -80 300 80 400 0
line_to 200 120
```

`point 0 0` posiciona a caneta; cada `line_to`/`curve_to` parte dessa posição
e a deixa onde terminou — ótimo para desenhar contornos sequenciais.

### 14. Expressões diretas (sem `let` toda hora)

Qualquer argumento aceita uma expressão entre parênteses, com parênteses
aninhados — não precisa de `let` para agrupar cálculos:

```
let i = 0          # só o índice do repeat
repeat 60 {
  let a = i * 6
  circle (cos(a)*150) (sin(a)*150) (8 + sin(t*2 + i)*5)
}
```

O `(8 + sin(t*2 + i)*5)` é uma expressão completa usada direto como raio.
O sinal de menos unário também funciona solto em argumentos: `move 0 -20`,
`line -50 -50`.

### 15. Transformações (translate / rotate / scale)

`translate`, `rotate` e `scale` reposicionam e deformam o sistema de coordenadas
para todos os comandos seguintes — perfeito para repetir formas em anel ou em
grade:

```
# anel de 12 círculos
repeat 12 {
  push
  rotate (i * 30)
  translate 220 0
  circle 0 0 30
  pop
}
```

### 16. push / pop (estado salvo)

`push` salva o estado (transformação + cor + estilo) e `pop` restaura. Use para
isolar mudanças sem afetar o resto do desenho:

```
color 1 0.4 0.2
rect -200 -40 120 80
push
translate 260 0
color 0.2 0.9 0.5
fill on
rect -60 -40 120 80
pop
# depois do pop, volta a cor laranja e sem fill
rect 200 -40 120 80
```

### 17. Atribuição direta (sem `let`)

`nome = expr` atualiza a variável (cria se não existir). Ótimo para acumular
posições ao longo do desenho:

```
let px = -300
let py = 0
point px py
repeat 10 {
  px = px + 60
  py = sin(px * 0.02) * 120
  line_to px py
}
```

### 18. lerp / map e snake

`lerp(a, b, t)` interpola; `map(v, fa, ta, fb, tb)` reescala intervalos.
`snake x y length segments` desenha uma cobra oscilando:

```
repeat 20 {
  let u = i / 19
  let r = lerp(0, 1, u)        # 0 -> 1
  let g = lerp(0.4, 0.2, u)
  let b = lerp(1, 0.6, u)
  color r g b
  let x = map(u, 0, 1, -300, 300)
  rect x -20 20 40
}
# ...
snake (-400) (sin(t)*60) 800 24
```

## Notas de implementação

- O código é **parseado uma vez** quando editado (o erro é reportado ao nó) e
  **avaliado por frame** com o tempo `t` — o parser em si não roda a cada
  quadro, só o avaliador (leve).
- `repeat` e `for` têm limite de 2000 iterações; `while` tem limite de 100000.
- Não há `eval` de Rust: a linguagem é um interpretador fechado (sem I/O),
  então não há risco de execução arbitrária.
- `noise(x)` retorna ruído determinístico 1D e `noise(x,y)` 2D no intervalo
  `[-1, 1]`, variando com o *Seed* do nó — útil para perturbações orgânicas.
- Erros de **avaliação** (ex.: `i` fora de `repeat`, divisão por zero) são
  reportados como `linha N: ...`, usando a linha do comando onde ocorreram.

## Estrutura de arquivos

- `src/dsl/pen.rs` — parser (recursivo-descendente), AST e avaliador da DSL da caneta.
- `src/procedural.rs` — `PenPath` (programa cacheado + estilo) e `CenaPreview::pen`.
- `src/ui/preview.rs` — `pen_cmds_para_shapes` converte `PathCmd` em `egui::Shape`.
- `src/ui/node_component.rs` — inspector (textarea + cor + erro).
- `src/ui/graph_editor.rs` — `formas_para_preview` coleta os nós Pen por cena.

---

# DSL de Projeto (autoramento do app inteiro)

Além do PenDSL, o Movimento aceita uma **DSL de projeto** que descreve o
aplicativo TODO em texto: configuração, cenas, nós e conexões. É uma
linguagem de autoramento — você escreve o projeto como um script e clica
**Aplicar** para reconstruir o grafo. Acesse via menu **Arquivo → Script (DSL)**.

## Sintaxe

`	ext
project "Nome" {
    width 1920
    height 1080
    fps 30
    duration 8
    background #1e1e26
}

scene s1 { name "Cena 1" opacity 1.0 }

shape sh1 {
    scene s1
    type star
    pos 960 540
    size 300 300
    color #eb9678
    noise 0.4
    amp 30
    speed 1
}

text tx1 {
    scene s1
    content "Movimento"
    size 80
    pos 960 700
    color #f0f0f5
}

pen p1 {
    scene s1
    pos 960 540
    stroke 3
    fill on
    color #c878dc
    codigo {
        # aqui dentro vale o PenDSL puro
        repeat 5 { line (cos(i)*10) (sin(i)*10) }
    }
}

layer l1 { scene s1 name "Formas" }

# Conexões entre portos de nós
edge l1.Formas -> sh1.Layer
edge l1.Formas -> tx1.Layer
edge l1.Formas -> p1.Layer
edge sh1.out -> master.in
edge tx1.out -> master.in
edge p1.out -> master.in
`

## Comandos de alto nível

| Comando | Descrição |
|---|---|
| project | Config do projeto (Canvas). Campos: width, height, ps, duration, ackground. O nome é opcional (entre aspas). |
| canvas | Sinônimo do nó Canvas (as config vêm de project). |
| scene | Nó de cena. Campos: 
ame, zoom, ngle, opacity. |
| layer | Nó de camada. Campos: scene, opacity. |
| shape | Nó de forma. Campos: scene, type, pos, size, rot, color, seed, noise, amp, speed, trim_start, trim_end. |
| text | Nó de texto. Campos: scene, content, size, bold, italic, pos, color, trim_start, trim_end. |
| pen | Nó Pen. Campos: scene, pos, stroke, fill, color, stroke_color, fill_color, seed, trim_start, trim_end, e o bloco codigo { }. `color` define traço e preenchimento juntos; `stroke_color`/`fill_color` definem cada um separadamente. |
| edge | Conexão: edge A.porto -> B.porto. |

## Portos nas conexões (edge)

A sintaxe é edge <nó_origem>.<porto_saida> -> <nó_destino>.<porto_entrada>.
Atalhos: .out / .in referenciam o primeiro porto de saída/entrada.
Nomes de porto aceitam aliases curtos em inglês: pos, size, color,
ot, canvas, scene, layer, pen — mapeados para os nomes reais
(em português) dos portos do nó.

## Bloco codigo { } do Pen

Dentro de um nó pen, o bloco codigo { ... } contém **PenDSL puro**
(veja a primeira seção deste documento). Seu conteúdo é passado cru ao
parser do Pen — não é re-interpretado pela DSL de projeto.

## Observações

- **⚠️ Aplicar é DESTRUTIVO — reconstrói o grafo inteiro.** Ao clicar em
  **Aplicar**, o grafo atual é **substituído por completo** pelo descrito no
  script. Tudo o que não estiver no texto (nós, conexões, edições manuais no
  canvas) é **perdido**. O Canvas, a Cena e o Master são recriados do zero.
- **Como não perder trabalho ao mexer num projeto já existente:**
  1. Sempre que quiser alterar o projeto via script, **comece do texto atual**,
     não de um esqueleto vazio. A janela *Script (DSL)* mostra o estado do
     projeto como texto; edite-o e aplique.
  2. Para **adicionar** um nó sem apagar os outros, mantenha os blocos
     `project`, `scene`, `shape`, etc. existentes e **acrescente** o novo
     bloco (ex.: outro `pen p2 { ... }`). O script descreve o projeto TODO —
     ele não é "patch", é o estado completo.
  3. Se você editou nós no canvas arrastando/sliders e quer preservá-los,
     gere (ou copie) o script correspondente antes de aplicar qualquer outro
     script. Hoje o app **não** exporta o grafo de volta para texto
     automaticamente; a fonte canônica é o último script que você aplicou.
  4. Use o botão **Salvar** (JSON do projeto) para backup do estado binário,
     mas lembre-se de que o *Script (DSL)* é uma reescrita total — salvar o
     JSON não impede que um Aplicar substitua o grafo.
- **Blocos terminam com `}`** ou com **duas ou mais linhas em branco**
  consecutivas (é assim que o parser separa objetos sem exigir a chave de
  fechamento). Ex.: você pode escrever um `shape` e dar 2 linhas vazias em
  vez de fechar com `}`.
- **Erros**: aparecem na própria janela de Script, com o número da linha
  (`linha N: ...`).
- **Cenas**: referencie pelo id usado no `scene` (ex.: `s1`). O `scene` de
  cada nó é resolvido pelo id informado no campo `scene`.
- **Hex e comentários**: cores usam `#rrggbb` (ex.: `#1e1e26`). O caractere
  `#` **só** é tratado como comentário quando está no início da linha (após
  espaços); dentro de um valor (`background #1e1e26`) ele é lido como cor.

## Estrutura de arquivos

- src/ui/project_dsl.rs — parser + AST da DSL de projeto, e helpers
  (parse_script, 	ipo_da_dsl, indice_porto, hex_para_cor).
- src/ui/graph_editor.rs — GraphPanel::aplicar_script aplica o AST
  criando nós (dicionar_no_em) e arestas (conectar_parametro).
- src/app.rs — janela "Script (DSL)" no menu Arquivo, com textarea e botão
  **Aplicar**.
