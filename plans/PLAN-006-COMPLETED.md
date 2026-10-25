# PLAN-006 — Migración del material de guion legacy a carpetas por slug

**Estado:** CERRADO — 2026-10-25
**Fecha:** 2026-10-25
**Ámbito:** datos reales en `/data/notas/Podcasts/guiones` (repo git de notas). **No** se toca código de `podtrack`.

## 1. Objetivo

Alinear el material de guion del **backlog publicado** (`0817`–`0842`) y limpiar el residuo `0863` para que todo el árbol cumpla la spec `episode-material-layout`: el directorio del episodio es `<slug>`, y los ficheros internos llevan `{slug}` en vez del token numérico.

Motivo práctico: `scripts.dir: "{slug}"`, así que hoy `podtrack where 819` responde «no existe» y `podtrack open 819` falla, porque la carpeta en disco es `0819`. Migrar enciende `where`/`open`/`get` en el backlog. Es la deuda registrada como D8 del SRS.

## 2. Reconocimiento (hecho, solo lectura)

`podtrack migrate-material --show atareao-con-linux --include-published --dry-run` sobre el árbol real:

- **27 carpetas numéricas**: `0817`–`0842` (26) + `0863` (1).
- **26 carpetas por slug** = los 26 episodios **no** publicados (844–869) → ya migrados en su día (commit `0a0f293`).
- Sin `--include-published` la pasada **no migra nada**: por defecto salta los publicados.
- Resultado del plan: **22 limpias**, **4 con colisión de fichero interno**, **1 con carpeta destino existente**.

| Carpeta | Colisión detectada |
|---|---|
| `0819` | `investigacion-0819.md` y `investigacion-e819.md` → mismo destino |
| `0822` | `investigacion-0822.md` y `investigacion-e822.md` → mismo destino |
| `0827` | `investigacion-0827.md` y `investigacion-e827.md` → mismo destino |
| `0842` | `investigacion-*.md` y `qa-*.md` (dos pares) → mismo destino |
| `0863` | el destino `autogen-agentes-que-hablan-entre-si` ya existe |

Evidencia de los pares `NNN` vs `eNNN` (fichero del mismo episodio, dos versiones):

| Carpeta | Par | Fechas (NNN → eNNN) | Tamaños (bytes) |
|---|---|---|---|
| 0819 | investigacion | 2026-07-25 → 2026-08-02 | 33.532 → 33.064 |
| 0822 | investigacion | 2026-07-25 → 2026-08-12 | 40.902 → 18.083 |
| 0827 | investigacion | 2026-08-29 → 2026-08-30 | 22.195 → 25.718 |
| 0842 | investigacion | 2026-10-01 → 2026-10-18 | 20.191 → 59.545 |
| 0842 | qa | 2026-10-01 → 2026-10-18 | 5.885 → 13.299 |

En los **cinco** casos el `eNNN` es **más nuevo** y es el que porta el frontmatter canónico del pipeline.

`0863/` no es un duplicado: contiene entregables POST antiguos (`articulo-e863.md`, `capitulos-e863.md`, `investigacion-e863.md`, `spotify-e863.md`, `working-e863.md`, `youtube-e863.md`, del 2026-09-02), mientras que el slug folder `autogen-agentes-que-hablan-entre-si/` tiene los insumos nuevos (`escaleta`, `guion`, `investigacion`, `ejemplos`, `presentacion.pdf`, `qa`, del 2026-10-24). Es material **superado**.

## 3. Decisiones aprobadas por el usuario

1. **Alcance:** migrar todo el backlog con `--include-published`.
2. **Conflictos internos:** conservar el `eNNN` (siempre más nuevo) y **archivar** el `NNN` antiguo.
3. **`0863`:** archivar la carpeta entera.
4. **Seguridad:** snapshot de git en `/data/notas` como baseline antes de tocar nada.

## 4. Procedimiento

1. **Snapshot**: commit en `/data/notas` del estado actual de `Podcasts/guiones` → baseline de rollback.
2. **Cirugía**: mover a `_legacy/` los ficheros antiguos y la carpeta `0863`:
   - `0819/investigacion-0819.md` → `_legacy/0819/`
   - `0822/investigacion-0822.md` → `_legacy/0822/`
   - `0827/investigacion-0827.md` → `_legacy/0827/`
   - `0842/investigacion-0842.md`, `0842/qa-0842.md` → `_legacy/0842/`
   - `0863/` → `_legacy/0863/`
   - `_legacy/README.md` explicando por qué está ahí.
   Nada se borra: el material antiguo sigue en disco, fuera del árbol de episodios (mismo criterio que las carpetas `_duplicado-e832` y `_pipewire_temp` ya existentes).
