# PLAN-007 — `material-layout-alignment`: una única regla para la carpeta de material

**Estado:** CERRADO — 2026-10-25
**Fecha:** 2026-10-25
**Ámbito:** código de `podtrack`. El material real de `/data/notas` solo se ha **leído**.
**Change archivado:** `openspec/changes/archive/2026-10-25-material-layout-alignment/`
**Rama:** `feature/material-layout-alignment`, creada desde `development` en `2d56a3a`.

## 1. Objetivo

Cerrar la deuda **D8** del SRS: el binario resolvía la carpeta de material de un episodio de dos
maneras distintas.

- `open` **renderizaba** `episodes.scripts.dir` del perfil (`{slug}`, `{n}`, `{n:0Nd}`).
- `where`, `get`, `status`, los documentos `--json` de esos comandos, `rebase-references` y el plan
  de `migrate-material` **cableaban el slug** (`<scripts_root>/<slug>`).

Ambas fórmulas coinciden mientras `dir` sea `{slug}` (el caso de `atareao-con-linux`), así que el
fallo estaba **latente**. Con `dir: "episodio-{n:02d}"` —el caso que declara
`la-era-de-las-distros`— discrepan: la lectura informaría de una carpeta que no existe y las
cirugías de material escribirían en el sitio equivocado.

## 2. Alcance aprobado

Una única fuente de verdad —`ResolvedShow::material_dir(slug, number)` en `src/profile.rs`— y **ocho
superficies** reapuntadas a ella: `where`, `get`, `status`, `json.rs` (×2 + `status_document`),
`rebase_references.rs`, el plan de `migrate_material.rs` y `open.rs`. Seis deltas de spec (cuatro
aprobados en el change original y dos más al cerrar la auditoría).

## 3. Decisiones (todas aprobadas por el usuario)

| ID | Decisión | Estado |
|---|---|---|
| D1 | Única fuente de verdad `ResolvedShow::material_dir(slug, number)` | Aplicada |
| D2 | Fallback a `{slug}` si el perfil no declara `episodes.scripts.dir` (o lo declara vacío), igual que `open` | Aplicada |
| D3 | El render no se reimplementa: se reutiliza `render_material_template` / `render_marker` | Aplicada |
| D4 | Se alinean también las **cirugías**, no solo la lectura (`migrate-material`, `rebase-references`) | Aplicada |
| D5 | Endurecimiento: un `dir` renderizado absoluto o con `..` es `MaterialDirNotRelative` (código **2**) | Aplicada |
| D6 | Errores tipados reutilizados (`EpisodeWithoutSlug`, `EpisodeWithoutNumber`); ninguno nuevo para el render | Aplicada |
| D7 | Regla uniforme «sin `slug` no hay ruta» en `Ctx::material` + precedencia de errores documentada | Aplicada |

## 4. Fases y evidencia

### 4.1 Línea base

`cargo test --locked` → **309 tests**, todos verdes, sin tocar ningún test.

### 4.2 RED

Fixture con `slug: episodio-siete`, `number: 7` y `dir: "episodio-{n:02d}"`, elegido a propósito para
que la carpeta correcta (`episodio-07`) y la vieja (`episodio-siete`) fueran distinguibles.
**11 tests nuevos en rojo** (4 unitarios en `src/profile.rs` + 7 de integración en
`tests/material_layout_alignment.rs`) y los **309** de la base verdes.

### 4.3 GREEN

`material_dir` real (render + fallback + rechazo D5) y los ocho puntos reapuntados; nuevo
`Ctx::material`. **325 verdes.**

### 4.4 Hueco detectado en la verificación propia (antes de commitear)

Con plantilla **numérica** y un episodio **sin `slug`**, el texto decía `—` pero el JSON emitía una
ruta: divergencia texto/JSON y violación de la cláusula incondicional de los deltas («Si el episodio
no tiene `slug`, SHALL mostrar `—`…» / «…o `null` si el episodio no tiene `slug`»). Reproducido con
el binario real:

| Superficie | Salida de entonces | Norma |
|---|---|---|
| `get` (texto) | `Ruta: —` | correcta |
| `status` (texto) | sin línea `ruta:` | correcta |
| `get --json` | `material: "/…/episodio-07"` | debía ser `null` |
| `status --json` | `material: "/…/episodio-07"` | debía ser `null` |
| `where <n>` | imprimía la ruta | caso tratado (paridad con la base) |

Se definió la regla **una sola vez** en `Ctx::material` (sin `slug` → `None`) y `where` pasó a
tratarlo como caso tratado. **330 verdes.**

### 4.5 REFACTOR

`cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` y `just check-all` limpios.

### 4.6 Auditoría (`rust-reviewer`)

**Sin bloqueantes.** Tres hallazgos importantes, los tres sobre el perfil objetivo del change:

| ID | Hallazgo | Corrección aplicada |
|---|---|---|
| I-1 | `rebase-references` **abortaba la cirugía entera** si un episodio tenía `slug` y le faltaba el número (estado que produce `backfill-slugs`), en vez de saltarlo como manda su spec | La carpeta no resoluble se salta; los errores de configuración se siguen propagando |
| I-2 | `EpisodeWithoutNumber` decía «…la ruta de su **presentación**» (texto escrito para `open`) y la etiqueta salía `—` | Mensaje generalizado a «su material» y etiqueta útil nueva (`render_episode_label`: el `slug` cuando falta el número) |
| I-3 | La spec delta, el design y el rustdoc anunciaban el marcador `{episode}`, que **no existe** (un `{episode}` literal habría fusionado el material de todos los episodios en una carpeta) | Retirado de los tres sitios |

