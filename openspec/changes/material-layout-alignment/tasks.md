# Tasks

## 0. Andamiaje

- [x] 0.1 `src/profile.rs`: `ResolvedShow::material_dir(slug, number)` — render de
      `episodes.scripts.dir` con `{slug}` por defecto y raíz `scripts_root()`.
- [x] 0.2 `src/error.rs`: variante `MaterialDirNotRelative { show, dir }` con su mensaje y su brazo
      de `exit_code()` (código de configuración, `2`).
- [x] 0.3 `cargo check --locked` limpio.

## 1. RED (tests que fallan primero)

- [x] 1.1 Unitario en `src/profile.rs`: `dir: "{slug}"` → `<root>/<slug>`; `dir: "episodio-{n:02d}"`
      con número 7 → `<root>/episodio-07`; sin `dir` → `<root>/<slug>`; `dir: "/etc"` →
      `MaterialDirNotRelative`; `dir: "../../x"` → `MaterialDirNotRelative`; `{n}` sin número →
      `EpisodeWithoutNumber`; `{slug}` sin slug → `EpisodeWithoutSlug`. No compila hoy.
- [x] 1.2 `tests/material_layout_alignment.rs` (nuevo): fixture de perfil con
      `scripts.dir: "episodio-{n:02d}"`, tracker con un episodio de número conocido y material
      temporal en `episodio-NN/`. `where <slug>` imprime `<root>/episodio-NN`; `get <slug>` muestra
      esa ruta en su línea `Ruta:`; `status` la muestra en el bloque «Siguiente a publicar». Falla
      hoy (los tres imprimen `<root>/<slug>`).
- [x] 1.3 `tests/material_layout_alignment.rs`: `where --json` y `get --json` emiten la ruta
      renderizada en su campo de material. Falla hoy.
- [x] 1.4 `tests/material_layout_alignment.rs`: `migrate-material --dry-run --include-published`
      sobre una carpeta `0NNN` planifica el destino `<root>/episodio-NN`, no `<root>/<slug>`.
      Falla hoy.
- [x] 1.5 `tests/material_layout_alignment.rs`: el endurecimiento — un perfil con `dir: "/etc"` hace
      que `where` salga con código `2` y el mensaje nombre el perfil. Falla hoy (sale con `0`).
- [x] 1.6 `cargo test --locked`: 1.1–1.5 FALLAN y **toda** la suite previa permanece VERDE. Que la
      suite previa pase sin editar ningún test existente es criterio de aceptación.
- [x] 1.7 (hueco de la verificación) `tests/material_layout_alignment.rs`: con una plantilla
      **numérica** (`episodio-{n:02d}`) y un episodio **sin slug**, la ausencia de slug manda —`get`
      muestra `—` en `Slug:`/`Ruta:`, `get --json`/`status --json` emiten `material: null`, `status`
      omite la línea `ruta:` y `where` sale como caso tratado—. Falla hoy en las cuatro superficies
      JSON/traza (`get`/`status` en texto ya cumplían). Fija la cláusula «sin slug no hay ruta» de
      `tracker-read`/`json-output`.
- [x] 1.8 (auditoría) `tests/material_layout_alignment.rs`: plantilla **numérica** + episodio **con
      slug y sin número** — `get <slug>` y `status` salen con `1` **nombrando el episodio** (por su
      slug, no `—`) y con `no tiene número`; `get --json` emite el documento de error. Falla hoy (el
      mensaje sale con `'—'`).
- [x] 1.9 (auditoría, I-1) `tests/material_layout_alignment.rs`: `rebase-references` con plantilla
      numérica y un episodio **con slug y sin número** — sale `0` y no aborta—; y con un episodio con
      número y carpeta `episodio-NN/` existente — visita esa carpeta (el hook se reescribe). Falla
      hoy el primero (aborta con `1`).
- [x] 1.10 (auditoría, cobertura) `tests/material_layout_alignment.rs`: `open --dry-run` con
      `dir: "/etc"` sale con `2` y no lanza; `migrate-material` **sin** `--dry-run` crea
      `<raíz>/episodio-NN/` y no `<raíz>/<slug>/`; `where` con `dir: "../../fuera"` sale con `2`.
