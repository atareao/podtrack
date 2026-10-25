# Tasks

## 0. Prerrequisito

- [x] 0.1 `src/model.rs`: extraer `unique_slug` (base + sufijado `-2`, `-3`, …) como
      helper público reutilizable, con tests unitarios.
- [x] 0.2 `src/commands/add.rs` y `src/commands/backfill_slugs.rs`: consumir el helper
      común sin cambiar conducta.
- [x] 0.3 `cargo check` limpio.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/parity_write.rs`: `insert` sin `--slug` deriva el slug del título.
- [x] 1.2 Caso de colisión → el slug derivado se sufija (`-2`).
- [x] 1.3 Caso de título sin slug utilizable → el episodio queda sin slug.
- [x] 1.4 Caso con `--slug` explícito → se usa tal cual (sin sufijo).
- [x] 1.5 `cargo test`: las pruebas nuevas fallan (la conducta actual deja `slug: null`).

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/commands/insert.rs`: derivar el slug del título cuando falte `--slug`.
- [x] 2.2 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en producción.
- [x] 3.3 Ningún test toca `/data/notas` ni `/data/podcasts` (fixtures temporales).
- [x] 3.4 `tests/parity_write_oracle.rs`: fijar `--slug` en los casos de `insert` y
      documentar la divergencia.
- [x] 3.5 Pasada manual en dry-run: `insert` previsualiza sin escribir.

## 4. Archivo

- [x] 4.1 `openspec validate insert-derives-slug --strict`.
- [x] 4.2 `just check-all` en verde.
- [x] 4.3 `just spec-archive insert-derives-slug`.
