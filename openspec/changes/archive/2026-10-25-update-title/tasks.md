# Tasks

## 0. Contrato de API

- [x] 0.1 `src/commands/update.rs`: `UpdateParams.status` pasa a `Option<String>` y gana `title: Option<String>`.
- [x] 0.2 `src/main.rs`: `--status` opcional y `--title` nuevo, con exigencia mutua (al menos uno).
- [x] 0.3 `src/error.rs`: variante para título vacío.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/parity_write.rs`: `update --title` cambia el título y **conserva el slug**.
- [x] 1.2 `update --status --title` cambia ambos.
- [x] 1.3 `update --title "   "` falla sin escribir.
- [x] 1.4 `tests/write_json.rs`: `--json` de `update --title` expone `changes[field=title]`.
- [x] 1.5 Ajustar los `UpdateParams` existentes a `status: Some(...)`.
- [x] 1.6 `cargo test`: las pruebas nuevas fallan.

## 2. GREEN (implementación mínima)

- [x] 2.1 `update::run` aplica estado y/o título, validando antes de escribir.
- [x] 2.2 `json::update_document` emite la lista de cambios (status y/o title).
- [x] 2.3 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`/`expect()`/`panic!()` en producción.
- [x] 3.3 La salida de texto de `update --status` queda **byte a byte** igual a la actual.
- [x] 3.4 Ningún test toca `/data/notas` ni `/data/podcasts`.

## 4. Archivo

- [x] 4.1 `openspec validate update-title --strict`.
- [x] 4.2 `just check-all` en verde.
- [x] 4.3 `just spec-archive update-title`.
