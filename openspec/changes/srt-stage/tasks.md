# Tasks

## 0. Prerrequisito: andamiaje

- [x] 0.1 `src/profile.rs`: vía de resolución que localiza `shows/<slug>.yaml` con la cascada del binario y lo parsea **sin exigir `tracker`**.
- [x] 0.2 `src/srt_stage.rs`: estructura de etapas (`order` + campos por etapa), parseo de `episodes.root`/`episodes.srt_glob`/`episodes.srt_stages`, compilación del `intro_pattern` con `regex` y validaciones (detect desconocido, intro inválido, etc.).
- [x] 0.3 `cargo check` limpio.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/srt_stage.rs`: fixtures SRT congelados (raw con cues cortos; final con intro de ~26 s; final con cue inicial `-`; final con intro partido en dos cues; final con marcador `INTRO`; fórmula casual sin el nombre; fichero sin cues; fichero vacío) y su etapa esperada.
- [x] 1.2 Fixtures de perfil: uno de una etapa (`detect: any`), uno de dos etapas (`not_intro`/`starts_with_intro` con las señales), uno con `tracker: none`, uno con `detect` desconocido y uno sin `srt_stages`.
- [x] 1.3 Contraste en vivo con el oráculo (`srt-stage.py`) sobre los mismos fixtures: stdout, stderr y *exit codes*, para `--srt` y `--episode`, con y sin `--expect`.
- [x] 1.4 Casos de código de salida: `2` (argumentos, perfil, fichero sin cues, `--expect` no declarada), `4` (`--episode` sin la etapa) y `5` (`--srt` de otra etapa).
- [x] 1.5 `--json`: documento con resultados y resultado de `--expect`; y "sin `--json` el texto no cambia".
- [x] 1.6 `cargo test`: las pruebas nuevas fallan por no existir el código.

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/srt_stage.rs`: `first_cue_texts`, `first_cue_span`, `normalize`, `intro_block_ok`, `looks_like_intro`, `classify`, `srt_files_for_episode`.
- [x] 2.2 `src/commands/srt_stage.rs`: el subcomando, la reportería (`ruta<TAB>etapa`, `OK`/`FALLO`) y los códigos de salida.
- [x] 2.3 `src/main.rs`: subcomando `srt-stage` con `--srt`, `--episode`, `--expect`.
- [x] 2.4 `src/json.rs`: documento de `--json`.
- [x] 2.5 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en producción.
- [x] 3.3 Ningún test toca `/data/notas` ni `/data/podcasts` (fixtures temporales).

## 4. Verificación de paridad (operador)

- [x] 4.1 Barrido en vivo sobre los 32 SRT reales de `la-era-de-las-distros` (copiados a un temporal) con `--srt`: stdout + exit idénticos al oráculo.
- [x] 4.2 Barrido de `--episode NN` sobre los episodios reales: la línea `ruta<TAB>etapa` idéntica al oráculo.
- [x] 4.3 `--expect raw|final` en ambos sentidos (acierto y fallo → `4`/`5`) idéntico al oráculo.
- [x] 4.4 `atareao-con-linux` (una sola etapa `detect: any`): `--srt` sobre un SRT real de `transcripciones/` → `final`, idéntico al oráculo.
- [x] 4.5 `podtrack srt-stage ... --json` devuelve JSON válido (`jq .`).
- [x] 4.6 `just check-all` en verde y guardián `PODTRACK_REQUIRE_ORACLE=1` en verde.
