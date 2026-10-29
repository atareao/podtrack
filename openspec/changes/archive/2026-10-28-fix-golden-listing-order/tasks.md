# Tasks

## 0. Prerrequisito

- [x] 0.1 La base es `main` @ `91fe756` (`chore: release v0.6.0`), con la suite en **363 passed / 0 failed** y el CI de `main` en rojo **solo** por el flake del comparador (run `40655346111`, con `development` verde en `40655350954` sobre el mismo árbol). Rama de trabajo: `hotfix/golden-listing-order`, creada desde `main`. — confirmado antes de tocar nada: 363/0 y el árbol limpio salvo el testigo sin trackear.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/golden_listing_order.rs` (fichero nuevo, con `mod common;`): fija el contrato del comparador reutilizando un golden real como material sintético (se reordena su propia salida y se vuelve a comparar). — el esperado es el golden **prístino** y el actual la salida **mutada**: la comparación no es tautológica.
- [x] 1.2 Permutar las líneas de una tirada **dentro de su grupo de profundidad** NO falla.
- [x] 1.3 Duplicar una línea del plan falla (`#[should_panic]`).
- [x] 1.4 Quitar una línea del plan falla (`#[should_panic]`).
- [x] 1.5 Alterar el destino de una línea del plan falla (`#[should_panic]`).
- [x] 1.6 Intercambiar una línea de un grupo de profundidad con otra de otro grupo falla (`#[should_panic]`).
- [x] 1.7 Permutar líneas que **no** son del plan (registros de `rebase-references`) falla (`#[should_panic]`).
- [x] 1.8 `cargo test --locked`: 1.2 falla (el comparador todavía compara en orden exacto) y los **363 existentes siguen verdes**. — RED medido con el comparador intacto: `5 passed; 1 failed`, fallando **solo** `permutar_hermanos_de_una_tirada_no_falla` con «stdout difiere [--show demo rename-material --from 101 --delta 1]», y los 363 previos intactos.

## 2. GREEN (implementación mínima)

- [x] 2.1 `tests/common/golden.rs`: ayudante privado que agrupa por profundidad dentro de cada tirada máxima del plan y ordena cada grupo antes de comparar. — `plan_depth` + `canonical_run` + `canonical_plan_runs`.
- [x] 2.2 `assert_outcome` compara `self.stdout` y `self.stderr` contra la salida **canonizada**; nada más cambia.
- [x] 2.3 Doc-comment del módulo: declara la excepción acotada y por qué existe.
- [x] 2.4 `cargo test --locked` en verde: 363 + 6. — **369 passed / 0 failed / 0 ignored**.
- [x] 2.5 `git status --porcelain tests/golden/` vacío (el golden no se toca). — vacío.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Ningún `unwrap()`, `expect()` ni `panic!()` nuevo en el ayudante.
- [x] 3.3 Determinismo comprobado con **dos `TMPDIR` distintos** (el del sistema y `/dev/shm`): `TMPDIR=/dev/shm cargo test --locked` en verde. — 369/0 en `tmpfs`, y 10 pasadas alternando `TMPDIR` por defecto y `/dev/shm` sobre los cuatro binarios de paridad afectados: 57/0 en todas.
- [x] 3.4 Ningún test toca `/data/notas`.
- [x] 3.5 `just check-all` en verde. — fmt, clippy, 369 tests y rustdoc, todo verde.

## 4. Verificación y cierre

- [x] 4.1 Los 363 tests previos pasan sin editar ningún caso de paridad ni el golden (el único fichero de tests tocado es el comparador compartido, `tests/common/golden.rs`). — el diff tracked se reduce a `tests/common/golden.rs` (71 inserciones / 4 borrados) más el fichero nuevo.
- [x] 4.2 `just spec-validate fix-golden-listing-order` en estricto. — `Change 'fix-golden-listing-order' is valid`.
- [x] 4.3 Auditoría con el subagente `rust-reviewer` sobre el diff completo (solo lectura). — una pasada, **sin hallazgos bloqueantes ni mayores**; los menores (redacción de D2/D3, fuerza real del unitario, independencia del predicado del test, riesgo latente de `slug_collisions`) quedaron corregidos o anotados.
- [x] 4.4 Commit `🐛 fix(test): …` (`8299d7b`) y PR **#33** `hotfix/golden-listing-order` → `main`. — PR `CLEAN`, CI verde.
- [x] 4.5 Merge a `main` (`4db696e`); release patch **`v0.6.1`** (`6514ad9`) con tag, `CHANGELOG.md`, `Cargo.toml`, `.vampus.yml`, crates.io y GitHub Release, y `development` sincronizado.
- [x] 4.6 El CI de `main` (push de la release) queda **verde**. — run `40753974938` `success`; `development` (`40753983858`) `success`; workflow Release `40753977380` con Preflight, Publish crate y GitHub Release en `success`.
- [x] 4.7 `just spec-archive fix-golden-listing-order`. — archivado como `openspec/changes/archive/2026-10-28-fix-golden-listing-order/`; delta fundido en `openspec/specs/python-retirement/spec.md`.
