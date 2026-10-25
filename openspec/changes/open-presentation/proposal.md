# Proposal

## Why

`podtrack` gobierna los metadatos y el *layout* del material, pero **no puede abrirlo**. La
presentación de cada episodio es un documento **Typst** (`presentacion-<slug>.typ`, junto a su
PDF) que se visualiza con **typshow**, un visor de doble ventana (controlador + proyección)
instalado en el PATH, cuyo único argumento de arranque es la ruta del `.typ`.

Hoy, para abrir la presentación del episodio 843 hay que teclear a mano la ruta completa
(`typshow /data/notas/Podcasts/guiones/la-memoria-de-valet/presentacion-la-memoria-de-valet.typ`),
replicando —a mano y mal— la resolución de material que el binario ya sabe hacer. Se quiere
`podtrack open 843`.

El oráculo Python **no tiene** comando `open`: esto es una **extensión aditiva** de podtrack, no
paridad. No altera ningún contrato existente (ni salida de texto, ni *exit codes*, ni JSON) de los
subcomandos ya portados.

## What Changes

1. **Nuevo subcomando `podtrack open <num|slug>`:** resuelve el episodio por número o por slug
   (la misma resolución que `get`/`where`), deriva la ruta de la **fuente** de su presentación y
   lanza `typshow` con ella.
2. **Derivación de la ruta desde el perfil (§4 del SRS y esquema `scripts`):**
   - la carpeta es `<scripts.root>/<scripts.dir>`, con `scripts.dir` **renderizado** (admite
     `{slug}` y `{n}`/`{n:0Nd}`); si el perfil no declara `dir`, se asume `{slug}`;
   - el nombre de fichero sale de `episodes.scripts.files.presentacion` (hoy
     `presentacion-{slug}.pdf`), renderizado, **reescribiéndole la extensión a `.typ`**.
   - Si el perfil **no declara** `files.presentacion` (p. ej. `la-era-de-las-distros`), es un
     error: el show no tiene presentación que abrir. **No se inventa** una ruta.
3. **Lanzamiento no bloqueante:** `open` arranca `typshow` **desacoplado** y **termina de
   inmediato** con éxito; NO espera a que se cierre la ventana. El *stdio* del visor se redirige
   para no contaminar la salida del binario.
4. **`--dry-run`:** resuelve y **muestra** la ruta sin lanzar nada (necesario para testear sin GUI
   y sin tocar `/data/notas`).
5. **`--json` (aditivo, opt-in):** documento con el show, el episodio y la ruta resuelta.
6. **Errores tipados nuevos:** `PresentationNotDeclared` (el perfil no declara presentación),
   `PresentationNotFound` (el `.typ` no existe), `LauncherNotFound` (`typshow` ausente del PATH) y
   `EpisodeWithoutNumber` (el layout necesita `{n}` y el episodio no tiene número); reutiliza
   `EpisodeWithoutSlug`.

## Capabilities

### New Capabilities
- `presentation-open`: resolución de la ruta de la presentación Typst de un episodio a partir del
  layout de material del perfil, su apertura desacoplada con `typshow`, y el contrato de salida
  (texto/JSON), `--dry-run` y errores tipados con sus códigos.

### Modified Capabilities
Ninguna. El consumo de `episodes.scripts.*` ya está cubierto por el requisito «El esquema del perfil
no se reinterpreta» (`episodes.*`); este change NO lo altera.

## Impact

- `src/commands/open.rs` (nuevo) — el subcomando: resolución, render de plantillas, lanzamiento.
- `src/profile.rs` — `ResolvedShow` expone las plantillas de material (`scripts.dir` y
  `scripts.files.presentacion`) leídas del perfil.
- `src/error.rs` — cuatro variantes nuevas con sus códigos de salida.
- `src/main.rs` — subcomando `open` con `<target>` y `--dry-run`.
- `src/json.rs` — documento de `--json`.
- `src/commands/mod.rs` — registro del módulo `open`.
- `tests/` — tests de integración con perfiles y material de **fixtures temporales** (nunca
  `/data/notas`).
- `docs/DEFINITION.md` — §6 (superficie CLI) y §10 (no-objetivos, si procede).
- **No** hay paridad que preservar: el Python no tiene `open`. **No** se toca `/data/notas`.
