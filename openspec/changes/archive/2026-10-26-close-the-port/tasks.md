# Tasks

> **Pausa resuelta (2026-10-25).** La decisión pendiente sobre la divergencia de `get` se tomó
> (decisión **A**, nota bajo 3.3) y las fases 2 (grabación), 3 (GREEN) y 4 (REFACTOR) están hechas.
> Testigo completo: `plans/PLAN-008-COMPLETED.md` §9.

## 0. Prerrequisito — recuperar el oráculo (temporal, fuera del repo)

- [x] 0.1 Recuperar los tres `.py` del histórico de `~/.agents`:
      `git -C ~/.agents cat-file -p c70287f^:skills/podcast-srt/<fichero>` hacia
      `/tmp/opencode/oracle-recovery/`. Comprobar bytes/líneas: `podcast-tracker.py` 65 617/1 813,
      `srt-stage.py` 23 983/589, `podcast-tracker.test.py` 62 166/1 817.
- [x] 0.2 Verificar el entorno: `python3 --version` (3.14.8) y `python3 -c "import yaml"`
      (PyYAML 6.0.3).
- [x] 0.3 Cargar el módulo recuperado y confirmar que expone los 17 `cmd_*` (de `cmd_add` a
      `cmd_where`).
- [x] 0.4 Registrar los 27 casos por fichero: `parity_oracle.rs` (2), `parity_write_oracle.rs` (9),
      `parity_writer.rs` (2), `parity_surgery.rs` (6), `srt_stage.rs` (7), `rebase_references.rs`
      (1).
- [x] 0.5 Medir la baseline y dejarla anotada: `just check-all` verde, `cargo test` sin variables →
      339/0, `PODTRACK_REQUIRE_ORACLE=1 cargo test --no-fail-fast` → 312/27.

## 1. RED — los 27 tests leen el golden y fallan por golden ausente

- [x] 1.1 Fijar la forma del golden (D4) y el lector en `tests/common/mod.rs`: ruta
      `<CARGO_MANIFEST_DIR>/tests/golden/<área>/<caso>.json`, lectura en tiempo de ejecución y
      `panic` con el nombre del fichero si falta o no parsea. Sin `Option` ni ruta de salto.
- [x] 1.2 Reescribir los 27 en su sitio, con los mismos nombres (D3): `parity_oracle.rs`,
      `parity_write_oracle.rs`, `parity_writer.rs`, `parity_surgery.rs`, `srt_stage.rs`,
      `rebase_references.rs`. Solo cambia el lado «esperado».
- [x] 1.3 Ejecutar la suite y comprobar que los 27 fallan **por golden ausente** (no por otra
      causa). Pegar la evidencia.
- [x] 1.4 Confirmar que los 312 que ya comprobaban algo siguen verdes **sin editarlos**.

## 2. Grabación (one-off; el código NO se commitea)

- [x] 2.1 Añadir el modo temporal de grabación que serializa el `Outcome` del oráculo sobre la misma
      matriz (`code`, `stdout`, `stderr`, tracker y material donde proceda, bytes del writer).
- [x] 2.2 Grabar `tests/golden/` con el oráculo recuperado (D1), nunca con el binario.
- [x] 2.3 Revisar la cobertura: están los 27 casos y las superficies de D4 (tracker resultante,
      árbol de material, bytes del writer).
- [x] 2.4 Retirar el código de grabación y verificar que `git status` no lo muestra; comprobar que
      no queda ninguna receta ni arnés dormante que dependa de un `.py`.
- [x] 2.5 Revisar una muestra del golden contra el oráculo y confirmar que el origen es el oráculo
      (`c70287f^`), no el binario.

## 3. GREEN

- [x] 3.1 `cargo test` **sin** ninguna variable `PODTRACK_*` → **340** passed / 0 failed / 0 saltados (los 339 previos + el guardián).
- [x] 3.2 Confirmar que ningún test se salta: no debe existir ninguna ruta de `return` temprano ni
      ningún `skip:` por oráculo.
