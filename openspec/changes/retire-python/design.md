# Design

## Context

`podtrack` es el port en Rust del tooling Python del podcast, con paridad byte a byte.
Las fases 1–4 dejaron el binario funcional; el Python sigue siendo lo que los agentes
invocan en producción. Esta fase es la **retirada**: conmutar y borrar.

Trabajo **cross-repo**:

- `/data/rust/podtrack` — el binario y este change.
- `~/.agents` — la skill `podcast-srt` (SKILL.md, perfiles, los `.py`) y su `openspec/`.
- `~/.config/opencode` — los agentes del podcast.

Restricciones:

- **No** commitean cambios ajenos preexistentes en `~/.agents`
  (`M skills/wordcraft/SKILL.md`) ni en `~/.config/opencode` (borrados y `.opencode/`).
- El binario debe quedar en un path estable que los agentes invoquen sin rutas
  largas frágiles.
- El Python es el **oráculo de paridad** hasta este mismo momento: no se borra antes
  de conmutar.

## Goals / Non-Goals

**Goals:**

- `podtrack` invocable como `podtrack` desde cualquier sitio (instalado).
- Cero apariciones de `.py` en `~/.agents/skills/podcast-srt/`.
- `SKILL.md` y los cuatro agentes del podcast invocando `podtrack`.
- Constancia en `~/.agents/openspec` de que las capacidades pasaron al binario.

**Non-Goals:**

- Cambiar la funcionalidad del binario (eso lo hicieron las fases 1–4).
- Publicar en crates.io (`publish = false`).
- Tocar `/data/notas`.

## Decisions

### D1. Instalación local con `cargo install --path . --locked`

El binario se instala con `cargo install --path . --locked` desde `/data/rust/podtrack`,
quedando en `~/.cargo/bin/podtrack`. Los agentes invocan `podtrack` **a secas**
(`~/.cargo/bin` ya está en `PATH`). Se elige esto frente a una ruta fija porque `PATH`
es el mecanismo estándar y no ata los agentes a una ruta del árbol de trabajo.
Alternativa descartada: copiar el binario a mano (envejece sin actualizarse).

### D2. La conmutación es textual y verificable

La conmutación de `SKILL.md` y de los agentes es **sustitución de la invocación**:
`python3 …/podcast-tracker.py <cmd>` → `podtrack <cmd>` y
`python3 …/srt-stage.py` → `podtrack srt-stage`. `--show <slug>` sigue funcionando
porque en `podtrack` es una bandera **global** (aceptada antes o después del
subcomando), igual que la aceptaba el oráculo. Se conserva el resto del texto.

### D3. Se borra el andamiaje, no solo los `.py`

Además de los tres `.py`, se eliminan `pytest.ini`, `__pycache__/`, `.pytest_cache/` y
`.ruff_cache/`: son huérfanos sin los ficheros que configuraban, y su presencia haría
que `grep -r '\.py'` siguiera devolviendo ruido.

### D4. El criterio de cierre es mecánico

`grep -r '\.py' ~/.agents/skills/podcast-srt/` sin resultados. Se comprueba ignorando
`.git/`.

### D5. La verificación en producción es una pasada real de solo lectura

El paso «una publicación real sin Python» de la SRS se cubre con una pasada real de
**solo lectura** del binario instalado contra el tracker real (`status`, `next`,
`srt-stage` sobre un SRT real): confirma que el binario instalado lee el estado real y
clasifica bien. No se modifica `/data/notas`.

## Risks / Trade-offs

- **Borrar el oráculo de paridad.** Una vez borrados, no hay contra qué comparar.
  *Mitigación:* solo se borra **después** de conmutar y de que los 244 tests con
  `PODTRACK_REQUIRE_ORACLE=1` pasen (el oráculo vive también en `~/.agents`, y en el
  repo podtrack los tests lo consultan por ruta). Se documenta que el oráculo deja de
  estar disponible.
- **Multi-repo.** Los cambios se reparten entre tres repos git. *Mitigación:* cada
  repo conserva su rama/commit; no se mezclan los cambios ajenos.
- **Agentes desincronizados.** Si un agente olvidado sigue invocando el `.py`, la
  producción rompería al borrarlo. *Mitigación:* el grep de cierre cubre el skill; la
  conmutación de los agentes se revisa uno a uno.
