# PodTrack — Especificación de requisitos (SRS)

> Documento vivo. Manda sobre `AGENTS.md`. Para el **comportamiento**, manda la implementación de referencia (§2).

## 1. Alcance

`podtrack` es el **rastreador de la parrilla y del ciclo de vida de los episodios** de un podcast, empaquetado como binario único en Rust. Sustituye a `podcast-tracker.py`, un script Python de ~1800 líneas que hoy usan los agentes del pipeline de producción (guion → grabación → publicación).

El producto tiene **dos objetivos en tensión**, y este SRS los ordena:

1. **Paridad exacta** con el script Python: mismas rutas, mismos subcomandos, misma salida de texto, mismos *exit codes*, mismo formato de YAML.
2. **Modernización aditiva**: una salida `--json` *opt-in* para que los agentes dejen de raspar texto, y un binario sin dependencia de intérprete.

La paridad manda. La modernización no puede romper a ningún consumidor.

**El objetivo final es desterrar Python.** El script `podcast-tracker.py` es el *andaime* que garantiza la paridad durante la migración, y su destino es **ser borrado** junto con su suite, en cuanto el binario lo sustituya por completo. Un podtrack que conviva para siempre con el Python es un fracaso del proyecto. El binario absorbe también la **clasificación de etapas del SRT** (`srt-stage.py`), para que la skill del podcast pueda quedarse sin un solo fichero Python (fase `srt-stage`).

### Qué NO es

- No es un cliente de podcasts: no hay feeds, descargas ni reproducción (eso es `podcli`).
- No es un generador de web ni de feeds (eso es `aopodcast` / `iapodcast`).
- No produce contenido: no escribe guiones, artículos ni descripciones. Solo gobierna **metadatos y layout**.

## 2. Fuentes de verdad durante el port (andaime a derribar)

> Estas fuentes son **temporales**: se consultan para *portar*, no para *depender*. Cuando el port esté completo y los agentes conmutados, `podcast-tracker.py` y `podcast-tracker.test.py` se borran (§12).

| Fuente | Rango |
|---|---|
| `~/.agents/skills/podcast-srt/podcast-tracker.py` | **Oráculo supremo *mientras dure el port*.** Salida, exit codes, formato YAML, invariantes. **Se borra al terminar.** |
| `~/.agents/skills/podcast-srt/podcast-tracker.test.py` | ~119 escenarios → se portan 1:1 como tests de integración de paridad. |
| `~/.agents/openspec/specs/{episode-identity,episode-material-layout,episode-scheduling}/spec.md` | Capacidades del dominio ya especificadas. |
| `~/.agents/skills/podcast-srt/shows/README.md` | Esquema canónico del perfil de show. |
| `~/.agents/skills/podcast-srt/SKILL.md` | Documentación de subcomandos. |

**Ante cualquier ambigüedad *durante el port*, decide el Python.** Cuando el port decida *de otra forma* a propósito, se documenta como decisión explícita (§9) y se marca en el spec como divergencia deliberada. Pasado el port, la fuente de verdad es el propio podtrack (y sus specs de OpenSpec).

## 3. Multi-show y resolución del perfil

Todo se resuelve por show. `--show <slug>` es **obligatorio en todas las invocaciones** salvo que exista un show por defecto.

- El perfil vive en `shows/<slug>.yaml`, **junto al binario** (mismo criterio que hoy junto al script), y declara `tracker:` (ruta del YAML de estado), `episodes.*` (generación, layout, temporadas, material) y el nombre del show.
- La ruta del `tracker:` se resuelve siempre a **absoluta y normalizada léxicamente** (`profile-absolute-paths`): si el perfil la declara absoluta se usa tal cual; si la declara **relativa**, se resuelve contra el **directorio del propio fichero de perfil**. De ahí sale `<scripts.root>` —su directorio padre—, y por tanto las rutas de material de `status`, `get` y `where` también son absolutas, declare el perfil lo que declare.
- El valor de `tracker:` se interpreta **recortado de espacios envolventes** antes de resolverlo: un `tracker: " /ruta/tracker.yaml "` entrecomillado resuelve a la misma ruta que `tracker: /ruta/tracker.yaml`. Un `tracker:` que quede vacío o valga `none` tras el recorte se trata como ausente (`slug-str-and-tracker-trim`).
- **Sin `--show`** se usa el show por defecto. En el Python se llama `atareao-con-linux`; aquí debe poder configurarse (ver §9, decisión D1).
- El esquema del perfil está en `shows/README.md` y **no se reinterpreta**: si el perfil está mal, el binario falla con un error claro, no improvisa.

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
| `add` | escritura | Alta (con o sin número). |
| `update` | escritura | Cambia estado (`scripted`/`recorded`/`published` — este último solo vía `publish`). |
| `publish` | escritura | Estampa el número autoritativo y lo congela. |
| `insert` | reordenar | Inserta antes/después de otro, sin renumerar a mano. |
| `move` | reordenar | Cambia la fecha de un episodio. |
| `renumber` | cirugía | Renumera un rango con delta. |
| `rename-material` | cirugía | Renombra el material derivado (`0NNN`/`NNN`/`eNNN`/`episodio-NNN`) tras renumerar. |
| `backfill-slugs` | cirugía | Rellena `slug` donde falte. |
| `migrate-material` | cirugía | Migra el material al layout por slug (colisión-segura; solo no publicados por defecto). |
| `rebase-references` | cirugía | Reescribe referencias estructurales entre guiones. |