3. **Dry-run de verificación**: esperar **26 carpetas migradas y 0 colisiones** (exit 0).
4. **Migración real**: `podtrack migrate-material --show atareao-con-linux --include-published`.
5. **Verificación**: sin carpetas numéricas; `where`/`open` resuelven 819, 827 y 842.
6. **Commit** de la migración en `/data/notas` (cirugía y migración en commits separados).

## 5. Rollback

- Baseline en git (paso 1). Deshacer = `git revert` de los commits de cirugía y migración, o `git checkout <snapshot> -- Podcasts/guiones`.
- `_legacy/` conserva los originales, así que **no hay pérdida de datos** en ningún caso.
- Los `.typshow.db` no están versionados (`.gitignore`), pero son cachés regenerables.

## 6. Hallazgos fuera de alcance (no se arreglan aquí)

- **Deriva tracker ↔ publicado**: el tracker conserva títulos antiguos en 827 (`Recursos IA: el botiquín del explorador`), 831 (`MCPs que no pueden faltar en tu IA`), 832 (`Migrar de Uptime-Kuma a WatchBeat…`) y 836 (`shuul: el guardián de tus datos`). El fichero final publicado dice otra cosa (p. ej. 827 → «No Necesitas una GPU de 3000€ para IA Local», en `T08/`). `migrate-material` usa el slug del **tracker** (es su contrato), así que las carpetas heredan esos slugs. Revisar/`update`ar esos títulos es tarea aparte.
- **`/data/notas` tenía trabajo sin commitear** antes de empezar (27 borrados, 14 modificados, 9 sin seguimiento en `guiones`: `_duplicado-e832`, `_pipewire_temp`, `la-memoria-de-valet`, `presentacion-base`). El snapshot lo recoge tal cual; se asume intencional.
- **`/data` al 97 %** (54 G libres) y las 27 carpetas suman 13 G: la copia física completa era inviable, por eso el rollback va por git + `_legacy/`.

## 7. Criterio de cierre

- [x] 0 carpetas numéricas en `/data/notas/Podcasts/guiones`
- [x] `migrate-material --dry-run` (con y sin `--include-published`) sin colisiones
- [x] `podtrack where 819` y `podtrack open 819 --dry-run` resuelven la ruta del slug
- [x] Baseline y migración commiteados en `/data/notas`

## 8. Cierre — resultado

Ejecutado el 2026-10-25.

- **Cirugía**: 5 ficheros antiguos (`investigacion-0819.md`, `investigacion-0822.md`, `investigacion-0827.md`, `investigacion-0842.md`, `qa-0842.md`) y la carpeta `0863/` completa, movidos a `Podcasts/guiones/_legacy/` (11 ficheros + `README.md` con el porqué). **Nada se borró.**
- **Migración**: `podtrack migrate-material --show atareao-con-linux --include-published` → exit 0, **26 carpetas** migradas, **0 colisiones**, stderr vacío. El dry-run previo daba el mismo resultado.
- **Verificación**: 0 carpetas numéricas en `guiones/`; **52** carpetas por slug (26 previas + 26 migradas); `podtrack where` y `podtrack open --dry-run` resuelven 817, 819, 827 y 842.
- **Commits en `/data/notas`**: `3bff1c1` (snapshot), `ccb1fe7` (cirugía), `f58b778` (migración). Rollback = `git revert f58b778 ccb1fe7`.

### Hallazgo colateral (sin pérdida de datos)

596 ficheros que estaban **versionados a la fuerza** pese a `/data/notas/.gitignore` (`target/`: 568 · `__pycache__/`: 4 · `*.typshow.db*`: 24) pasaron a **no versionados** al renombrarse: git computa el renombrado como borrado + alta, y el alta se salta por regla de ignore. Siguen **en disco**.

Se comprobó que el renombrado fue puro: las 605 rutas versionadas de `0821` mapean 1:1 a ficheros existentes en su destino. Los **770 ficheros de material real** (`.md`, `.typ`, `.pdf`, `.py`, …) siguen versionados. El efecto neto es que el repo deja de versionar artefactos de compilación, que es lo que su `.gitignore` ya pedía.

### Pendiente, fuera de alcance

- `Podcasts/guiones/presentacion-base/` conserva ficheros con token numérico (`presentacion-0845.typ`, `test-0826.typ`, …): no es una carpeta de episodio, así que la migración no la toca. Decidir si se archiva o se limpia.
- Deriva tracker ↔ publicado en 827, 831, 832 y 836 (el tracker guarda el título antiguo; el fichero final publicado dice otra cosa). Requiere un pase de `update`, es tarea aparte.
