# show-profile Specification

## Purpose
Resolver, para cada invocación, **qué podcast** se está manipulando: localizar su perfil y, de ahí, la ruta del tracker de estado, los días de publicación y el calendario de temporadas. Todo el binario depende de esta resolución, y debe fallar alto y claro cuando no puede resolverla.

## Requirements

### Requirement: El flag `--show` selecciona el perfil del podcast

Toda invocación SHALL aceptar `--show <slug>`. El binario SHALL cargar el perfil `shows/<slug>.yaml` correspondiente y resolver de él la ruta del tracker de estado.

#### Scenario: `--show` carga el perfil correcto
- **GIVEN** un perfil válido en `shows/atareao-con-linux.yaml` que declara `tracker: /tmp/x/podcast-tracker.yaml`
- **WHEN** se invoca `podtrack status --show atareao-con-linux`
- **THEN** SHALL leer el tracker de la ruta declarada por ese perfil

#### Scenario: un `--show` inexistente falla
- **GIVEN** que no existe `shows/no-existe.yaml`
- **WHEN** se invoca `podtrack status --show no-existe`
- **THEN** SHALL salir con un código distinto de cero
- **AND** SHALL escribir un error claro en `stderr` nombrando el slug pedido

### Requirement: El perfil se busca en una cascada determinista

Cuando `--show` no es una ruta absoluta, el binario SHALL buscar `shows/<slug>.yaml` en este orden: el directorio de `PODTRACK_SHOWS_DIR` (si está definido y no vacío), el directorio junto al ejecutable, y `~/.config/podtrack/shows/`. SHALL ganar el primero que contenga un perfil válido.

#### Scenario: el directorio del entorno gana a los demás
- **GIVEN** un perfil en `PODTRACK_SHOWS_DIR` y otro del mismo slug junto al ejecutable
- **WHEN** se resuelve el perfil
- **THEN** SHALL cargarse el de `PODTRACK_SHOWS_DIR`

### Requirement: Existe un show por defecto cuando falta `--show`

Si la invocación no trae `--show`, el binario SHALL usar el slug de `PODTRACK_SHOW` y, si esa variable no está definida ni vacía, `atareao-con-linux`.

#### Scenario: sin `--show` se usa el defecto del entorno
- **GIVEN** `PODTRACK_SHOW=mi-podcast`
- **WHEN** se invoca un comando sin `--show`
- **THEN** SHALL cargarse `shows/mi-podcast.yaml`

#### Scenario: sin `--show` ni entorno se usa el defecto compilado
- **GIVEN** un entorno sin `PODTRACK_SHOW`
- **WHEN** se invoca un comando sin `--show`
- **THEN** SHALL usarse el slug `atareao-con-linux`

### Requirement: El esquema del perfil no se reinterpreta

El binario SHALL leer del perfil únicamente los campos que el esquema define (`tracker`, `name`, `episodes.*`) y NO SHALL inventar valores para los campos ausentes que el esquema marca como obligatorios. Un perfil inválido SHALL ser un error, no un modo degradado.

#### Scenario: perfil sin la clave del tracker es un error
- **GIVEN** un perfil al que le falta la clave que declara la ruta del tracker
- **WHEN** se invoca cualquier comando
- **THEN** SHALL salir con un código distinto de cero
- **AND** SHALL indicar en `stderr` qué clave falta en qué perfil

### Requirement: El perfil se puede cargar sin exigir `tracker`

Para los comandos que no operan sobre una parrilla (`srt-stage`), el binario SHALL poder
resolver y parsear `shows/<slug>.yaml` con la misma cascada de directorios
(`PODTRACK_SHOWS_DIR` → junto al ejecutable → `~/.config/podtrack/shows/`) **sin exigir la
clave `tracker`**. Los comandos que sí operan sobre el tracker (todo el ciclo F1–F8)
SHALL seguir exigiendo `tracker` y fallar con su error tipado si falta.

#### Scenario: perfil sin tracker para srt-stage
- **GIVEN** un perfil con `tracker: none` y `episodes.srt_stages`
- **WHEN** se invoca `srt-stage`
- **THEN** el perfil SHALL cargarse y el SRT SHALL clasificarse

#### Scenario: el ciclo del tracker sigue exigiendo tracker
- **GIVEN** un perfil sin clave `tracker`
- **WHEN** se invoca `status` (o cualquier comando de la parrilla)
- **THEN** SHALL fallar con el error tipado de tracker ausente

### Requirement: `episodes.srt_stages` declara las etapas y sus señales

El perfil SHALL declarar las etapas del SRT en `episodes.srt_stages`, como un mapa con
`order` (lista ordenada de nombres) y una entrada por etapa con `label`, `detect`
(`any` | `not_intro` | `starts_with_intro`), `deliverables` y, opcionalmente,
`canonical_name`. Una etapa `detect: starts_with_intro` SHALL declarar además
`intro_pattern` (o `intro_marker`) y, para la señal de bloque, `intro_anchor_chars`,
`intro_block_min_seconds` e `intro_block_max_start`. `srt-stage` SHALL consumir estos
campos.

#### Scenario: un perfil de dos etapas
- **GIVEN** un `srt_stages` con `order: [raw, final]`, `raw` con `detect: not_intro` y `final` con `detect: starts_with_intro` y sus señales
- **WHEN** `srt-stage` carga el perfil
- **THEN** SHALL poder clasificar SRT en `raw` y `final`

#### Scenario: detect desconocido
- **GIVEN** una etapa con `detect: inventado`
- **WHEN** `srt-stage` carga el perfil
- **THEN** SHALL error con código `2` indicando que el `detect` es desconocido