- [x] 3.3 `just check-all` en verde.
      **Evidencia medida:** decisión **A** — excluir `get 100`/`get 101` **solo** en el test del
      fixture `mini`, con la exclusión pasada por test porque `SPECS` es compartida con `v2`. Golden
      re-grabado con el oráculo → **27 ficheros / 128 escenarios**; huella de árbol
      `a02b6a69d041e13163455c5e81fb46e6e2e0a42d7b1f272fd618e1753dcf851f` (la anterior `6df44684…`
      queda invalidada), 0 rutas temporales y determinismo verificado (`diff -r` idéntico).

## 4. REFACTOR — retirada del arnés y guardián

- [x] 4.1 Retirar de `tests/common/mod.rs` `oracle_path`, `python_ready`, `require_oracle`,
      `require_oracle_srt`, `ORACLE_PREAMBLE`, `SRT_MAIN_HARNESS` y todo lo que solo ellos usaban.
      Comprobar que no queda ningún helper huérfano.
- [x] 4.2 Retirar las variables `PODTRACK_REQUIRE_ORACLE`, `PODTRACK_ORACLE` y
      `PODTRACK_ORACLE_SRT`.
- [x] 4.3 Guardián de no-Python (D5): falla si reaparece `python3`, un `.py` del oráculo o una
      variable `PODTRACK_ORACLE*` bajo `tests/`.
      **Nota (auditoría 2026-10-26):** el guardián se auditó con nueve sondas y se le cerraron
      **cuatro** agujeros —(1) una invocación del intérprete sin el literal `python3`
      (`Command::new("python")`) escapaba; (2) solo se escaneaban `*.rs`, así que un script de shell
      bajo `tests/` pasaba; (3) los errores de lectura se tragaban en silencio y el guardián pasaba
      en vacío ante un fichero o directorio ilegible; (4) un directorio llamado `*.py` no se veía— y
      las cuatro se cerraron con el diseño de dos familias (referencias inequívocas en todo fichero;
      el intérprete, por nombre sin versión, en todo fichero salvo `tests/fixtures/**` y
      `tests/golden/**`) y la regla «un error de lectura es una infracción».
      **Segunda pasada (2026-10-26):** el revisor volvió sobre el guardián **ya reparado** y lo dio
      por **cerrado**: **14/14 sondas detectadas**, sin un solo agujero nuevo de severidad bloqueante
      ni importante y sin romper nada de lo declarado limpio (340/0/0, golden `a02b6a69…` intacto,
      `src/` solo comentarios, `clippy -D warnings` y `fmt --check` limpios). Quedan **dos reservas
      menores**: **N-1** —la exclusión de `tests/fixtures/**` en la familia B pasa de implícita a
      **declarada**, con su residuo (un script plantado a mano bajo `fixtures/`) anotado como límite
      conocido— y **N-2a** —un fichero llamado exactamente `.py` (sin tallo) escapaba porque se miraba
      la extensión; el código pasa a comprobar el **nombre completo** («termina en `.py`», sin
      distinguir mayúsculas) en ficheros y directorios—, **cerrada**. **N-2b** —un directorio llamado
      con un token de la familia A (p. ej. `tests/PODTRACK_ORACLE/`) no se detecta— queda como
      **observación asumida**, sin reparación (exigiría mala fe). Ver D5 y `plans/PLAN-009.md` §9.7.
- [x] 4.4 Cabecera de cada uno de los seis ficheros: documentar que el golden se grabó con el
      oráculo recuperado (`c70287f^`) y que ya no hay paridad viva.
- [x] 4.5 `just check-all` y `just fmt-fix` si hiciera falta.

## 5. F — documentación

- [x] 5.1 `AGENTS.md`: líneas 7, 21-22, 27, 29 y la sección 43-53. Sustituir «Trabajo en fases» por
      el estado final (las cinco fases archivadas, con puntero a `openspec/changes/archive/`).