`insert`, `move`, `renumber`, `rename-material`, `backfill-slugs`, `migrate-material` y `rebase-references` aceptan `--dry-run`.

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

## 9. Decisiones abiertas

| ID | Decisión | Estado |
|---|---|---|
| D1 | ¿Cómo se declara el show por defecto sin `--show`? (variable de entorno / fichero de config / constante compilada) | Propuesta: variable de entorno `PODTRACK_SHOW`, con `atareao-con-linux` como último recurso. |
| D2 | ¿Dónde busca el binario los `shows/*.yaml`? ¿Junto al binario, vía `XDG_CONFIG_HOME`, o ruta configurable? | Propuesta: resolutor en cascada (env → junto al binario → `~/.config/podtrack/shows/`). |
| D3 | ¿Se mantiene `--dry-run` con semántica idéntica (no escribe nada, imprime qué haría)? | Sí. |
| D4 | ¿El `--json` se publica ya en `bootstrap-core` para los comandos de lectura, o al final? | Propuesta: en `bootstrap-core`, para los de lectura. |

## 10. No-objetivos

- **No** se toca el material de guion ni el contenido del tracker durante el port (los tests corren sobre fixtures temporales; jamás sobre `/data/notas`).
- **No** se reescribe el esquema del perfil de show.
- **No** se reapuntan los agentes hasta que las tres primeras fases estén verdes.
- **No** se publica el crate (`publish = false` hasta que merezca la pena).
- **No** hay TUI ni servidor: solo CLI.

## 11. Criterios de aceptación (global)

1. La suite de paridad (port de los ~119 escenarios) pasa en verde.
2. Para cada subcomando, la salida de texto es **idéntica** a la del Python sobre el mismo fixture (comparación por `diff`).
3. `--json` produce JSON válido y estable, sin afectar a la salida por defecto.
4. `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` y `cargo doc` limpios.
5. Ningún test ni ejecución real toca `/data/notas`.

## 12. Retirada de Python (fase `retire-python`)

El port no termina cuando el binario funciona: termina cuando **no queda Python**. El objetivo es **Python cero** en `~/.agents/skills/podcast-srt/`.

La skill tiene hoy exactamente tres ficheros Python, y **los tres caen**:

| Fichero | Lo sustituye | Fase de port |
|---|---|---|
| `podcast-tracker.py` | los subcomandos de la parrilla | `bootstrap-core` + `write-commands` + `migration-surgery` |
| `podcast-tracker.test.py` | la suite de paridad del binario | `bootstrap-core` (port) |
| `srt-stage.py` | el subcomando de clasificación de etapas del SRT | `srt-stage` |

Pasos de la fase:

1. Portar `srt-stage.py` al binario (fase `srt-stage`) con paridad de contrato, antes de tocar nada.
2. Conmutar el `SKILL.md` de `podcast-srt` y los agentes que hoy invocan los scripts (planificador, coordinador, procesador de capítulos y notas, como mínimo) para llamar a `podtrack`.
3. Verificar en producción un ciclo completo (una publicación real) sin Python.
4. **Borrar** los tres `.py`, limpiar `__pycache__`/`.ruff_cache` y la configuración huérfana de `pytest`/`ruff`.
5. Dejar constancia en `~/.agents/openspec` de que las capacidades pasaron al binario.

**Criterio de cierre:** `grep -r '\.py' ~/.agents/skills/podcast-srt/` no devuelve nada.
