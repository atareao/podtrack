# PodTrack — Especificación de requisitos (SRS)

> Documento vivo. Manda sobre `AGENTS.md`. Para el **comportamiento**, mandan el binario y su suite de tests; §2 conserva la historia del oráculo retirado.

## 1. Alcance

`podtrack` es el **rastreador de la parrilla y del ciclo de vida de los episodios** de un podcast, empaquetado como binario único en Rust. Sustituyó a `podcast-tracker.py`, un script Python de ~1800 líneas que usaban los agentes del pipeline de producción (guion → grabación → publicación) y que hoy ya no existe.

El producto persiguió **dos objetivos en tensión**, y este SRS los ordenó:

1. **Paridad exacta** con el script Python: mismas rutas, mismos subcomandos, misma salida de texto, mismos *exit codes*, mismo formato de YAML. Ese criterio histórico se conserva hoy como **golden congelado** en `tests/golden/`, que **no se regenera**.
2. **Modernización aditiva**: una salida `--json` *opt-in* para que los agentes dejen de raspar texto, y un binario sin dependencia de intérprete.

La paridad mandó como criterio de aceptación, y la modernización no puede romper a ningún consumidor.

**El objetivo final era desterrar Python.** El script `podcast-tracker.py` fue el *andaime* que garantizó la paridad durante la migración, y se **borró** junto con su suite al quedar sustituido por completo. Un podtrack que conviviera para siempre con el Python habría sido un fracaso del proyecto. El binario absorbió también la **clasificación de etapas del SRT** (`srt-stage.py`), de modo que la skill del podcast quedó sin un solo fichero Python (fase `srt-stage`). Se borraron los tres `.py` y el objetivo —desterrar Python— **está cumplido** (§12).

### Qué NO es

- No es un cliente de podcasts: no hay feeds, descargas ni reproducción (eso es `podcli`).
- No es un generador de web ni de feeds (eso es `aopodcast` / `iapodcast`).
- No produce contenido: no escribe guiones, artículos ni descripciones. Solo gobierna **metadatos y layout**.

## 2. Fuentes de verdad (históricas y vivas)

El port ya terminó. Sus fuentes **históricas** fueron el oráculo Python y su suite, hoy borrados; de ellos queda la evidencia del **golden congelado**. Las fuentes **vivas** de hoy son las especificaciones del dominio y el esquema del perfil.

| Fuente | Rango |
|---|---|
| `~/.agents/skills/podcast-srt/podcast-tracker.py` | **Oráculo histórico (retirado).** Fijó salida, exit codes, formato YAML e invariantes. Borrado en la fase `retire-python` (§12). |
| `tests/golden/` | **Golden congelado del oráculo**, grabado una sola vez desde el histórico y **no regenerable**. Es la única evidencia viva de su comportamiento: **se lee, nunca se reescribe**. |
| `openspec/specs/` (capacidades fundidas: `tracker-read`, `tracker-write`, `srt-stage`, `python-retirement`, …) | Capacidades del dominio ya especificadas y verificadas. |
| `~/.agents/openspec/specs/{episode-identity,episode-material-layout,episode-scheduling}/spec.md` | Capacidades del dominio ya especificadas. |
| `~/.agents/skills/podcast-srt/shows/README.md` | Esquema canónico del perfil de show. |
| `~/.agents/skills/podcast-srt/SKILL.md` | Documentación de subcomandos, ya sobre el binario. |

**Durante el port, ante cualquier ambigüedad decidía el Python.** Cuando el port decidió *de otra forma* a propósito, se documentó como decisión explícita (§9) y se marcó en el spec como divergencia deliberada. Hoy la fuente de verdad es el propio podtrack: su suite, el golden de `tests/golden/` y sus specs de OpenSpec.

## 3. Multi-show y resolución del perfil

Todo se resuelve por show. `--show <slug>` es **obligatorio en todas las invocaciones** salvo que exista un show por defecto.

