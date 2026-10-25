# Spec Delta

## Purpose

Definir `srt-stage`, la clasificación de la **etapa de transcripción** de un SRT
(`raw`/`final`) por **contenido**, y su uso como guardián: pedir un entregable de una
etapa que el SRT no es debe fallar con un código distinto de cero.

## ADDED Requirements

### Requirement: La etapa se decide por contenido, nunca por el nombre

`srt-stage` SHALL clasificar cada SRT por su **contenido**, evaluando las etapas del
perfil en el orden de `srt_stages.order` y quedándose con la primera que encaje. NO SHALL
usar el nombre del fichero para decidir la etapa. Con una única etapa `detect: any`, todo
SRT del show SHALL clasificarse en esa etapa.

#### Scenario: show de una sola etapa
- **GIVEN** un perfil con una única etapa `final` y `detect: any`
- **WHEN** se clasifica cualquier SRT
- **THEN** la etapa SHALL ser `final`

#### Scenario: el nombre no decide
- **GIVEN** un SRT del bruto con un nombre no canónico (p. ej. `-edicion`) cuyo contenido es conversación
- **WHEN** se clasifica
- **THEN** la etapa SHALL ser `raw`, con independencia del nombre

### Requirement: El `final` exige las dos señales del intro combinadas con un AND

Para clasificar una etapa `detect: starts_with_intro`, `srt-stage` SHALL exigir **las dos
señales** del intro, combinadas con un AND:

1. **Texto:** el primer cue **con texto real** (ignorando los cues iniciales vacíos o de
   solo puntuación) SHALL coincidir con `intro_pattern` **empezando dentro de los
   primeros** `intro_anchor_chars` caracteres del arranque (primer cue + su continuación
   inmediata). El patrón del perfil exige el nombre del programa, de modo que una fórmula
   de apertura casual sin el nombre NO SHALL disparar la señal.
2. **Bloque:** el **primer cue del fichero** (el primero con línea de tiempos, con o sin
   texto) SHALL arrancar en `<= intro_block_max_start` segundos y terminar en
   `>= intro_block_min_seconds` segundos.

Si `intro_block_min_seconds` no está declarado, la señal de bloque NO SHALL aplicarse.

#### Scenario: las dos señales presentes → final
- **GIVEN** un SRT cuyo primer cue arranca en 00:00:00, dura 26 s, y cuyo primer cue con texto real empieza por el intro del show
- **WHEN** se clasifica
- **THEN** la etapa SHALL ser `final`

#### Scenario: solo la señal de texto → bruto (AND)
- **GIVEN** un SRT cuyo primer cue arranca en 00:00:00 pero dura 6 s, aunque el texto empiece por el intro
- **WHEN** se clasifica
- **THEN** la etapa NO SHALL ser `final` (SHALL ser `raw`)

#### Scenario: fórmula casual sin el nombre del programa → bruto
- **GIVEN** un SRT que arranca con una frase de bienvenida genérica que no nombra el programa
- **WHEN** se clasifica
- **THEN** la etapa NO SHALL ser `final`

#### Scenario: el intro partido en dos cues
- **GIVEN** un SRT cuyo primer cue abre el intro de forma cortada y el nombre del programa cae en el segundo cue
- **WHEN** se clasifica
- **THEN** la señal de texto SHALL evaluarse también sobre la continuación inmediata del primer cue

#### Scenario: cue inicial de solo puntuación se ignora
- **GIVEN** un SRT cuyo primer cue es `-` (o `...`, `—`) y el intro real está en el segundo cue
- **WHEN** se clasifica
- **THEN** el primer cue CON TEXTO REAL SHALL ser el segundo

### Requirement: El marcador literal del intro basta por sí solo

Si el perfil declara `intro_marker` y el primer cue con texto real (normalizado) es
**exactamente** ese marcador, `srt-stage` SHALL clasificar la etapa `starts_with_intro`
sin exigir la señal de bloque.

#### Scenario: marcador literal
- **GIVEN** un perfil con `intro_marker: "INTRO"` y un SRT cuyo primer cue con texto real es `INTRO`
- **WHEN** se clasifica
- **THEN** la etapa SHALL ser `final`

### Requirement: `--srt` y `--episode` son excluyentes y uno es obligatorio

`srt-stage` SHALL exigir exactamente uno de `--srt <ruta>` o `--episode NN`. Si se dan
los dos, o ninguno, SHALL fallar por `stderr` con código `2`.

#### Scenario: los dos a la vez
- **GIVEN** una invocación `srt-stage --srt a.srt --episode 22`
- **WHEN** se ejecuta
- **THEN** SHALL error por `stderr` con código `2`

#### Scenario: ninguno
- **GIVEN** una invocación `srt-stage` sin `--srt` ni `--episode`
- **WHEN** se ejecuta
- **THEN** SHALL error por `stderr` con código `2`

### Requirement: `--episode` localiza los SRT con `episodes.srt_glob`

