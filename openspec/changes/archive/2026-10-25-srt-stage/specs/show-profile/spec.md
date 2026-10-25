# Spec Delta

## Purpose

Documentar la lectura del perfil **sin exigir `tracker`** (necesaria para `srt-stage`,
cuyo show de referencia declara `tracker: none`) y los campos de `episodes.srt_stages`
que consume el clasificador.

## ADDED Requirements

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
