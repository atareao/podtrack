# PLAN-003 — `slug-str-and-tracker-trim`: las dos asimetrías del `trim`

- **Estado:** terminado
- **Change OpenSpec:** `slug-str-and-tracker-trim`
- **Ámbito:** modelo (`src/model.rs`, accesor del slug) + perfil (`src/profile.rs`, `trim` del `tracker:`) + consumidores (`src/commands/{status,where_cmd,get}.rs`, `src/json.rs`)
- **Release objetivo:** parche `0.4.0 → 0.4.1` (no BREAKING)

## Objetivo

Cerrar las dos asimetrías del `trim` registradas como deuda tras `get-shows-slug-and-path` (D8) y `profile-absolute-paths`: (1) la extracción del `slug` duplicada en varios sitios con `trim` asimétrico —texto recorta, JSON no—, unificada con un accesor canónico `Episode::slug_str()`; y (2) el `tracker:` del perfil, que se compara recortado (`raw.trim()`) pero se resuelve sin recortar (`Path::new(&raw)`).

## Problema

- **`tracker:` del perfil.** `src/profile.rs::ShowResolver::load` usa `raw.trim()` en la guarda de ausencia (~línea 193) pero construye la ruta con `Path::new(&raw)` (~línea 200). Un perfil con `tracker: " /a/b.yaml "` (entrecomillado: YAML preserva los espacios) pasa la guarda —no está vacío— y produce una ruta con espacios que no existe → `TrackerMissing` espurio. Ningún perfil real lo declara: «funciona por casualidad del dato». **Observable y testeable.**
- **Extracción del `slug`.** Sitios con `trim`: `status.rs::next_to_publish_block`, `where_cmd.rs::run`, `get.rs::run` y el `material` de `json.rs::get_document`. Sitios **sin** `trim`: `json.rs::status_document` (next_to_publish y last_published), `json.rs::where_document` y `json.rs::episode_fields`. **No observable** hoy: `src/tracker.rs:95` ya normaliza el slug al cargar (`Slug::parse(s.trim())`) y `Slug::parse` exige kebab-case, así que un episodio cargado nunca tiene slug con espacios ni vacío.

## Decisiones cerradas

| # | Decisión |
|---|---|
| D1 | El accesor canónico vive en el modelo: `Episode::slug_str(&self) -> Option<&str>` = slug recortado y no vacío, o `None`. Firma prescrita en D8 del change archivado. Vista prestada, sin asignar. |
| D2 | La mitad «slug» es un refactor **no observable**; se registra como contrato del modelo (DRY + defensa), no como bug fix. No cambia ninguna salida del CLI para datos cargados. |
| D3 | La mitad «`tracker:`» **sí** es observable: `ShowResolver::load` resuelve con `raw.trim()`, coherente con la guarda. |
| D4 | Deltas de spec: `show-profile` (ADDED, observable) y `tracker-store` (ADDED, contrato de modelo, caja blanca). `tracker-read` y `json-output` **sin** delta: ningún `Given/When/Then` suyo cambia. |
| D5 | Fuera de alcance: no se normaliza el slug en `add`/`insert`/`backfill-slugs`/`renumber` (ya pasan por `Slug::parse`); no se toca el writer, el parseo del tracker, los códigos de error ni el esquema JSON; no se documenta la ruta relativa en `shows/README.md` (fuera del repo). |

## Contrato de salida

Con perfil `tracker: " /data/notas/Podcasts/guiones/podcast-tracker.yaml "`:

```
(la ruta resuelta es /data/notas/Podcasts/guiones/podcast-tracker.yaml, sin espacios)
```

Con un episodio con slug, texto y JSON exponen el **mismo** valor canónico:

```
$ podtrack get la-memoria-de-valet        →  Slug:      la-memoria-de-valet
$ podtrack get la-memoria-de-valet --json →  episode.slug == "la-memoria-de-valet"
```

Para datos reales (perfil absoluto y limpio, slugs sin espacios): salida **byte a byte** idéntica a la de hoy.

## Tareas

### 1. Spec
- [x] 1.1 `just spec-validate slug-str-and-tracker-trim` en estricto (deltas `tracker-store` + `show-profile`).

### 2. TDD — RED («`tracker:`», observable)
- [ ] 2.1 Unitario en `src/profile.rs`: `tracker: " /ruta/absoluta.yaml "` → `ResolvedShow.tracker == /ruta/absoluta.yaml` (falla hoy).
- [ ] 2.2 Unitario en `src/profile.rs`: `tracker: "   "` → error tipado de tracker ausente.
- [ ] 2.3 Integración (`tests/`): perfil con `tracker:` entrecomillado con espacios resuelve a la ruta limpia (falla hoy).
- [ ] 2.4 `cargo test --locked`: los nuevos FALLAN, los preexistentes VERDES.

### 3. TDD — RED («slug», no observable)
- [ ] 3.1 Unitario en `src/model.rs` del accesor: normal → `Some(...)`; `" x "` en caja blanca → `Some("x")`; sin slug / vacío / solo espacios → `None` (falla hoy, el accesor no existe).
- [ ] 3.2 Paridad texto/JSON del slug para episodios con y sin slug (mismo valor canónico).

### 4. TDD — GREEN («`tracker:`»)
- [ ] 4.1 `ShowResolver::load`: resolver con `raw.trim()`.
- [ ] 4.2 `cargo test --locked` y `cargo check --locked` en verde.

### 5. TDD — GREEN («slug»)
- [ ] 5.1 `Episode::slug_str()` en `src/model.rs` con rustdoc.
- [ ] 5.2 `status.rs`, `where_cmd.rs`, `get.rs`: usar el accesor.
- [ ] 5.3 `json.rs`: `status_document` (next_to_publish y last_published), `get_document` (material), `where_document` y `episode_fields` usan el accesor.
- [ ] 5.4 `cargo test --locked` y `cargo check --locked` en verde.

### 6. TDD — REFACTOR
- [ ] 6.1 `cargo fmt --all` / `cargo clippy --all-targets -- -D warnings` sin avisos.
- [ ] 6.2 Rustdoc del accesor y de la resolución recortada del `tracker:`.
- [ ] 6.3 `just check-all` en verde.

### 7. Auditoría y verificación end-to-end
- [ ] 7.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
- [ ] 7.2 Binario real sobre fixture `tracker: " /ruta "`: `status`/`get`/`where` resuelven la ruta limpia.
- [ ] 7.3 Paridad texto/JSON del slug sobre la fixture (con slug y sin slug).

### 8. Documentación y cierre
- [ ] 8.1 `docs/DEFINITION.md`: nota de que el `tracker:` se interpreta recortado (si procede).
- [ ] 8.2 `just spec-archive slug-str-and-tracker-trim` (fundir deltas en `openspec/specs/{tracker-store,show-profile}/spec.md`).
- [x] 8.3 `just plan-done 3`.

## Release

- Rama `feature/slug-str-and-tracker-trim` → PR a `development` → merge → PR `development → main`.
- Conventional commit `fix`/`refactor` (gitmoji) ⇒ bump de parche automático del CI: `0.4.0 → 0.4.1`.
- CI: `release-prepare` (bump + CHANGELOG + tag `v0.4.1`) y `release` (crates.io + GitHub Release).

## Deuda que NO paga este plan

- Mapeo de `io::Error` a `InvalidProfile` en `profile.rs` (poco probable, sin síntoma real).
- Documentación de la semántica de ruta relativa en `shows/README.md` (fuera del repo).
