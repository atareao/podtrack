# Spec Delta

## Purpose

Resolver, para cada invocación, **qué podcast** se está manipulando: localizar su perfil y, de ahí, la ruta del tracker de estado, los días de publicación y el calendario de temporadas. Todo el binario depende de esta resolución, y debe fallar alto y claro cuando no puede resolverla.

## ADDED Requirements

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