- [x] 5.2 `README.md:19`: sin prometer la retirada futura del script.
- [x] 5.3 `GIT_FLOW.md:5`: última release `v0.5.1`; barrido para descartar otras afirmaciones
      desfasadas: vocabulario del port en marcha («hoy vive», «durante el port», «se derriba»,
      «sigue siendo el que corre», «v0.5.0») ausente de los documentos **vivos** (`AGENTS.md`,
      `README.md`, `GIT_FLOW.md`, los doc-comments de `src/` y cualquier documento que **afirme** el
      estado del proyecto), con
      lista blanca de históricos: `CHANGELOG.md`, `plans/*-COMPLETED.md`,
      `openspec/changes/archive/` y `openspec/specs/python-retirement/spec.md`.
      **Nota:** el barrido cubre también `docs/DEFINITION.md`, que no figuraba en la lista y se
      contradecía (§12 cerrada frente a §1/§2/§10/§11 en presente). Las coincidencias que quedan
      tras el barrido son la lista blanca de históricos y las citas en negativo de los testigos del
      change (`plans/PLAN-008-COMPLETED.md`, `plans/PLAN-009.md`) y de sus propios artefactos: no son
      afirmaciones desfasadas.
      **Ampliación de la lista blanca (H-5):** el escenario «barrido final sin afirmaciones
      desfasadas» del delta se amplió para que la lista blanca incluya también los artefactos del
      propio change y los testigos de sesión (`plans/PLAN-NNN.md`), precisando que se admiten citas
      en negativo —el vocabulario entrecomillado para prohibirlo o corregirlo— y no afirmaciones. La
      exigencia de fondo sobre los documentos vivos de verdad no se relaja.
- [x] 5.4 Doc-comments de `src/lib.rs:3`, `src/error.rs:3,164` y `src/srt_stage.rs:3`: el port en
      pasado, sin prometer trabajo futuro. Sin tocar ninguna línea de lógica.

## 6. Verificación de cierre

- [x] 6.1 `cargo test` sin variables → **340** passed / 0 failed / 0 saltados (los 339 previos + el
      guardián); `just check-all` verde.
      **Evidencia medida (2026-10-26):** `cargo test --locked` **sin ninguna variable `PODTRACK_*`**
      → **340 passed / 0 failed / 0 ignorados**, ningún target fallido; `just check-all` verde
      (fmt + clippy `-D warnings` + test + rustdoc). Verificado el **2026-10-26** tras las
      reparaciones de la auditoría.
- [x] 6.2 `grep -rn 'python3\|PODTRACK_REQUIRE_ORACLE\|PODTRACK_ORACLE' tests/` → vacío.
- [x] 6.3 `openspec validate close-the-port --strict` OK.
      **Evidencia medida:** `openspec validate close-the-port --strict` → `Change 'close-the-port'
      is valid`, ejecutado **antes** de archivar. Tras el archivado, `openspec validate --specs
      --strict` → **13 passed / 0 failed**.
- [x] 6.4 `/data/notas` intacto: solo lectura; el golden se grabó sobre fixtures sintéticos.
      **Evidencia medida:** `/data/notas` **intacto**: `git -C /data/notas status --porcelain |
      sort` **idéntico** antes y después de la matriz de comprobación (61 paths sucios
      preexistentes, de la producción del episodio; `diff` vacío). El golden se grabó sobre
      fixtures **temporales**.
- [x] 6.5 Paridad con datos reales como comprobación de no-regresión: 14/14 contra `v0.5.1`
      (procedimiento de `PLAN-007`). Se hará contra `v0.5.1` (la última release, etiqueta `v0.5.1`, `main` en `chore: release v0.5.1`), no contra `v0.5.0`, y se ejecuta en la verificación final.
      **Evidencia medida:** **14/14** invocaciones **idénticas** frente al binario de la última
      release (`v0.5.1`, en un worktree sobre el tag, ambos compilados en release): `stdout`,
      `stderr` y código de salida coinciden byte a byte en `status` (texto y `--json`), `next`,
      `list --last 5`, `get` (843 texto y `--json`, 819, 827 texto y `--json`), `where` (843, 819,
      827) y `open 843 --dry-run` (texto y `--json`). Toda la matriz es de **solo lectura**.

## Release

- Rama `feature/close-the-port` → PR a `development` → merge → PR `development → main`.
- Conventional commits `test`/`docs` (gitmoji): no hay cambio de comportamiento, así que no hay
  bump de versión.
