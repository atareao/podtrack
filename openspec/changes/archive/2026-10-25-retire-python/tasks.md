# Tasks

## 0. Prerrequisito: paridad cerrada antes de tocar nada

- [x] 0.1 Confirmar que las fases 1–4 están mergeadas en `development` y que `PODTRACK_REQUIRE_ORACLE=1 cargo test` pasa en verde (el oráculo vive en `~/.agents`; se comprueba antes de borrarlo).
- [x] 0.2 `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` y `just check-all` limpios.

## 1. Instalar el binario

- [x] 1.1 `cargo install --path /data/rust/podtrack --locked` → `~/.cargo/bin/podtrack`.
- [x] 1.2 `podtrack --version` responde (invocable como `podtrack`, sin `python3`).
- [x] 1.3 Pasada real de **solo lectura** contra el tracker real: `podtrack status --show atareao-con-linux`, `podtrack next --show atareao-con-linux` y `podtrack srt-stage --show atareao-con-linux --srt <SRT real> --expect final`. Sin modificar `/data/notas`.

## 2. Conmutar la skill `podcast-srt` (~/.agents)

- [x] 2.1 `SKILL.md`: `python3 /home/lorenzo/.agents/skills/podcast-srt/podcast-tracker.py <cmd>` → `podtrack <cmd>` y `python3 …/srt-stage.py` → `podtrack srt-stage`. Incluye la sección «Tracker de episodios — podcast-tracker.py» (título y rutas) y la nota de tests.
- [x] 2.2 `shows/atareao-con-linux.yaml`: quitar/actualizar el comentario de proveniencia que cita `podcast-tracker.py`.
- [x] 2.3 Revisar `shows/README.md` y el resto del skill por si citan los `.py`.

## 3. Conmutar los agentes del podcast (~/.config/opencode/agents)

- [x] 3.1 `coordinador-podcast.md`.
- [x] 3.2 `planificador-podcast-srt.md`.
- [x] 3.3 `procesador-capitulos-podcast-srt.md`.
- [x] 3.4 `notas-podcast-srt.md`.
- [x] 3.5 Revisar el resto de agentes del podcast por si invocan los `.py`.

## 4. Borrar el andamiaje Python (~/.agents)

- [x] 4.1 Borrar `podcast-tracker.py`, `srt-stage.py`, `podcast-tracker.test.py`.
- [x] 4.2 Borrar `pytest.ini`, `__pycache__/`, `.pytest_cache/`, `.ruff_cache/` del skill.
- [x] 4.3 (Opcional) Retirar del skill cualquier configuración huérfana de Python.

## 5. Constancia en `~/.agents/openspec`

- [x] 5.1 Registrar que las capacidades (parrilla, cirugía de material, clasificación de etapas del SRT) residen ahora en el binario `podtrack`.

## 6. Verificación de cierre

- [x] 6.1 **Criterio de cierre:** `grep -r '\.py' ~/.agents/skills/podcast-srt/` (ignorando `.git/`) NO devuelve nada.
- [x] 6.2 Ningún agente del podcast invoca `python3 …/*.py` (`grep -rn 'python3.*\.py' ~/.config/opencode/agents/`).
- [x] 6.3 `podtrack` instalado responde a `--version` y a un `status` real de solo lectura.
- [x] 6.4 No se han commiteado cambios ajenos en `~/.agents` (p. ej. `skills/wordcraft/SKILL.md`) ni en `~/.config/opencode`.