- [x] 1.11 (auditoría) `cargo test --locked`: 1.8–1.9 FALLAN (4 escenarios) y **toda** la suite
      previa permanece VERDE sin editar ningún test existente.

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/profile.rs`: implementar `material_dir` reutilizando `render_material_template` y
      aplicando el rechazo de rutas no relativas.
- [x] 2.2 `src/commands/where_cmd.rs`, `src/commands/get.rs`, `src/commands/status.rs`: sustituir
      `scripts_root().join(slug)` por `material_dir(...)`. `next_to_publish_block` cambia de firma
      para recibir la ruta ya resuelta (o el show).
- [x] 2.3 `src/json.rs`: los dos puntos que componen la ruta de material usan el helper.
- [x] 2.4 `src/commands/rebase_references.rs`: `root.join(&slug)` → `material_dir(...)`.
- [x] 2.5 `src/material.rs`: `collect_slug_renames` recibe el destino ya resuelto en vez de cablear
      el slug sobre `root`.
- [x] 2.6 `src/commands/open.rs`: pasa a usar `material_dir` sin cambiar su contrato de salida.
- [x] 2.7 `cargo test --locked` y `cargo check --locked` en verde.
- [x] 2.8 (hueco de la verificación) `src/commands/mod.rs`: la regla uniforme vive en `Ctx::material`
      —un episodio **sin slug** no tiene ruta de material (`None`), sea cual sea la plantilla; con
      slug, delega en `material_dir` y propaga sus errores—. `where` (`where_cmd.rs`) la consume y, si
      es `None`, devuelve `EpisodeWithoutSlug` (caso tratado, mismo error/código que la línea base);
      `status.rs::next_to_publish_block` recibe el `Ctx` y usa la misma regla; `get.rs` prescinde de
      la guarda local. `open.rs` NO se toca (su `EpisodeWithoutSlug` es dependiente de la plantilla).
      Cláusula de `specs/json-output/spec.md` («`material` … o `null` si el episodio no tiene
      `slug`») y de `specs/tracker-read/spec.md` («NO SHALL inventarse ninguna ruta de material»).
- [x] 2.9 (auditoría, I-1/I-2) `src/commands/rebase_references.rs`: un `EpisodeWithoutNumber` **no**
      aborta la cirugía —se salta, como los publicados/sin-slug/carpetas inexistentes— y solo se
      propaga un error de configuración. `src/error.rs`: mensaje de `EpisodeWithoutNumber`
      generalizado a «…la ruta de su **material**» (conserva la subcadena `no tiene número`).
      `src/profile.rs`: nuevo `render_episode_label(slug, number)` (el slug si lo hay, si no el
      número) usado por `material_dir` y por `open.rs`, para que el mensaje nombre el episodio en vez
      de `'—'`; test unitario del helper.
- [x] 2.10 (auditoría, I-3 + deltas) Retirada la mención falsa del marcador `{episode}` del rustdoc de
      `material_dir`, del delta `specs/show-profile/spec.md` y de `design.md` (el render solo
      sustituye `{slug}`, `{n}`, `{n:0Nd}`). Nuevos deltas `specs/reference-rebase/spec.md` y
      `specs/presentation-open/spec.md`, con sus dos capacidades añadidas a «Modified Capabilities»
      de `proposal.md`. `where --json` sin slug (documento de error) fijado en `tracker-read` y en
      `design.md` D7. `collect_slug_renames` marcado `#[doc(hidden)]`.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets --locked -- -D warnings` limpios.
- [x] 3.2 Rustdoc del helper, del error nuevo y de los sitios reapuntados.
- [x] 3.3 Ningún test toca `/data/notas` (fixtures temporales).
- [x] 3.4 `just check-all` en verde.

## 4. Auditoría y verificación end-to-end

- [x] 4.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
- [x] 4.2 Binario real: con el perfil de `atareao-con-linux`, `where`, `get` y `status` siguen dando
      exactamente la misma ruta que antes del change.
- [x] 4.3 `where --json` y `get --json` sin cambios para `dir: "{slug}"`.

## 5. Documentación y cierre

- [x] 5.1 `docs/DEFINITION.md` §6: anotar que la carpeta de material sale de `scripts.dir`
      renderizado, no del slug cableado.
- [ ] 5.2 `just spec-archive material-layout-alignment`.
- [ ] 5.3 Cerrar el plan correspondiente.

## Release

- Rama `feature/material-layout-alignment` → PR a `development` → merge → PR `development → main`.
- Conventional commit `fix` (gitmoji) ⇒ **patch** automático del CI: `0.5.0 → 0.5.1`.