- El perfil vive en `shows/<slug>.yaml`, **junto al binario** (mismo criterio que hoy junto al script), y declara `tracker:` (ruta del YAML de estado), `episodes.*` (generación, layout, temporadas, material) y el nombre del show.
- La ruta del `tracker:` se resuelve siempre a **absoluta y normalizada léxicamente** (`profile-absolute-paths`): si el perfil la declara absoluta se usa tal cual; si la declara **relativa**, se resuelve contra el **directorio del propio fichero de perfil**. De ahí sale `<scripts.root>` —su directorio padre—, y por tanto las rutas de material de `status`, `get` y `where` también son absolutas, declare el perfil lo que declare.
- El valor de `tracker:` se interpreta **recortado de espacios envolventes** antes de resolverlo: un `tracker: " /ruta/tracker.yaml "` entrecomillado resuelve a la misma ruta que `tracker: /ruta/tracker.yaml`. Un `tracker:` que quede vacío o valga `none` tras el recorte se trata como ausente (`slug-str-and-tracker-trim`).
- **Sin `--show`** se usa el show por defecto. En el Python se llama `atareao-con-linux`; aquí debe poder configurarse (ver §9, decisión D1).
- El esquema del perfil está en `shows/README.md` y **no se reinterpreta**: si el perfil está mal, el binario falla con un error claro, no improvisa.
- Los fallos de resolución del perfil se reportan con una causa **disjunta**: perfil **ilegible** (no se pudo leer el fichero) —«No se pudo leer el perfil»—, perfil **mal formado** (el YAML no parsea) —«no es válido»— y ruta del **tracker irresoluble** —«no se pudo resolver la ruta del tracker»—. Los tres salen con el código de perfil inválido (`2`) (`profile-error-taxonomy`).

## 4. Modelo de datos

### 4.1 El tracker (YAML de estado)

Estructura exacta:

```yaml
season: 9
season_start: 2026-09-03
episode_duration_minutes: 25
publish_days: [Monday, Thursday]
episodes:
  - number: 843
    slug: la-memoria-de-valet
    season: 9
    date: 2026-10-26
    title: "La Memoria de Valet"
    status: scripted
```

### 4.2 La entidad `Episode`

| Campo | Tipo | Invariante |
|---|---|---|
| `number` | `Option<u32>` | `null` mientras no publicado. Provisional (informativo) si el episodio no está publicado; **autoritativo y congelado** si lo está. |
| `slug` | `Option<Slug>` | kebab-case, identidad estable. **No** participa en el número. La lectura canónica del slug (texto y `--json`) lo expone recortado y no vacío, o ausente (`slug-str-and-tracker-trim`). |
| `season` | `u16` | Se deduce de `date` y del calendario de temporadas si falta. |
| `date` | `Date` | ISO `YYYY-MM-DD`. `TBD` o vacío van al final del orden. |
| `title` | `String` | Libre. |
| `status` | `Status` | `scripted` \| `recorded` \| `published`. |

### 4.3 Orden

Por `date` ascendente; `number` solo desempata. **El número no ordena la parrilla.**

## 5. Invariantes (no negociables)

1. **La clave es el `slug`**, no el número. Ningún flujo debe exigir renumerar para insertar o mover.
2. **El número se asigna al publicar** con `publish`: `max(publicados) + 1`, y queda **congelado**. Nunca se reutiliza.
3. `publish` **falla** si el episodio ya estaba publicado.
4. Un no publicado puede llevar un número **provisional** (informativo); `publish` lo **sobrescribe** y, si otro no publicado ocupaba ese número, se le anula (`number: null`).
5. `add` sin número asigna el **siguiente hueco provisional** de parrilla; el `slug` se deriva del título si no se pasa.
6. La **fecha de publicación** cae en los días declarados por el perfil (lunes y jueves por defecto) y no se puede fijar a un día no publicable (`move` lo **rechaza**).
7. Toda escritura hace **backup** (`.yaml.bak`) antes de tocar el fichero y es **atómica** (escritura a temporal + rename).

## 6. Superficie CLI

`podtrack <subcomando> [args] --show <slug> [--dry-run]`

