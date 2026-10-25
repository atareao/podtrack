# Design

## Context

El proyecto arranca vacío: solo el esqueleto (crate, `.justfile`, CI, OpenSpec). La implementación de referencia está en `~/.agents/skills/podcast-srt/`, con su oráculo (`podcast-tracker.py`), su suite (~119 escenarios) y el esquema del perfil de show. Este change cubre la **infraestructura + la superficie de lectura**; deja escritura y cirugía para changes posteriores.

Restricciones que dan forma al diseño:

- La salida de texto es **contrato con nueve agentes**: debe ser byte a byte idéntica.
- Los tests **nunca** pueden tocar `/data/notas`.
- El fichero de estado se reescribe entero en cada operación, así que su formato debe ser reproducible de forma exacta.

## Goals / Non-Goals

**Goals:**

- Paridad byte a byte de la superficie de lectura (texto + *exit codes*).
- Modelo tipado que haga estructurales los invariantes del dominio (número al publicar, slug como identidad, orden por fecha).
- Escritura canónica fiable y atómica, lista para las fases de escritura.
- `--json` estable y versionado, sin tocar la salida por defecto.

**Non-Goals:**

- Los comandos de escritura y de cirugía (changes siguientes).
- Reapuntar los agentes al binario (solo cuando las tres fases estén verdes).
- Publicar el crate (`publish = false`).
- Cualquier TUI, servidor o cliente de feeds.

## Decisions

### D1. El *writer* del YAML se escribe a mano, no con `serde_yaml`

`serde_yaml` no preserva comentarios y puede reordenar claves. El formato del tracker (cabecera comentada, orden fijo de campos) se perdería. La observación clave: **el Python ya canoniza el formato al escribir** — no hace *round-trip*, escribe un formato fijo. Portar eso a mano es trivial y garantiza paridad. `serde` se usa **solo para leer**.

*Alternativas:* `serde_yaml` para escribir (pierde comentarios/orden); un YAML con preservación de *trivia* (complejidad innecesaria para un formato que ya es fijo).

### D2. `clap` (derive) para el CLI, con paridad de ayuda y errores

`clap` cubre los ~17 subcomandos y sus banderas (`--show`, `--dry-run`, `--status`, `--season`, `--last`). El contrato que importa no es el `--help` sino la **salida de los comandos** y los *exit codes*; el `--help` puede diferir del de `argparse`.

### D3. Fechas con una librería tipada

`Date` tipada (no `String`) para el modelo; el serializador convierte a `YYYY-MM-DD`. Se admite `TBD`/vacío como estado «sin fecha» que ordena al final, igual que el Python.

### D4. Errores tipados, sin `unwrap` en producción

`thiserror` para el error del dominio (perfil inválido, tracker ausente, episodio inexistente, estado no válido) y `anyhow` en el borde (`main`). Prohibido `unwrap()`/`expect()`/`panic!()` en código de producción. Los *exit codes* del Python se reproducen exactamente.

### D5. Resolución del perfil en cascada

`--show` → busca `shows/<slug>.yaml` en: `PODTRACK_SHOWS_DIR` (env) → junto al ejecutable → `~/.config/podtrack/shows/`. Cierra la decisión **D2 del SRS**. El show por defecto se toma de `PODTRACK_SHOW` (env) y, si no, `atareao-con-linux` (**D1 del SRS**).

### D6. Fixtures temporales, jamás el árbol real

Cada test construye un `shows/` y un tracker en un directorio temporal (`tempfile`). Esto hace imposible un accidente sobre `/data/notas` y permite fijar el contrato con datos controlados.

## Divergencias conscientes con el oráculo

`podtrack` reproduce byte a byte la salida del oráculo, **salvo** en los puntos siguientes, todos deliberados y anclados en un `SHALL` del spec o en una decisión de diseño. Es la lista que el revisor puede usar como referencia.

| # | Comportamiento | Oráculo | `podtrack` | Ancla |
|---|---|---|---|---|
| 1 | `status` de un episodio con valor desconocido (p. ej. `cancelled`) | Lo tolera y lo cuenta como `Sin estado` | **Error de lectura** (exit 1) | spec `tracker-store`: «un `status` fuera de ese conjunto SHALL ser un error» |
| 2 | Perfil por defecto ausente | Cae en silencio a una ruta legacy | **Error** nombrando la causa | spec `show-profile`: perfil inválido = error |
| 3 | Perfil sin la clave `tracker` | Lo funde con `tracker: none` (exit 3) | **Error tipado** (exit 2) nombrando la clave | spec `show-profile`: error diciendo qué clave falta |
| 4 | Escritura del tracker | *In place* | **Atómica** (temp + rename) + backup `.bak` | spec `tracker-store`: backup y atomicidad |
| 5 | Resolución de episodio | Acepta dígitos Unicode no ASCII (`isdigit()`) y el objetivo vacío casa con un slug vacío | Solo `ASCII`/número y slug no vacío | bordes exóticos; se priorizan los datos canónicos |
| 6 | Errores de argumentos del CLI | `argparse` | `clap` (mensaje y exit distintos) | decisión D2 |
| 7 | `next-date` | Días de publicación fijos lunes/jueves; no consulta el perfil | Idéntico | spec `tracker-read` (días del show por defecto) |

> **El punto 1 no rompe la paridad de datos canónicos:** el enum cierra los **cuatro** valores reales (`pending`, `scripted`, `recorded`, `published`); solo diverge ante un valor que el esquema del perfil no admite.

## Risks / Trade-offs

- **Deriva de paridad.** El Python puede cambiar mientras se porta. *Mitigación:* la suite de paridad se porta en el mismo change y se corre contra fixtures congelados; el `diff` de salidas es el árbitro.
- **`--json` como contrato prematuro.** Congelar un esquema joven duele. *Mitigación:* se marca `schema_version: 1` y es *opt-in*; ningún consumidor depende de él en esta fase.
- **Backend YAML.** `serde_yaml` está en desuso en favor de `serde_yml`. *Mitigación:* la decisión queda abierta a una dependencia concreta en la tarea 0, resolviéndola contra la caché local de `~/.cargo/registry`.
- **Duplicación temporal.** Mientras convivan Python y Rust hay dos fuentes de verdad del comportamiento. *Mitigación:* el Python sigue siendo el oráculo hasta la conmutación final; el Rust no se usa en producción hasta entonces.
