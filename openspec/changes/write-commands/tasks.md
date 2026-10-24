# Tasks

## 0. Prerrequisito: andamiaje

- [x] 0.1 Añadir a `model.rs` la mutación del dominio que le falta (derive `Clone` del `Tracker`/`Episode` para el `--dry-run`, constructores de alta, helpers de número provisional/autoritativo ya existentes).
- [x] 0.2 Crear `src/material.rs` con `renamed_component`, `plan_number_renames` y `apply_renames`.
- [x] 0.3 `cargo check` limpio.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 Portar a `tests/` los escenarios del oráculo de `add`, `update`, `publish`, `insert` y `move` (texto + *exit codes* + tracker resultante) sobre fixtures temporales.
- [x] 1.2 Test del renombrado de material: tokens `0NNN`/`eNNN`, no tocar secuencias más largas, orden del plan, aborto por destino existente, reporte de cada par.
- [x] 1.3 Test de `--dry-run`: `publish`/`insert`/`move` no escriben el tracker, no crean `.bak` y no renombran material.
- [x] 1.4 Test de semántica del número: `publish` estampa `max(publicados)+1`, no reutiliza, y anula el provisional obsoleto ajeno.
- [x] 1.5 `json-output`: test de los documentos de escritura y de `--dry-run --json`.
- [x] 1.6 `cargo test`: las pruebas nuevas fallan por no existir el código.

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/material.rs`: renombrado de material.
- [x] 2.2 `commands/add.rs`, `commands/update.rs`: alta y cambio de estado.
- [x] 2.3 `commands/publish.rs`: estampado del número autoritativo.
- [x] 2.4 `commands/insert.rs`: desplazamiento de números, fechas y material.
- [x] 2.5 `commands/move.rs`: cambio de fecha.
- [x] 2.6 `main.rs`: subcomandos y banderas nuevas (`--before`, `--title`, `--slug`, `--date`, `--season`, `--dry-run`).
- [x] 2.7 `json.rs`: documentos de escritura.
- [x] 2.8 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en producción.
- [x] 3.3 Ninguna escritura fuera de `save_tracker`; ningún test toca `/data/notas`.

## 4. Verificación de paridad (operador)

- [x] 4.1 Para cada comando, comparar con `diff` la salida y el **tracker resultante** del binario y del Python sobre el **mismo fixture** congelado; SHALL ser idénticos.
- [x] 4.2 Comprobar los *exit codes* de los casos de error contra los del Python.
- [x] 4.3 `podtrack publish <slug> --json --show <fixture>` devuelve JSON válido (`jq .`).
- [x] 4.4 `just check-all` en verde.
