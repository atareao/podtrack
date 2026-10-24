# Proposal

## Why

El estado del podcast —qué episodios existen, cuándo salen, en qué estado están y dónde vive su material— lo gobierna hoy `podcast-tracker.py`, un script Python de ~1800 líneas que solo existe dentro de `~/.agents/skills/podcast-srt/`. Funciona y está cubierto por ~119 tests, pero tiene tres costes que ya pesan:

1. **Runtime ajeno.** Arrastra Python + PyYAML en una máquina donde **nueve agentes** lo invocan muchas veces por episodio, pagando el arranque del intérprete en cada llamada.
2. **Contrato de texto.** Su salida está pensada para ojos humanos y los agentes la *raspan*. No hay forma estructurada de consumirla.
3. **Hogar equivocado.** Una herramienta de este peso no debería vivir dentro de una skill de agentes, sin repo propio, sin CI, sin versionado.

Este change abre el proyecto `podtrack` con su **primera fase**: la infraestructura tipada (modelo, resolución de show, escritura canónica del YAML) y el **port de paridad de la superficie de solo lectura**, más una salida `--json` *opt-in*. Se empieza por la lectura a propósito: es la mitad del contrato, **no toca el material real** y permite validar la paridad byte a byte con riesgo nulo. Las fases de escritura y de cirugía vienen en changes posteriores.

## What Changes

1. **Nace el crate `podtrack`** (binario único, edition 2024). El esqueleto, el tooling y el CI ya existen; aquí se llena de comportamiento.

2. **Modelo tipado del dominio.** `Episode { number: Option<u32>, slug: Option<Slug>, season: u16, date: Date, title: String, status: Status }`, con `Status` como enum cerrado (`scripted` | `recorded` | `published`), `Slug` como *newtype* kebab-case validado, y el orden por `date` con `number` solo como desempate. El invariante «el número no ordena la parrilla» se vuelve estructural.

3. **Resolución de show.** `--show <slug>` resuelve el perfil `shows/<slug>.yaml` mediante una cascada de búsqueda (env → junto al binario → config del usuario). Sin `--show` se usa un show por defecto configurable; el esquema del perfil **no se reinterpreta**.

4. **Lector y escritor canónico del tracker.** Lectura por `serde`; **escritura a mano**, reproduciendo el formato fijo del Python (cabecera de comentarios literal, orden de campos, `title` entrecomillado, episodios ordenados por fecha), con **backup `.bak`** y **escritura atómica**. No es un *round-trip* de YAML genérico: es el formato canónico de origen.

5. **Comandos de solo lectura con paridad byte a byte:** `status`, `next`, `next-date`, `list`, `list-range`, `get` y `where`. Iconos, alineación, leyendas (`* número provisional (se fija al publicar)`), bloques (`▶ Siguiente a publicar:`) y *exit codes* idénticos a los del Python.

6. **Salida `--json` opt-in.** `schema_version 1`, aditiva: no altera la salida por defecto y ningún agente migra a ella en esta fase. Se añade para poder migrarlos después, uno a uno.

7. **Suite de paridad.** Port 1:1 de los ~119 escenarios del Python que cubren estas rutas, corriendo sobre **fixtures temporales**. Ningún test ejecuta nada contra `/data/notas`.

## Capabilities

### New Capabilities

- `show-profile`: resolución de `--show` y del perfil del podcast (dónde se buscan los `shows/*.yaml`, cuál es el show por defecto, cómo se falla ante un perfil ausente o inválido).
- `tracker-store`: el modelo tipado del episodio y la lectura/escritura canónica del YAML de estado (formato fijo, backup, atomicidad, orden por fecha).
- `tracker-read`: la superficie CLI de solo lectura — `status`, `next`, `next-date`, `list`, `list-range`, `get`, `where` — con paridad de contrato.
- `json-output`: el contrato de la salida estructurada `--json` (opt-in, versionada, estable).

### Modified Capabilities

Ninguna: este repositorio no tiene especificaciones previas.

## Impact

- **`Cargo.toml` / `Cargo.lock`** — dependencias nuevas: `clap` (derive), `serde` (derive), `serde_json`, un backend YAML (`serde_yaml` o `serde_yml`), una librería de fechas (`time` o `chrono`), `anyhow`/`thiserror`; en dev, `assert_cmd`, `predicates` y `tempfile`.
- **`src/`** — módulos nuevos: arranque y CLI (`main.rs`), modelo (`model.rs`), perfil (`profile.rs`), tracker (`tracker.rs`), comandos de lectura (`commands/`) y salida JSON (`json.rs`).
- **`tests/`** — la suite de paridad, con fixtures temporales.
- **No** se toca `/data/notas`, ni los agentes de `~/.config/opencode/agents/`, ni `~/.agents/skills/podcast-srt/`. El Python sigue siendo el que corre en producción.