Con `--episode NN`, `srt-stage` SHALL componer `episodes.srt_glob` con el número y
clasificar **todos** los SRT que casen, en orden. Si no casa ninguno, SHALL fallar por
`stderr` con código `2`.

#### Scenario: varios SRT del episodio
- **GIVEN** un episodio con dos SRT que casan el `srt_glob`
- **WHEN** se clasifica con `--episode`
- **THEN** SHALL emitirse una línea por SRT con su etapa

#### Scenario: sin SRT para el episodio
- **GIVEN** un episodio cuyo `srt_glob` no casa ningún fichero
- **WHEN** se clasifica con `--episode`
- **THEN** SHALL error por `stderr` con código `2`

### Requirement: Un fichero sin cues de SRT no es clasificable

`first_cue`/`classify` SHALL rechazar como inválido (código `2`) cualquier fichero que no
contenga al menos un cue con línea de tiempos (`-->`), en vez de darlo por `raw`.

#### Scenario: fichero sin cues
- **GIVEN** un fichero de texto suelto sin ninguna línea `-->`
- **WHEN** se clasifica con `--srt`
- **THEN** SHALL error por `stderr` con código `2`

### Requirement: Salida de texto: una línea por SRT y `OK`/`FALLO` con `--expect`

Sin `--expect`, `srt-stage` SHALL imprimir una línea por SRT con el formato
`<ruta><TAB><etapa>` y salir con `0`. Con `--expect <etapa>`, además SHALL imprimir una
última línea `OK` (si el conjunto contiene la etapa) o `FALLO` (si no).

#### Scenario: sin expect
- **GIVEN** un SRT clasificado como `final`
- **WHEN** se ejecuta sin `--expect`
- **THEN** SHALL imprimirse `<ruta>\tfinal` y salir con `0`

#### Scenario: expect presente
- **GIVEN** un SRT `final` y `--expect final`
- **WHEN** se ejecuta
- **THEN** SHALL imprimirse la línea del SRT y luego `OK`, con salida `0`

### Requirement: Códigos de salida de contrato

`srt-stage` SHALL usar estos códigos:

- `0` éxito (y, con `--expect`, la etapa esperada presente).
- `2` perfil/SRT/argumentos ilegibles, no encontrados o inválidos (perfil inexistente,
  perfil sin `episodes.srt_stages`, `detect` desconocido, `--expect` de una etapa no
  declarada, campo del intro inválido, fichero sin cues, SRT no encontrado, argumentos
  excluyentes o ausentes).
- `4` con `--episode`: ningún SRT del episodio es de la etapa esperada.
- `5` con `--srt`: el fichero se clasificó pero no es la etapa esperada.

#### Scenario: etapa equivocada con `--srt`
- **GIVEN** un SRT `raw` y `--expect final`
- **WHEN** se ejecuta
- **THEN** SHALL imprimirse `FALLO`, el error por `stderr` y salir con `5`

#### Scenario: ninguna etapa esperada entre los SRT del episodio
- **GIVEN** un episodio sin ningún SRT `final` y `--expect final --episode 22`
- **WHEN** se ejecuta
- **THEN** SHALL imprimirse `FALLO`, el error por `stderr` y salir con `4`

#### Scenario: expect de una etapa no declarada
- **GIVEN** `--expect final` sobre un perfil cuyo `srt_stages` solo declara `raw`
- **WHEN** se ejecuta
- **THEN** SHALL error por `stderr` con código `2`

### Requirement: `srt-stage` resuelve el perfil sin exigir `tracker`

`srt-stage` SHALL resolver `shows/<slug>.yaml` con la cascada de directorios del binario
(`PODTRACK_SHOWS_DIR` → junto al ejecutable → `~/.config/podtrack/shows/`) y SHALL leer
`episodes.root`, `episodes.srt_glob` y `episodes.srt_stages` **sin exigir la clave
`tracker`**. Un perfil con `tracker: none` SHALL ser válido para clasificar.

#### Scenario: perfil sin tracker
- **GIVEN** un perfil con `tracker: none` y `episodes.srt_stages` válido
- **WHEN** se clasifica un SRT
- **THEN** SHALL clasificarse con normalidad

#### Scenario: perfil inexistente
- **GIVEN** un `--show` sin perfil
- **WHEN** se ejecuta
- **THEN** SHALL error por `stderr` con código `2` listando los perfiles disponibles

### Requirement: `--json` es aditivo

Con `--json`, `srt-stage` SHALL emitir un único documento con `schema_version`, `show`,
la lista de resultados (`path` y `stage`) y, con `--expect`, el resultado (`ok`). Sin
`--json`, la salida de texto NO SHALL cambiar.

#### Scenario: json con expect
- **GIVEN** `srt-stage --srt x.srt --expect final --json`
- **WHEN** se ejecuta
- **THEN** `stdout` SHALL ser un único JSON válido con los resultados y el resultado de la expectativa
