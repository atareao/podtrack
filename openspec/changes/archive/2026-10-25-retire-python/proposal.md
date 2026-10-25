# Proposal

## Why

Las fases 1–4 portaron `podcast-tracker.py` y `srt-stage.py` al binario `podtrack`,
con paridad byte a byte. Pero el port **no termina cuando el binario funciona:
termina cuando no queda Python**. Hoy la skill `podcast-srt` sigue teniendo tres
ficheros Python que **son conmutados por los agentes en producción**, de modo que
el binario es código muerto mientras el Python siga mandando.

Esta fase cierra el port: conmuta el `SKILL.md` y los agentes del podcast de las
invocaciones `python3 …/xxx.py` a `podtrack`, instala el binario, **borra** los
tres `.py` y su andamiaje (caches, `pytest.ini`, config de ruff), y deja
constancia en `~/.agents/openspec` de que las capacidades pasaron al binario.

## What Changes

1. **Publicar/instalar el binario.** `podtrack` SHALL instalarse con
   `cargo install --path . --locked` (desde `/data/rust/podtrack`), quedando
   invocable como `podtrack` (en `~/.cargo/bin`, ya en `PATH`). No se publica en
   crates.io (`publish = false`): la instalación es local.

2. **Conmutar la skill `podcast-srt`.** `SKILL.md` SHALL invocar `podtrack` en vez
   de `python3 /home/lorenzo/.agents/skills/podcast-srt/{podcast-tracker,srt-stage}.py`.
   El comentario de proveniencia de `shows/atareao-con-linux.yaml` que cita el `.py`
   SHALL actualizarse.

3. **Conmutar los agentes del podcast.** `coordinador-podcast.md`,
   `planificador-podcast-srt.md`, `procesador-capitulos-podcast-srt.md` y
   `notas-podcast-srt.md` (en `~/.config/opencode/agents/`) SHALL invocar `podtrack`.

4. **Borrar el andamiaje Python.** Los tres `.py` (`podcast-tracker.py`,
   `podcast-tracker.test.py`, `srt-stage.py`) y su configuración huérfana
   (`pytest.ini`, `__pycache__/`, `.pytest_cache/`, `.ruff_cache/`) SHALL eliminarse.

5. **Constancia en `~/.agents/openspec`.** Se registrará que las capacidades
   (parrilla, cirugía de material, clasificación de etapas del SRT) residen ahora
   en el binario `podtrack`.

## Capabilities

### New Capabilities
- `python-retirement`: el invariante de «Python cero» en la skill `podcast-srt`, la
  invocación por `podtrack` y la instalación local del binario.

## Impact

- **`~/.agents/skills/podcast-srt/`** — `SKILL.md`, `shows/atareao-con-linux.yaml`;
  se borran los `.py` y el andamiaje.
- **`~/.config/opencode/agents/`** — los cuatro agentes del podcast.
- **`~/.agents/openspec/`** — constancia del traslado de capacidades.
- **`/data/rust/podtrack`** — sin cambios de código; solo este change y, si acaso,
  documentación.
- **No** se toca `/data/notas`. No se commitean cambios ajenos preexistentes en
  `~/.agents` (`skills/wordcraft/SKILL.md`) ni en `~/.config/opencode`.

## Criterio de cierre

`grep -r '\.py' ~/.agents/skills/podcast-srt/` no devuelve nada.
