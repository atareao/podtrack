# Spec Delta

## Purpose

Definir el invariante de **Python cero** en la skill `podcast-srt`: que no quede
ningún `.py`, que el tooling se invoque como `podtrack` y que el binario esté
instalado de forma invocable.

## ADDED Requirements

### Requirement: La skill `podcast-srt` no contiene ficheros Python

El árbol `~/.agents/skills/podcast-srt/` NO SHALL contener ningún fichero `.py`, ni
el andamiaje que los acompaña (`pytest.ini`, `__pycache__/`, `.pytest_cache/`,
`.ruff_cache/`).

#### Scenario: criterio de cierre mecánico
- **GIVEN** la skill `podcast-srt` tras la retirada
- **WHEN** se ejecuta `grep -r '\.py' ~/.agents/skills/podcast-srt/`
- **THEN** NO SHALL devolver ninguna coincidencia

#### Scenario: el andamiaje también desaparece
- **GIVEN** la skill tras la retirada
- **WHEN** se listan sus ficheros
- **THEN** NO SHALL existir `podcast-tracker.py`, `podcast-tracker.test.py`, `srt-stage.py`, `pytest.ini` ni caches de `pytest`/`ruff`

### Requirement: El tooling se invoca como `podtrack`

El `SKILL.md` de `podcast-srt` y los agentes del podcast SHALL invocar el binario
`podtrack` en lugar de `python3 …/{podcast-tracker,srt-stage}.py`. La bandera
`--show <slug>` SHALL seguir usándose como hoy (bandera global, antes o después del
subcomando).

#### Scenario: el SKILL usa el binario
- **GIVEN** el `SKILL.md` tras la conmutación
- **WHEN** se buscan invocaciones de los scripts
- **THEN** NO SHALL aparecer `python3 …/podcast-tracker.py` ni `python3 …/srt-stage.py`
- **AND** las invocaciones SHALL ser `podtrack <subcomando>` y `podtrack srt-stage`

#### Scenario: los agentes usan el binario
- **GIVEN** los agentes del podcast tras la conmutación
- **WHEN** se buscan invocaciones de los scripts
- **THEN** NO SHALL aparecer ninguna invocación `python3 …/*.py`
- **AND** SHALL invocar `podtrack`

### Requirement: El binario está instalado e invocable, y encuentra los perfiles

`podtrack` SHALL instalarse con `cargo install --path . --locked`, invocable como
`podtrack` desde el entorno de los agentes sin ruta absoluta al árbol de trabajo.
El directorio de perfiles de la skill SHALL quedar accesible por la cascada del
binario (enlace en `~/.config/podtrack/shows`), de modo que `podtrack --show <slug>`
encuentre `shows/<slug>.yaml` sin variables de entorno.

#### Scenario: invocación directa
- **GIVEN** el binario instalado
- **WHEN** se ejecuta `podtrack --version`
- **THEN** SHALL responder con la versión, sin `python3`

#### Scenario: resuelve el perfil por la cascada
- **GIVEN** los perfiles de la skill enlazados en `~/.config/podtrack/shows`
- **WHEN** se ejecuta `podtrack status --show atareao-con-linux`
- **THEN** SHALL mostrar el estado del tracker sin `PODTRACK_SHOWS_DIR`

### Requirement: Constancia del traslado de capacidades

La retirada SHALL dejar constancia en `~/.agents/openspec` de que las capacidades
que vivían en los scripts (parrilla, cirugía de material, clasificación de etapas
del SRT) residen ahora en el binario `podtrack`.

#### Scenario: la constancia existe
- **GIVEN** el repo `~/.agents`
- **WHEN** se revisa su `openspec`
- **THEN** SHALL existir constancia de que las capacidades pasaron a `podtrack`
