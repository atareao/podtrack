# Design

## Context

`srt-stage.py` (589 líneas) es el tercer y último Python del skill `podcast-srt`. Clasifica
un SRT en una de las etapas declaradas por el perfil (`raw`/`final`) **por contenido**, y
sirve de guardián: si le pides la etapa equivocada, sale con código ≠ 0.

A diferencia de las fases anteriores, aquí:

- El show de referencia (`la-era-de-las-distros`) declara **`tracker: none`**: el perfil se
  carga para clasificar SRT, no para leer/escribir una parrilla.
- La clasificación depende de **dato del show** (regex del intro, anclaje, marcador,
  umbrales de bloque) que vive en el perfil, no en el código.
- No hay suite de referencia (`srt-stage.test.py` no existe), así que las fixtures las
  construimos nosotros.

Restricciones que dan forma al diseño:

- La salida de texto (`ruta<TAB>etapa`, `OK`/`FALLO`) y los **códigos de salida** son
  contrato con los agentes: byte a byte.
- Los tests **nunca** tocan `/data/notas` ni `/data/podcasts`.

## Goals / Non-Goals

**Goals:**

- Paridad byte a byte de `srt-stage` (stdout, stderr, *exit codes*) frente a
  `srt-stage.py`, para `--srt` y `--episode`, con y sin `--expect`.
- Reproducir la **regla del intro** exacta: dos señales con AND, marcador que basta solo,
  cues vacíos/de puntuación ignorados, `intro_anchor_chars`, `intro_block_*`.
- Cargar el perfil **sin exigir `tracker`**, con la cascada de directorios del binario.

**Non-Goals:**

- Reapuntar los agentes al binario y borrar los `.py` (fase 5).
- Cambiar el esquema del perfil: se **consume** `episodes.srt_stages` tal cual lo declaran
  los perfiles actuales.

## Decisions

### D1. Módulo propio `src/srt_stage.rs`, no una extensión del tracker

La clasificación no toca el tracker en absoluto. Se aísla en `src/srt_stage.rs` con las
funciones del oráculo (`load_stages`, `first_cue_texts`, `first_cue_span`, `normalize`,
`intro_block_ok`, `looks_like_intro`, `classify`, `srt_files_for_episode`) y se expone un
subcomando fino en `src/commands/srt_stage.rs`. Así el resto del binario no carga con la
lógica del intro.

### D2. El perfil se carga sin exigir `tracker`

El `ShowResolver` actual devuelve `ProfileNoTracker`/`ProfileMissingTrackerKey` cuando
falta `tracker`. `srt-stage` necesita lo contrario. Se añade al resolutor una vía que
localiza el fichero `shows/<slug>.yaml` (misma cascada: `PODTRACK_SHOWS_DIR` → junto al
ejecutable → `~/.config/podtrack/shows/`) y lo **parsea como YAML para las etapas**, sin la
validación de `tracker`. Es la única capacidad del binario que opera sobre un show sin
parrilla, y es correcto: la etapa es una propiedad del SRT, no del tracker.

### D3. `--episode` reproduce el `glob` del oráculo

`srt_files_for_episode` compone `episodes.srt_glob` con `{n:NN}` y hace `sorted(glob)`.
El binario SHALL imprimir la **misma ruta** que imprimiría el Python (la cadena del
fichero tal como la produce el `glob` del perfil), para que la línea `ruta<TAB>etapa` sea
idéntica. Con `--srt`, se imprime la ruta **tal cual la pasó el usuario** (como el oráculo).

### D4. Códigos de salida: `clap` solo para el parseo; el resto es contrato explícito

`--expect foo` (valor fuera de `raw|final`) y argumentos inválidos de forma los rechaza
`clap` con exit `2` (como `argparse`). El resto de `2`/`4`/`5` los emite el comando con
sus propios errores tipados. `--srt` y `--episode` excluyentes y la ausencia de ambos
también son `2`.

### D5. `normalize` reutiliza lo que ya hay

`normalize` (minúsculas, NFKD sin diacríticos, colapso de espacios) es el mismo que
`srt-stage.py`; el proyecto ya depende de `unicode-normalization` para `slugify`, así que
se reutiliza el mismo enfoque. Los patrones del intro se compilan con el crate `regex`
(ya añadido en la fase 3); el `intro_pattern` de los perfiles no usa lookbehind.

### D6. `--json` aditivo

Documento `{ schema_version, show, results: [{path, stage}], expect?, ok? }`. Sin `--json`
el texto no cambia.

## Risks / Trade-offs

- **Fixtures hechas a mano.** Al no haber suite de referencia, el riesgo es que las
  fixtures no cubran los casos reales. *Mitigación:* además de las fixtures congeladas, un
  **barrido de paridad en vivo** sobre los 32 SRT reales de `la-era-de-las-distros`
  (copiados a un temporal) y sobre los SRT de `atareao-con-linux`, contrastando `--srt` y
  `--episode` con el oráculo.
- **Deriva del perfil.** El clasificador depende de los campos de `srt_stages`; si el
  perfil cambia, la paridad se mide contra el mismo perfil en ambos lados. Se documentan
  los campos consumidos en la spec `show-profile`.
- **Rutas de glob.** La igualdad de la línea `ruta<TAB>etapa` depende de reproducir el
  `glob` del oráculo; se prueba explícitamente en la suite.
