# Proposal

## Why

El binario resuelve la **carpeta de material** de un episodio de dos maneras distintas que hoy
coinciden por casualidad:

- `open` **renderiza** `episodes.scripts.dir` del perfil (`{slug}`, `{n}`, `{n:0Nd}`) — es la
  implementación de referencia, la que introdujo el change `open-presentation`.
- `where`, `get`, `status`, los documentos `--json` de esos dos, `rebase-references` y
  `collect_slug_renames` (el plan de renombrado de `material.rs`, que usa `migrate-material`)
  **cablean el slug**: `scripts_root().join(slug)`.

Para `atareao-con-linux` (`scripts.dir: "{slug}"`) las dos fórmulas dan el mismo resultado, así que
la divergencia está **latente**. Para un perfil con `dir: "episodio-{n:02d}"` —el caso que declara
`la-era-de-las-distros`— dan rutas distintas: el binario informaría de una carpeta que no existe y
las cirugías de material escribirían en el sitio equivocado.

Es la deuda **D8**: `open` honra `scripts.dir`; todo lo demás, no.

## What Changes

1. **Una única fuente de verdad** en `src/profile.rs`: `ResolvedShow::material_dir(slug, number)`
   devuelve `<scripts_root>/<dir>`, con `dir` = `episodes.scripts.dir` **renderizado**. Si el perfil
   no declara `dir` o lo declara vacío, SHALL asumirse `{slug}` (mismo criterio que `open`).
2. **Reapuntar las seis derivaciones** a ese helper: `where`, `get`, `status` (bloque «Siguiente a
   publicar»), los documentos `--json` de `where` y `get`, `rebase-references` y
   `collect_slug_renames`. `open` pasa también a usarlo, sin cambiar su contrato.
3. **Cero cambio de comportamiento** en los perfiles que declaran `dir: "{slug}"` o no declaran
   `dir`: la fórmula nueva colapsa a la vieja. La paridad con el oráculo retirado no se toca (el
   Python tampoco resolvía plantillas).
4. **Errores tipados ya existentes**: `EpisodeWithoutSlug` (plantilla con `{slug}` y episodio sin
   slug) y `EpisodeWithoutNumber` (`{n}`/`{n:0Nd}` sin número). No se inventan variantes para eso.
5. **Endurecimiento**: si el `dir` renderizado es **absoluto** o contiene componentes `..`, SHALL ser
   un error tipado nuevo (`MaterialDirNotRelative`) con el código de error de configuración, en vez
   de escapar de la raíz de material.

## Capabilities

### New Capabilities
Ninguna.

### Modified Capabilities
- `show-profile`: fija la **regla única** de resolución de la carpeta de material (render de
  `episodes.scripts.dir` y fallback a `{slug}`) y el rechazo de rutas no relativas.
- `tracker-read`: `where`, `get` y `status` pasan de `<scripts.root>/<slug>` a `<scripts.root>/<dir>`.
- `json-output`: el campo de ruta de los documentos `--json` de `where`/`get` sigue la misma regla.
- `material-slug-migration`: el destino de la carpeta migrada pasa de `<scripts_root>/<slug>` a
  `<scripts_root>/<dir>`.

## Impact

- `src/profile.rs` — helper nuevo `material_dir` + rustdoc; reutiliza `render_material_template`.
- `src/material.rs` — `collect_slug_renames` recibe el destino ya resuelto.
- `src/commands/where_cmd.rs`, `src/commands/get.rs`, `src/commands/status.rs`,
  `src/commands/rebase_references.rs` — reapuntados al helper.
- `src/commands/open.rs` — pasa a usar el helper (contrato de salida intacto).
- `src/json.rs` — los dos puntos que componen la ruta de material.
- `src/error.rs` — variante nueva `MaterialDirNotRelative` con su `exit_code()`.
- `tests/` — escenarios nuevos con un perfil de fixture `dir: "episodio-{n:02d}"`, sobre **fixtures
  temporales**; jamás `/data/notas`.
- `docs/DEFINITION.md` §6 si procede.
- **No** se toca `/data/notas`. **No** se reapuntan agentes del pipeline.
