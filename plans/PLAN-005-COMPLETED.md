# PLAN-005 — `open-presentation`: abrir la presentación de un episodio con typshow

- **Estado:** en curso
- **Change OpenSpec:** `open-presentation`
- **Ámbito:** nuevo subcomando `open` + lectura del layout de material del perfil (`src/profile.rs`)
- **Release objetivo:** minor `0.4.2 → 0.5.0` (subcomando nuevo, extensión aditiva; no BREAKING)

## Objetivo

`podtrack open <num|slug>` resuelve el episodio y abre su presentación Typst
(`presentacion-<slug>.typ`) con **typshow**, sin bloquear: lanza el visor en segundo plano y vuelve
de inmediato. La ruta se deriva del **layout de material del perfil**, no de una regla cableada.

## Problema

Hoy hay que teclear a mano
`typshow /data/notas/Podcasts/guiones/<slug>/presentacion-<slug>.typ`, replicando la resolución de
material que el binario ya conoce. El Python no tiene `open`: es capacidad **nueva**, no paridad.

## Decisiones cerradas

| # | Decisión |
|---|---|
| D1 | Ruta desde el perfil: `<scripts.root>/<scripts.dir>` (renderizado) + `episodes.scripts.files.presentacion` (renderizado) con la extensión reescrita a `.typ`. `scripts.root` = directorio padre del tracker. |
| D2 | Render de plantillas mínimo: `{slug}`, `{n}` y `{n:0Nd}`. Falta el valor → error tipado (`EpisodeWithoutSlug` / `EpisodeWithoutNumber`). Marcador desconocido → literal. El render vive en `src/profile.rs`. |
| D3 | `typshow` se busca **solo en el PATH** (sin env ni flag). Ausente → `LauncherNotFound` (exit 1). |
| D4 | Lanzamiento **desacoplado**: `Command::new("typshow").arg(<ruta>)`, stdio a `null`, `process_group(0)`, `spawn()` sin `wait()`. `podtrack` sale con 0 de inmediato. |
| D5 | `--dry-run`: resuelve y muestra la ruta sin lanzar (testeable sin GUI, sin tocar `/data/notas`). |
| D6 | Errores: `PresentationNotDeclared` (exit 2, defecto de perfil), `PresentationNotFound` (exit 1), `LauncherNotFound` (exit 1), `EpisodeWithoutNumber` (exit 1); reutiliza `EpisodeWithoutSlug` y `EpisodeNotFound`. |
| D7 | Extensión aditiva: no toca salida/códigos/YAML de ningún subcomando portado; la paridad sigue verde. |
| D8 | Material legacy numerado (`guiones/0819/`) fuera de alcance, igual que `where`. Deuda anotada: `open` honra `scripts.dir`, `where`/`get` cablean el slug. |

## Contrato de salida

- Texto normal: `▶ typshow: <ruta>` — exit 0.
- `--dry-run`: `▶ typshow: <ruta>` + `  ⚠ DRY-RUN: no se lanza nada` — exit 0, sin lanzar.
- `--json`: `{ "schema_version": 1, "show": …, "episode": { "number": …, "slug": … }, "presentation": "<ruta.typ>", "launcher": "typshow", "launched": true|false }`.

## Tareas

### 0. Andamiaje
- [ ] 0.1 `ResolvedShow` expone las plantillas de material (`scripts.dir`, `scripts.files.presentacion`), sin romper `Eq`.
- [ ] 0.2 Helper de render de plantillas (`{slug}`, `{n}`, `{n:0Nd}`) con su error.
- [ ] 0.3 `cargo check --locked` limpio.

### 1. TDD — RED
- [ ] 1.1 `tests/open_presentation.rs`: fixtures temporales; `open <slug> --dry-run --json` resuelve la ruta y `launched:false`; `open <num> --dry-run` da la misma ruta.
- [ ] 1.2 Perfil sin `files.presentacion` → exit 2.
- [ ] 1.3 `.typ` declarado pero ausente → exit 1, mensaje con episodio y ruta, sin lanzar.
- [ ] 1.4 Unitario de render en `src/profile.rs` (`{n:02d}` → `episodio-07`; faltas → error).
- [ ] 1.5 Unitario de `exit_code()` y mensajes de los cuatro errores nuevos.
- [ ] 1.6 `cargo test --locked`: 1.1–1.3 FALLAN, 1.4–1.5 no compilan, resto VERDE.

### 2. TDD — GREEN
- [ ] 2.1 `src/error.rs`: cuatro variantes nuevas con mensajes y brazo de `exit_code()`.
- [ ] 2.2 `src/profile.rs`: lectura de `scripts.dir`/`files.presentacion` + render.
- [ ] 2.3 `src/commands/open.rs`: resolución, composición de la ruta, `--dry-run`, lanzamiento desacoplado.
- [ ] 2.4 `src/main.rs`: subcomando `open <target> --dry-run`.
- [ ] 2.5 `src/json.rs`: documento de `open`.
- [ ] 2.6 `cargo test --locked` y `cargo check --locked` en verde.

### 3. TDD — REFACTOR
- [ ] 3.1 `cargo fmt --all` / `cargo clippy --all-targets --locked -- -D warnings` sin avisos.
- [ ] 3.2 Rustdoc de las variantes, del render y del comando.
- [ ] 3.3 Ningún test toca `/data/notas`.
- [ ] 3.4 `just check-all` en verde.

### 4. Auditoría y verificación end-to-end
- [ ] 4.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
- [ ] 4.2 `open 843 --dry-run` imprime la ruta real de `la-memoria-de-valet`.
- [ ] 4.3 Manual: `open 843` abre el `.typ` con typshow y `podtrack` vuelve de inmediato.
- [ ] 4.4 `open 843 --json --dry-run` emite el documento esperado.

### 5. Documentación y cierre
- [ ] 5.1 `docs/DEFINITION.md` §6: fila del subcomando `open`.
- [ ] 5.2 `just spec-archive open-presentation`.
- [ ] 5.3 `just plan-done 5`.

## Deuda que NO paga este plan

- Alinear `where`/`get`/`status` con `scripts.dir` (hoy cablean el slug). Deuda preexistente.
- Material legacy numerado sin migrar (lo arregla `migrate-material`).
- Override del binario por env/flag (`PODTRACK_TYPSHOW`): el usuario eligió solo PATH.
- `tests/common/mod.rs::Fixture::show()` fija `scripts_dir: None`/`presentation: None`, así que un futuro test de librería sobre `open::run` con `show()` daría `PresentationNotDeclared` engañoso (el fixture no declara el layout de material).
- `open` mapea los errores de E/S de `spawn` a `PodtrackError::Io(String)`, perdiendo el `ErrorKind` original; solo el caso `NotFound` (lanzador ausente) se preserva como error tipado.