| Subcomando | Etapa | Qué hace |
|---|---|---|
| `status` | lectura | Resumen: temporada, totales por estado, último publicado, **«Siguiente a publicar»** (con slug y ruta del material), hueco para uno nuevo. |
| `next` | lectura | Hueco provisional («Siguiente episodio») y número autoritativo («Próximo a publicar»). |
| `next-date` | lectura | Próxima fecha publicable. |
| `list` / `list-range` | lectura | Listado filtrable (`--status`, `--season`, `--last`). Marca no publicados con `*` + leyenda. |
| `get` | lectura | Detalle de un episodio (por número o slug), con su `slug` y la ruta del material. |
| `where` | lectura | Ruta del material de guion de un episodio (por slug/número), sin crear nada. |
| `open` | lectura | Abre la presentación Typst del episodio (por número o slug) con typshow, sin bloquear; `--dry-run` resuelve la ruta sin lanzar. |
| `add` | escritura | Alta (con o sin número). |
| `update` | escritura | Cambia estado (`scripted`/`recorded`/`published` — este último solo vía `publish`). |
| `publish` | escritura | Estampa el número autoritativo y lo congela. |
| `insert` | reordenar | Inserta antes/después de otro, sin renumerar a mano. |
| `move` | reordenar | Cambia la fecha de un episodio. |
| `renumber` | cirugía | Renumera un rango con delta. |
| `rename-material` | cirugía | Renombra el material derivado (`0NNN`/`NNN`/`eNNN`/`episodio-NNN`) tras renumerar. |
| `backfill-slugs` | cirugía | Rellena `slug` donde falte. |
| `migrate-material` | cirugía | Migra el material a la carpeta de la regla única del perfil (por defecto, `{slug}`) (colisión-segura; solo no publicados por defecto). |
| `rebase-references` | cirugía | Reescribe referencias estructurales entre guiones. Resuelve la carpeta con la regla única y salta los episodios sin carpeta resoluble. |

`insert`, `move`, `renumber`, `rename-material`, `backfill-slugs`, `migrate-material`, `rebase-references` y `open` aceptan `--dry-run`.

### 6.1 Resolución de la carpeta de material

La carpeta de material de un episodio tiene **una sola definición** en el binario
(`ResolvedShow::material_dir`, en `src/profile.rs`): `<scripts.root>/<episodes.scripts.dir
renderizado>`. El `dir` admite `{slug}`, `{n}` y `{n:0Nd}`; si el perfil no lo declara (o lo declara
vacío) vale `{slug}`. La usan por igual las superficies de lectura (`where`, `get`, `status` y sus
documentos `--json`), `open` y las cirugías que escriben material (`migrate-material`,
`rebase-references`), de modo que lo que se informa y lo que se escribe no pueden divergir.

Un `dir` que, ya renderizado, sea **absoluto** o contenga `..` es un error de configuración
(`MaterialDirNotRelative`, código de salida **2**): la raíz de material es una frontera, no una
convención. Y un episodio **sin `slug`** no tiene ruta de material en ninguna superficie: el texto
muestra `—` (en `status` se omite la línea), el JSON emite `null`, `where` lo trata como caso
tratado y `rebase-references` lo salta.

## 7. Contrato de salida

### 7.1 Texto (por defecto)

La salida de texto es un **contrato con los agentes** del pipeline. El port debe reproducirla **byte a byte**: iconos, alineación de columnas, leyendas (`* número provisional (se fija al publicar)`), bloques (`▶ Siguiente a publicar:`) y mensajes de confirmación (`✓ Tracker actualizado: <ruta>`).

Los *exit codes* también son contrato: `0` éxito, `!= 0` fallo, con el mensaje de error en `stderr`.

### 7.2 JSON (`--json`, opt-in)

`--json` es **aditivo y opt-in**. No cambia la salida por defecto. Emite a `stdout` un objeto JSON estable y versionado, para que los agentes consuman estructura en vez de texto:

```json
{
  "schema_version": 1,
  "show": "atareao-con-linux",
  "next_to_publish": { "number": 843, "slug": "la-memoria-de-valet", "date": "2026-10-26", "material": "/data/notas/Podcasts/guiones/la-memoria-de-valet" }
}
```

En los comandos de escritura, `--json` reporta la operación y el antes/después. **Ningún agente migra a `--json` en la fase de paridad**: se añade para poder migrarlos después, uno a uno.

## 8. Formato del tracker: writer canónico

El fichero se reescribe entero en cada operación, con **formato fijo** (no round-trip de YAML genérico):

- cabecera de comentarios literal (`# podcast-tracker.yaml`, `# Estado del podcast "<show>"`, …);
- orden fijo: `season`, `season_start`, `episode_duration_minutes`, `publish_days`, `episodes`;
- cada episodio: `number`, `slug` (solo si existe), `season`, `date`, `title` (entrecomillado), `status`;
- episodios ordenados por fecha antes de escribir.

