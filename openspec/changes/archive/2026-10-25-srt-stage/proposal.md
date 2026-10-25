# Proposal

## Why

El port de `podcast-tracker.py` está cerrado (fases 1–3). Queda el **tercer y último
Python** del skill `podcast-srt`: `srt-stage.py`, el clasificador que decide de qué
**etapa de transcripción** es un SRT (`raw` = bruto de la conversación; `final` = audio
ya montado con el intro locutado). Es la pieza que hace real el «fallar en voz alta»: si
un agente pide un entregable de una etapa que el SRT no es, debe abortar con un código
distinto de cero en vez de generar capítulos con las marcas de tiempo desplazadas ~26–30 s
(un error invisible).

Portarlo a Rust con **paridad byte a byte** deja el terreno listo para la fase 5
(`retire-python`): reapuntar los agentes y borrar los tres `.py`.

## What Changes

1. **Nuevo subcomando `podtrack srt-stage`:** porta `srt-stage.py`.

   - `--show <slug> --srt <ruta> [--expect raw|final]`: clasifica **ese** fichero.
   - `--show <slug> --episode NN [--expect raw|final]`: localiza con
     `episodes.srt_glob` del perfil **todos** los SRT del episodio y los clasifica.
   - `--srt` y `--episode` son **excluyentes**; falta uno de los dos → error.
   - Salida: una línea por SRT con `ruta<TAB>etapa`; con `--expect`, una última línea
     `OK` o `FALLO`.

2. **Clasificación por contenido, con las dos señales del intro.** La etapa se decide
   **nunca** por el nombre del fichero. El `final` se reconoce por **dos señales
   independientes combinadas con un AND**: la de **texto** (el primer cue con texto real
   coincide con `intro_pattern` empezando dentro de `intro_anchor_chars`, exigiendo el
   nombre del programa en el patrón) y la de **bloque** (el primer cue del fichero arranca
   en ~00:00:00 y dura `>= intro_block_min_seconds`). El marcador literal `intro_marker`
   basta por sí solo. Se ignoran los cues iniciales vacíos o de solo puntuación.

3. **Códigos de salida como contrato:** `0` éxito; `2` perfil/SRT/argumentos
   ilegibles, no encontrados o inválidos (incluye `detect` desconocido, perfil sin
   `srt_stages`, fichero sin cues y `--expect` de una etapa no declarada); `4` con
   `--episode`, ningún SRT del episodio es de la etapa esperada; `5` con `--srt`, el
   fichero se clasificó pero no es la etapa esperada.

4. **Lectura del perfil sin exigir tracker.** El comando SHALL resolver `shows/<slug>.yaml`
   con la misma cascada que el resto del binario, pero **sin exigir la clave `tracker`**
   (un show como `la-era-de-las-distros` declara `tracker: none` y aun así se clasifica).
   Se parsean `episodes.root`, `episodes.srt_glob` y `episodes.srt_stages`.

5. **`--json` opt-in (aditivo):** documento con el show, los SRT clasificados y, con
   `--expect`, el resultado. Sin `--json`, la salida de texto NO SHALL cambiar.

## Capabilities

### New Capabilities
- `srt-stage`: clasificación de la etapa de un SRT por contenido, con las dos señales del
  intro, la localización de SRT de un episodio y los códigos de salida de contrato.

### Modified Capabilities
- `show-profile`: se documenta la lectura del perfil **sin exigir `tracker`** para el
  camino de `srt-stage`, y los campos de `episodes.srt_stages` que consume.

## Impact

- **`src/srt_stage.rs`** (nuevo) — port de `srt-stage.py`: carga del perfil de etapas,
  extracción de cues, `normalize`, las dos señales (`looks_like_intro`, `intro_block_ok`),
  `classify` y la localización por glob.
- **`src/commands/srt_stage.rs`** (nuevo) — el subcomando y su reportería/exit codes.
- **`src/profile.rs`** — se añade una carga de perfil que no exige `tracker` (o un
  parámetro para relajarlo), reutilizando la cascada de directorios.
- **`src/main.rs`** — el subcomando `srt-stage` y sus banderas.
- **`src/json.rs`** — documento de `--json`.
- **`tests/`** — suite de paridad de clasificación (fixtures SRT congelados + contraste en
  vivo con el oráculo sobre los SRT reales copiados a un temporal).
- **No** se toca `/data/notas` ni `/data/podcasts`; los tests usan fixtures temporales.
  El Python sigue en producción hasta la fase 5.
