# Tasks

## 0. Prerrequisito

- [ ] 0.1 La base es `main` @ `91fe756` (`chore: release v0.6.0`), con la suite en **363 passed / 0 failed** y el CI de `main` en rojo **solo** por el flake del comparador (run `40655346111`, con `development` verde en `40655350954` sobre el mismo árbol). Rama de trabajo: `hotfix/golden-listing-order`, creada desde `main`.

## 1. RED (pruebas que fallan primero)

- [ ] 1.1 `tests/golden_listing_order.rs` (fichero nuevo, con `mod common;`): fija el contrato del comparador reutilizando un golden real como material sintético (se reordena su propia salida y se vuelve a comparar).
- [ ] 1.2 Permutar las líneas de una tirada **dentro de su grupo de profundidad** NO falla.
- [ ] 1.3 Duplicar una línea del plan falla (`#[should_panic]`).
- [ ] 1.4 Quitar una línea del plan falla (`#[should_panic]`).
- [ ] 1.5 Alterar el destino de una línea del plan falla (`#[should_panic]`).
- [ ] 1.6 Intercambiar una línea de un grupo de profundidad con otra de otro grupo falla (`#[should_panic]`).
- [ ] 1.7 Permutar líneas que **no** son del plan (registros de `rebase-references`) falla (`#[should_panic]`).
- [ ] 1.8 `cargo test --locked`: 1.2 falla (el comparador todavía compara en orden exacto) y los **363 existentes siguen verdes**.

## 2. GREEN (implementación mínima)

- [ ] 2.1 `tests/common/golden.rs`: ayudante privado que agrupa por profundidad dentro de cada tirada máxima del plan y ordena cada grupo antes de comparar.
- [ ] 2.2 `assert_outcome` compara `self.stdout` y `self.stderr` contra la salida **canonizada**; nada más cambia.
- [ ] 2.3 Doc-comment del módulo: declara la excepción acotada y por qué existe.
- [ ] 2.4 `cargo test --locked` en verde: 363 + 6.
- [ ] 2.5 `git status --porcelain tests/golden/` vacío (el golden no se toca).

## 3. REFACTOR

- [ ] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [ ] 3.2 Ningún `unwrap()`, `expect()` ni `panic!()` nuevo en el ayudante.
- [ ] 3.3 Determinismo comprobado con **dos `TMPDIR` distintos** (el del sistema y `/dev/shm`): `TMPDIR=/dev/shm cargo test --locked` en verde.
- [ ] 3.4 Ningún test toca `/data/notas`.
- [ ] 3.5 `just check-all` en verde.

## 4. Verificación y cierre

- [ ] 4.1 Los 363 tests previos pasan sin editar ningún caso de paridad ni el golden (el único fichero de tests tocado es el comparador compartido, `tests/common/golden.rs`).
- [ ] 4.2 `just spec-validate fix-golden-listing-order` en estricto.
- [ ] 4.3 Auditoría con el subagente `rust-reviewer` sobre el diff completo (solo lectura).
- [ ] 4.4 Commit `🐛 fix(test): …` y PR `hotfix/golden-listing-order` → `main`.
- [ ] 4.5 Merge a `main`; release patch **`v0.6.1`** (tag, `CHANGELOG.md`, `Cargo.toml`, `.vampus.yml`, crates.io, GitHub Release) y `development` sincronizado.
- [ ] 4.6 El CI de `main` (push de la release) queda **verde**.
- [ ] 4.7 `just spec-archive fix-golden-listing-order`.