Esto elimina el mayor riesgo de un port a Rust (que `serde_yaml` reordene claves o pierda comentarios): **el formato ya está canonizado en origen**. El writer se reimplementa a mano, sin `serde` para escribir.

## 9. Decisiones de diseño (cerradas)

| ID | Decisión | Estado |
|---|---|---|
| D1 | ¿Cómo se declara el show por defecto sin `--show`? (variable de entorno / fichero de config / constante compilada) | **Cerrada** — variable de entorno `PODTRACK_SHOW`; sin ella, `atareao-con-linux` compilado (`DEFAULT_SHOW`). |
| D2 | ¿Dónde busca el binario los `shows/*.yaml`? ¿Junto al binario, vía `XDG_CONFIG_HOME`, o ruta configurable? | **Cerrada** — resolutor en cascada: `PODTRACK_SHOWS_DIR` → directorio junto al ejecutable → `~/.config/podtrack/shows/`. |
| D3 | ¿Se mantiene `--dry-run` con semántica idéntica (no escribe nada, imprime qué haría)? | **Cerrada** — sí: `--dry-run` no escribe nada e imprime qué haría. |
| D4 | ¿El `--json` se publica ya en `bootstrap-core` para los comandos de lectura, o al final? | **Cerrada** — en `bootstrap-core`, para los comandos de lectura (`schema_version: 1`). |

## 10. No-objetivos

- **No** se toca el material de guion ni el contenido del tracker (los tests corren sobre fixtures temporales; jamás sobre `/data/notas`).
- **No** se reescribe el esquema del perfil de show.
- **No** se reapuntan los agentes hasta que las tres primeras fases estén verdes.
- **No** se retiene la publicación del crate: la guarda `publish = false` se retiró al preparar los metadatos de publicación, y desde entonces cada release publica en crates.io (última: `v0.5.1`).
- **No** hay TUI ni servidor: solo CLI.

## 11. Criterios de aceptación (global)

1. La suite de paridad —construida sobre el **golden del oráculo retirado** (`tests/golden/`, no regenerable)— pasa en verde.
2. Para cada subcomando, la salida de texto coincide con la del **golden congelado** sobre el mismo fixture (comparación por `diff`).
3. `--json` produce JSON válido y estable, sin afectar a la salida por defecto.
4. `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` y `cargo doc` limpios.
5. Ningún test ni ejecución real toca `/data/notas`.

## 12. Retirada de Python (fase `retire-python`)

El port no termina cuando el binario funciona: termina cuando **no queda Python**. El objetivo es **Python cero** en `~/.agents/skills/podcast-srt/`.

La skill tenía exactamente tres ficheros Python, y **los tres cayeron**:

| Fichero | Lo sustituye | Fase de port |
|---|---|---|
| `podcast-tracker.py` | los subcomandos de la parrilla | `bootstrap-core` + `write-commands` + `migration-surgery` |
| `podcast-tracker.test.py` | la suite de paridad del binario | `bootstrap-core` (port) |
| `srt-stage.py` | el subcomando de clasificación de etapas del SRT | `srt-stage` |

Pasos de la fase (todos completados):

1. Portar `srt-stage.py` al binario (fase `srt-stage`) con paridad de contrato, antes de tocar nada.
2. Conmutar el `SKILL.md` de `podcast-srt` y los agentes que invocaban los scripts (planificador, coordinador, procesador de capítulos y notas, como mínimo) para llamar a `podtrack`.
3. Verificar en producción un ciclo completo (una publicación real) sin Python.
4. **Borrar** los tres `.py`, limpiar `__pycache__`/`.ruff_cache` y la configuración huérfana de `pytest`/`ruff`.
5. Dejar constancia en `~/.agents/openspec` de que las capacidades pasaron al binario.

**Estado: cerrada.** No queda **ningún `.py`** en `~/.agents/skills/podcast-srt/`: quedan `shows/`,
`SKILL.md` y dos copias de seguridad de este último, `SKILL.md.bak-puntos` y `SKILL.md.bak-story`;
los agentes del pipeline invocan `podtrack`. La constancia del traspaso de capacidades vive en
`~/.agents/openspec/specs/podtrack-tooling/spec.md`.

**Criterio de cierre:** `grep -r '\.py' ~/.agents/skills/podcast-srt/` no devuelve nada.