Además: dos deltas nuevos (`reference-rebase`, `presentation-open`) para el comportamiento que el
change ya implementaba y que habría quedado sin declarar al archivar, la marca `#[doc(hidden)]` en
`collect_slug_renames` (envoltorio que solo usan los tests) y 9 tests nuevos (el caso «plantilla
numérica sin número», el salto de `rebase-references`, el endurecimiento en `open` y el renombrado
real de `migrate-material`). **339 verdes.**

### 4.7 Archivo

`just spec-archive material-layout-alignment`: los seis deltas fundidos en `openspec/specs/`, el
change movido a `openspec/changes/archive/2026-10-25-material-layout-alignment/`, `openspec list`
sin changes activos y `openspec validate --specs --strict` → **13 passed, 0 failed**.

## 5. Verificación end-to-end (perfil real, solo lectura)

Se compiló el release anterior (`v0.5.0`, commit `84e6339`) en un worktree aparte y se compararon las
dos salidas línea a línea sobre el tracker real de `atareao-con-linux` (`dir: "{slug}"`):

| Invocación | Resultado |
|---|---|
| `status`, `status --json` | idénticas (exit 0) |
| `get 843`, `get 843 --json`, `get 819`, `get 827 --json` | idénticas (exit 0) |
| `where 843`, `where 819`, `where 827` | idénticas (exit 0) |
| `open 843 --dry-run`, `open 843 --dry-run --json` | idénticas (exit 0) |

Salida del binario nuevo, para dejar constancia de la ruta real:

```
$ podtrack where 843
/data/notas/Podcasts/guiones/la-memoria-de-valet existe

$ podtrack get 843   (bloque)
  Slug:      la-memoria-de-valet
  Ruta:      /data/notas/Podcasts/guiones/la-memoria-de-valet

$ podtrack status    (bloque)
  ▶ Siguiente a publicar: #844 «Podman, IPs reales y La Liga»
      slug:  podman-ips-reales-y-la-liga
      ruta:  /data/notas/Podcasts/guiones/podman-ips-reales-y-la-liga

$ podtrack open 843 --dry-run
▶ typshow: /data/notas/Podcasts/guiones/la-memoria-de-valet/presentacion-la-memoria-de-valet.typ
  ⚠ DRY-RUN: no se lanza nada
```

`/data/notas` no se modificó: `git status` no muestra ninguna ruta atribuible a este trabajo (las dos
que aparecen dentro de `guiones` son de la producción del episodio 844, escritas a las 19:15–19:16
por otro proceso, antes de esta verificación).

## 6. Rollback

`git revert f003b74 b93a757 208fb53 6e9beb9` (y, para descartar también la propuesta, `7252453`).
**No hay datos que deshacer**: el change no escribió nada en `/data/notas`.

## 7. Hallazgos fuera de alcance y deuda viva

- Un `dir: "."` produce `<raíz>/.` (no escapa de la raíz y es coherente entre texto y JSON). Se
  acepta sin normalizar; ningún perfil real lo usa.
- `collect_slug_renames` (en `#[doc(hidden)]`) sigue siendo la forma con el `slug` cableado, que solo
  usan los tests de `material_renumber`. Migrarlos a `collect_slug_renames_into` en un cambio de
  limpieza: no se tocaron aquí para no romper el criterio de paridad.
- Deriva de títulos entre el tracker y el material publicado en 827, 831, 832 y 836 (anotada en
  `PLAN-006-COMPLETED.md` §8).
- Deuda viva ya registrada: `Fixture::show()` de `tests/common/mod.rs` no refleja las plantillas del
  perfil, y `open` mapea los fallos de `spawn` a `PodtrackError::Io(String)` perdiendo el `ErrorKind`.
- La convención `docs/FEATURE-NNN-<slug>.md` que declara el `.justfile` no tiene ningún documento
  todavía: usarla o retirar la receta.

## 8. Criterio de cierre — resultado

- [x] La regla única existe y las **ocho** superficies pasan por ella.
- [x] La paridad con `v0.5.0` es **byte a byte** en el perfil real (`where`, `get`, `status`, `--json`
      y `open --dry-run`).
- [x] **Ningún test preexistente se ha editado**: 309 → 339, y los 309 de la base siguen verdes.
- [x] `just check-all` verde (fmt + clippy + test + rustdoc) y `openspec validate --strict` OK.
- [x] Auditoría cerrada (3 importantes corregidos; menores y observaciones documentados).
- [x] Change archivado y `docs/DEFINITION.md` §6.1 documenta la regla.
- [x] `/data/notas` intacto: solo lectura.

**Commits de la rama:**

```
7252453 📝 docs(spec): aprueba el change material-layout-alignment
6e9beb9 🐛 fix(material): resuelve la carpeta de material con una única regla
208fb53 🐛 fix(material): cierra los hallazgos de la auditoría del change
b93a757 📝 docs(definicion): documenta la regla única de la carpeta de material
f003b74 📝 docs(spec): archiva material-layout-alignment
```

**Release:** conventional commit `fix` ⇒ **patch**: `0.5.0 → 0.5.1`.
