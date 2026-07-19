# Coração Animado com Rotação

# Coração animado com rotação e repetição infinita
stroke 2
color 1 0.2 0.4
fill on

let s = 12

# Lógica de rotação e animação baseada no tempo t:
# t vai subindo continuamente. Usamos o operador de tempo para criar os ciclos:
# - Gira 3 voltas para um sentido (6*PI radianos)
# - Volta 3 voltas para o sentido oposto
# Ciclo completo de ida e volta acontece a cada 6 segundos

let ciclo = t * 1.05
let fase = floor(ciclo)
let progresso = ciclo - fase

# Alterna a direção da rotação dependendo se a fase é par ou ímpar
# Se fase é par, gira positivo (até 3 voltas = 18.84 radianos)
# Se fase é ímpar, gira negativo (volta 3 voltas)
let angulo = 0

# Como não temos if/else explícito, usamos a matemática do ciclo:
# Se o resto da divisão da fase por 2 for 0, vai. Senão, volta.
# Truque: usamos o seno ou a propriedade do ciclo para vai e volta perfeito:
# cos(ciclo * 3.14159) cria um movimento suave de vai e volta automático!

let rotacao = cos(t * 1.047) * 18.8439

# Opcional: Se sua DSL aceitar transformações de rotação direto, ótimo.
# Se precisar girar os pontos matematicamente dentro do loop:

repeat 100 {
  let a = i * 0.0628318

  let x = 16 * sin(a) * sin(a) * sin(a)
  let y = -(13 * cos(a) - 5 * cos(2*a) - 2 * cos(3*a) - cos(4*a))

  # Aplicando rotação 2D nos pontos (x, y) usando a variável 'rotacao'
  let rx = x * cos(rotacao) - y * sin(rotacao)
  let ry = x * sin(rotacao) + y * cos(rotacao)

  line (rx * s) (ry * s)
}

---

# Bolinhas em Linha com Cores Variaveis

stroke 2
fill on

# Desenha 10 bolinhas em uma linha

repeat 10 {

    # posição horizontal
    let px = -450 + (i * 100)

    # movimento com atraso
    let py = sin(t * 2 + i * 0.5) * 150

    # cores mudando com o tempo e posição
    let r = sin(t + i * 0.5) * 0.5 + 0.5
    let g = sin(t + i * 0.5 + 2.0) * 0.5 + 0.5
    let b = sin(t + i * 0.5 + 4.0) * 0.5 + 0.5

    color r g b

    circle (px) (py) 30
}

---

# Circulos com Raio Pulsante

stroke 2
fill on
color 0.2 0.6 1

repeat 10 {
    let px = -450 + (i * 100)
    # Raio varia entre 10 e 50 baseado no tempo
    let raio = 30 + sin(t * 3) * 20

    circle (px) 0 (raio)
}

---

# Anel de Bolinhas Girando

stroke 2
fill on
color 0.8 0.2 0.5

repeat 15 {
    # 'i' define o ângulo inicial, 't' faz a animação girar
    let angulo = i * (6.28 / 15)
    let distancia = 100 + sin(t * 3) * 50
    let px = cos(angulo) * distancia
    let py = sin(angulo) * distancia
    color (i/15) 0.5 1
    circle px py 15
}

---

# Espirais em Expansao

stroke 2
fill on
repeat 10 {
    let px = -450 + (i * 100)
    # Efeito de queda acelerada
    let py = -200 + abs(sin(t * 2 + i * 0.3) * 300)
    color 1 (i/10) 0.2
    circle px py 25
}

---

# Ondas Verticais

stroke 2
fill on
repeat 12 {
    let py = -300 + (i * 50)
    let px = sin(t * 4 + i * 0.5) * 150
    color 0.2 0.8 0.8
    circle px py 20
}

---

# Orbitas Cruzadas

stroke 2
fill on
repeat 8 {
    let px = cos(t * 2) * 200 + cos(t * 5 + i) * 50
    let py = sin(t * 2) * 200 + sin(t * 5 + i) * 50
    color (i/8) (1 - i/8) 1
    circle px py 30
}

---

# Barra com Oscilação

stroke 2
fill on
color 0.2 0.8 0.5

repeat 10 {
    let px = -400 + (i * 90)
    # A altura varia com o tempo e a posição
    let h = 50 + sin(t * 3 + i) * 100
    # O retângulo é desenhado a partir do centro
    rect (px) (0) (60) (h)
}

---

# Linhas Oscillantes

stroke 5
fill off
color 0.9 0.3 0.1

repeat 20 {
    let x = -450 + (i * 50)
    let y_offset = sin(t * 2 + i * 0.4) * 150
    # Desenha uma linha de cima para baixo
    move (x) (y_offset + 100)
    line (x) (y_offset - 100)
}

---

# Triangulos Girando

stroke 3
fill on
color 0.1 0.5 0.9

repeat 8 {
    let angulo = t * 2 + (i * 0.8)
    let px = cos(angulo) * 200
    let py = sin(angulo) * 200
    # Deslocamento para o triângulo
    move (px) (py + 30)
    line (px + 30) (py - 30)
    line (px - 30) (py - 30)
    close
}

---

# Diamantes Pulsantes

stroke 2
fill on
color 0.9 0.9 0.1

repeat 12 {
    let px = -400 + (i * 75)
    let tamanho = 20 + sin(t * 4) * 20
    # Diamante desenhado via linhas conectadas
    move (px) (tamanho)
    line (px + tamanho) (0)
    line (px) (-tamanho)
    line (px - tamanho) (0)
    close
}

---

# Grade de Quadrados

stroke 1
fill on
color 0.6 0.1 0.8

repeat 25 {
    let px = -450 + (i * 40)
    let py = sin(t * 2 + i * 0.5) * 200
    # Desenha um quadrado de 20x20
    rect (px - 10) (py - 10) (20) (20)
}

---

# Formas com Shape

project "Formas" { width 1280 height 720 fps 30 duration 6 background #202028 }

scene s1 { name "Cena 1" opacity 1.0 }

shape sh3 {
  scene s1
  type triangle
  pos 640 200
  size 160 160
  color #7affa3
  amp 15
  speed 3
}

edge s1.out -> master.in

---

# Coração Animado com Partículas

project "Exemplo" {
  width 1920
  height 1080
  fps 30
  duration 8
  background #1e1e26
}

scene s1 { name "Cena 1" opacity 1.0 }

pen coracao_principal {
  scene s1
  pos 960 540
  stroke 3.5
  fill on
  codigo {
    color 1 0.18 0.38

    let s = 13.5
    let rotacao = cos(t * 0.95) * 18.85

    repeat 120 {
      let a = i * 0.05236
      let x = 16 * sin(a) * sin(a) * sin(a)
      let y = -(13 * cos(a) - 5 * cos(2*a) - 2 * cos(3*a) - cos(4*a))

      let rx = x * cos(rotacao) - y * sin(rotacao)
      let ry = x * sin(rotacao) + y * cos(rotacao)

      line (rx * s) (ry * s)
    }
  }
}

pen particulas {
  scene s1
  pos 960 540
  codigo {
    color 1 0.65 0.9
    stroke 2

    repeat 24 {
      let ang = i * 0.2618 + t * 1.1
      let dist = 210 + sin(t * 2.8 + i) * 25
      let px = cos(ang) * dist
      let py = sin(ang) * dist * 0.75
      circle px py 3.5
    }
  }
}

pen coracao_orbitando {
  scene s1
  stroke 2.8
  fill on
  codigo {
    color 1 0.45 0.65

    let ang_orbita = t * 1.4
    let dist = 290
    let offset_x = cos(ang_orbita) * dist
    let offset_y = sin(ang_orbita) * (dist * 0.68)

    let scale = 4.2

    repeat 90 {
      let a = i * 0.06981
      let x = 16 * sin(a) * sin(a) * sin(a)
      let y = -(13 * cos(a) - 5 * cos(2*a) - 2 * cos(3*a) - cos(4*a))

      let rx = x * scale + offset_x
      let ry = y * scale + offset_y

      line rx ry
    }
  }
}

edge s1.out -> master.in
---

# Caminho com Point / LineTo / CurveTo

# Sintaxe limpa para contornos: point define o inicio, line_to/curve_to
# continuam do ponto atual. Expressoes diretas nos argumentos (sem let).

project "Caminho Limpo" {
  width 1280 height 720 fps 30 duration 6 background #141420
}

scene s1 { name "Cena 1" opacity 1.0 }

pen traco {
  scene s1
  pos 640 360
  stroke 4
  color #06d6a0
  codigo {
    point 0 0
    line_to 120 -40
    curve_to 220 -160 320 60 400 0
    line_to 200 120
    line_to -40 80
    fill on
    close
  }
}

edge s1.out -> master.in
---

# Espiral Mágica

project "Espiral Magica" {
  width 1920 height 1080 fps 30 duration 8 background #1e1e26
}

scene s1 { name "Cena 1" opacity 1.0 }

pen espiral {
  scene s1
  pos 960 540
  stroke 2
  codigo {
    color 0.5 0.8 1.0
    repeat 120 {
      let a = i * 0.12 + t * 0.9
      let dist = i * 1.85
      let x = cos(a) * dist
      let y = sin(a) * dist
      circle x y 3
    }
  }
}

edge s1.out -> master.in
---

# Título com Texto (nó Texto) e Animação

# O nó `text` exibe texto estático. Para animar, use o comando `text`
# dentro do `codigo { }` de um nó `pen`, onde `t` dá acesso ao tempo.

project "Titulo" {
  width 1280 height 720 fps 30 duration 8 background #141420
}

scene s1 { name "Cena 1" opacity 1.0 }

# Texto via nó Texto (estático, sem animação por DSL)
text titulo {
  scene s1
  content "MOVIMENTO"
  size 96
  bold on
  color #7affa3
  pos 640 300
}

# Pen com texto animado: a posição varia com `t`
pen subtitulo_anim {
  scene s1
  pos 640 420
  codigo {
    color #cfcfe0
    text "animacao procedural" (sin(t * 0.8) * 60) (cos(t * 1.2) * 15) 40 italic align center
  }
}

edge s1.out -> master.in
---

# Texto Desenhado na Caneta (comando text)

# O comando `text "str" x y [size]` desenha texto direto no código do Pen,
# usando a cor atual (color). A posição é o canto superior-esquerdo.

project "Texto na Caneta" {
  width 1280 height 720 fps 30 duration 6 background #141420
}

scene s1 { name "Cena 1" opacity 1.0 }

pen rotulo {
  scene s1
  codigo {
    color #ffd166
    text "OLA" (-150 + sin(t * 1.2) * 30) (-120 + cos(t * 0.8) * 15) 120 bold
    color #ef476f
    text "MUNDO" (-160 + sin(t * 0.9 + 2) * 40) (-10 + sin(t * 1.5) * 20) 96 bold
    color #06d6a0
    text "procedural" (-150 + sin(t * 1.1 + 4) * 35) (90 + cos(t * 0.7) * 15) 48 italic
  }
}

edge s1.out -> master.in
---

# Texto + Caneta na Mesma Cena

# Combina um nó Texto (título) com um nó Pen (decoracao) ligados a Master.

project "Texto + Pen" {
  width 1280 height 720 fps 30 duration 6 background #141420
}

scene s1 { name "Cena 1" opacity 1.0 }

text titulo {
  scene s1
  content "PULSE"
  size 110
  bold on
  color #ffd166
  pos 640 260
}

pen decor {
  scene s1
  pos 640 360
  stroke 3
  color #ef476f
  codigo {
    repeat 12 {
      let a = i * 0.523598 + t * 1.5
      let dist = 200 + sin(t * 3) * 40
      let px = cos(a) * dist
      let py = sin(a) * dist
      circle px py 8
    }
  }
}

edge s1.out -> master.in
---

# Primitivas de Forma na Caneta

# polygon, star, arc, round_rect e grid — formas prontas na DSL da caneta.

project "Primitivas" {
  width 1280 height 720 fps 30 duration 6 background #141420
}

scene s1 { name "Cena 1" opacity 1.0 }

pen formas {
  scene s1
  pos 640 360
  codigo {
    color #ffd166
    polygon 6 0 -180 80
    color #ef476f
    star 5 0 0 110 50
    color #06d6a0
    round_rect -160 120 320 90 24
    color #118ab2
    arc 0 360 150 0 -180
    stroke 2
    color #f78c6b
    grid 8 1 -200 250 50 0 6
  }
}

edge s1.out -> master.in
---

# Texto Rico na Caneta

# Texto com negrito/italico, alinhamento center e rotacao (graus).

project "Texto Rico" {
  width 1280 height 720 fps 30 duration 6 background #141420
}

scene s1 { name "Cena 1" opacity 1.0 }

pen aviso {
  scene s1
  pos 640 360
  codigo {
    color #ffd166
    text "CENTRO" (sin(t * 0.7) * 40) (-80 + cos(t * 1.1) * 20) 120 bold align center
    color #ef476f
    text "inclinado" (sin(t * 0.9 + 1) * 50) (40 + sin(t * 1.3) * 15) 64 italic rot (18 + sin(t * 0.6) * 12) align center
    color #06d6a0
    text "direita" (sin(t * 0.8 + 3) * 45) (130 + cos(t * 0.5) * 15) 48 align right
  }
}

edge s1.out -> master.in
---

# Random Determinístico na Caneta

# rand(a,b) gera a MESMA sequencia para a mesma seed do no => export reprodutivel.
# rand() sozinho retorna 0..1.

project "Random" {
  width 1280 height 720 fps 30 duration 6 background #141420
}

scene s1 { name "Cena 1" opacity 1.0 }

pen confete {
  scene s1
  pos 640 360
  stroke 3
  codigo {
    repeat 24 {
      let px = rand(-300, 300)
      let py = rand(-200, 200)
      let rr = rand(4, 14)
      circle px py rr
    }
  }
}

edge s1.out -> master.in
---

# Transformações: translate / rotate / scale

# translate, rotate e scale reposicionam e deformam o sistema de coordenadas.
# perfeito para repetir formas em grade ou em anel.

project "Transform" {
  width 1280 height 720 fps 30 duration 6 background #101020
}

scene s1 { name "Cena 1" opacity 1.0 }

pen anel {
  scene s1
  pos 640 360
  stroke 2
  color 0.3 0.8 1
  codigo {
    repeat 12 {
      push
      rotate (i * 30 + t * 30)
      translate 220 0
      circle 0 0 30
      pop
    }
  }
}

edge s1.out -> master.in
---

# push / pop (estado salvo)

# push salva transform + cor + estilo; pop restaura. usa para isolar
# transformacoes ou mudancas de cor sem afetar o resto do desenho.

project "PushPop" {
  width 1280 height 720 fps 30 duration 6 background #101020
}

scene s1 { name "Cena 1" opacity 1.0 }

pen blocos {
  scene s1
  pos 640 360
  stroke 2
  codigo {
    color 1 0.4 0.2
    rect -200 -40 120 80
    push
    translate (260 + sin(t * 1.5) * 60) 0
    color 0.2 0.9 0.5
    fill on
    rect -60 -40 120 80
    pop
    # depois do pop, volta a cor laranja e sem fill
    rect 200 -40 120 80
  }
}

edge s1.out -> master.in
---

# Atribuição direta (sem let)

# var = expr atualiza a variavel (cria se nao existir). otimo para
# acumular posicoes ao longo do desenho.

project "Atribuicao" {
  width 1280 height 720 fps 30 duration 6 background #101020
}

scene s1 { name "Cena 1" opacity 1.0 }

pen trilha {
  scene s1
  pos 640 360
  stroke 3
  color 0.6 0.9 1
  codigo {
    let px = -300
    let py = 0
    point px py
    repeat 30 {
      px = px + 20
      py = sin(px * 0.02 + t * 2) * 120
      line_to px py
    }
  }
}

edge s1.out -> master.in
---

# lerp e map (interpolação / reescala)

# lerp(a, b, t)          -> a + (b-a)*t          (t em 0..1)
# map(v, fa, ta, fb, tb) -> reescala v do intervalo [fa,ta] para [fb,tb]

project "LerpMap" {
  width 1280 height 720 fps 30 duration 6 background #101020
}

scene s1 { name "Cena 1" opacity 1.0 }

pen barra {
  scene s1
  pos 640 360
  stroke 2
  fill on
  codigo {
    repeat 20 {
      let u = i / 19
      let anim = ease((progress + u * 0.2) % 1, "quad")
      # cor vai de azul (0,0.4,1) a rosa (1,0.2,0.6) pelo lerp
      let r = lerp(0, 1, u)
      let g = lerp(0.4, 0.2, u)
      let b = lerp(1, 0.6, u)
      color r g b
      let x = map(u, 0, 1, -300, 300)
      let h = lerp(20, 220, anim)
      rect x -h 20 h
    }
  }
}

edge s1.out -> master.in
---

# snake (linha serpenteante)

# snake x y length segments desenha uma cobra oscilando na vertical.

project "Snake" {
  width 1280 height 720 fps 30 duration 6 background #101020
}

scene s1 { name "Cena 1" opacity 1.0 }

pen cobra {
  scene s1
  pos 640 360
  stroke 4
  color 0.4 1 0.6
  codigo {
    let desl = sin(t) * 60
    snake (-400) desl 800 24
  }
}

edge s1.out -> master.in
---

# Animação: variáveis implícitas e easing

# t (tempo), phase (t*2π), beat (0..1 a 120bpm), progress (0..1 do ciclo).
# ease(x, "tipo") suaviza; osc(freq, amp, offset) oscila.

project "Anim" {
  width 1280 height 720 fps 30 duration 6 background #101020
}

scene s1 { name "Cena 1" opacity 1.0 }

pen pulse {
  scene s1
  pos 640 360
  fill on
  stroke 3
  color 0.3 0.8 1
  codigo {
    # raio cresce e volta com easing "quad" (suave)
    let r = lerp(20, 180, ease(progress, "quad"))
    circle 0 0 r
    # ponto que oscila horizontalmente
    let x = osc(1, 300, 0)
    let y = osc(1, 150, 1.57)
    push
    translate x y
    color #ff5500
    circle 0 0 12
    pop
  }
}

edge s1.out -> master.in
---

# Easing em cadeia (entrada e saída)

# "quadin" entra devagar; "elastic" dá molejo; "bounce" quica.

project "Easing" {
  width 1280 height 720 fps 30 duration 6 background #101020
}

scene s1 { name "Cena 1" opacity 1.0 }

pen barras {
  scene s1
  pos 640 360
  stroke 2
  fill on
  codigo {
    repeat 5 {
      let u = i / 4
      let e = ease(progress, "bounce")
      let h = lerp(10, 260, e)
      let x = map(u, 0, 1, -300, 300)
      rect x (-h) 40 h
    }
  }
}

edge s1.out -> master.in
---
