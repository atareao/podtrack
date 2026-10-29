# Tasks

## 0. Prerrequisito

- [x] 0.1 La base es `development` @ `cee2e78`, con la suite en **369 passed / 0 failed** y el árbol limpio. Ningún test toca `/data/notas`. — verificado antes de escribir nada.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/completions.rs` (fichero nuevo, con `mod common;`): los escenarios del delta, escritos antes de la implementación.
- [x] 1.2 Candidatos puros: `targets` (orden de fichero, número antes que slug, filtro por prefijo, sin duplicados) y el filtrado por prefijo de los perfiles de `--show` sobre `ShowResolver::available_slugs()`.
- [x] 1.3 Extrema a extrema con el protocolo real: `COMPLETE=bash` + `_CLAP_COMPLETE_INDEX` + `podtrack -- …` para el `target` de `get` y para `--show`. — con las variables `_CLAP_*` limpias en el helper.
- [x] 1.4 `podtrack completions bash|zsh|fish`: script no vacío, sin `stderr`, determinista (dos ejecuciones, bytes idénticos) y sin rutas del sistema.
- [x] 1.5 `podtrack completions tcsh` → exit `2` y sin script. — endurecido: además SHALL nombrar el valor rechazado y listar los shells válidos, para que no pueda pasar sin la feature.
- [x] 1.6 `bash -c 'source <(podtrack completions bash); complete -p podtrack'` → exit `0` y manejador registrado.
- [x] 1.7 Sin show resoluble, el completado del `target` sale con `0` y sin candidatos ni mensajes.
- [x] 1.8 RED medido: los tests de la feature fallan y los **369 previos siguen verdes**. — `cargo test --locked --test completions` → **2 passed / 10 failed**, compilando; los 2 que pasan son guardas y los 10 fallos son por la feature ausente.

## 2. GREEN (implementación mínima)

- [x] 2.1 `Cargo.toml`: `clap_complete = { version = "4.6", features = ["unstable-dynamic"] }`; `Cargo.lock` actualizado y commiteado.
- [x] 2.2 `src/completions.rs`: `targets`, `target_completer`, `show_completer` (sobre `ShowResolver::available_slugs`) y `write_registration`, más `SHELLS` (la lista de tres shells que acota también el protocolo de entorno).
- [x] 2.3 `src/commands/completions.rs`: `CompletionsParams`, `enum Shell` (`ValueEnum`) y `run` → `Rendered::stdout`.
- [x] 2.4 `src/lib.rs` y `src/commands/mod.rs`: declarar los módulos.
- [x] 2.5 `src/profile.rs`: `available_slugs()` público (extraído de `available()`, mismo resultado).
- [x] 2.6 `src/main.rs`: `CompleteEnv::with_factory(Cli::command).bin(BIN).completer(BIN).shells(SHELLS).complete()` como primera línea de `main`; variante `Completions { shell }`; `add = ArgValueCompleter::new(...)` en los siete `target` y en `--show`; despacho temprano, antes de `resolver.resolve`.
- [x] 2.7 `tests/common/mod.rs`: `Fixture::cmd()` retira `COMPLETE` del entorno. — extendido a **todos** los helpers de arranque del binario de la suite, no solo a la fixture.
- [x] 2.8 `cargo test --locked` en verde (369 previos intactos + los nuevos). — **384 passed / 0 failed**.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.
- [x] 3.2 Ningún `unwrap()`, `expect()`, `panic!()` ni `unreachable!()` nuevo en el camino de completado.
- [x] 3.3 Determinismo del script comprobado con `cmp` de dos ejecuciones. — idénticos (1139 bytes) desde dos directorios distintos y con `TMPDIR=/dev/shm`.
- [x] 3.4 `git status --porcelain tests/golden/` vacío y ningún test toca `/data/notas`.
- [x] 3.5 `just check-all` en verde. — 384 passed / 0 failed.

## 4. Verificación y cierre

- [x] 4.1 Los 369 tests previos pasan sin editar ningún caso de paridad ni el golden.
- [x] 4.2 `just spec-validate shell-completions` en estricto. — `Change 'shell-completions' is valid`.
- [x] 4.3 Auditoría con el subagente `rust-reviewer` sobre el diff completo (solo lectura). — sin hallazgos bloqueantes; los menores (guarda `COMPLETE` en toda la suite, protocolo de entorno acotado a los tres shells, test de shell no soportado endurecido, `show_completer` sobre `available_slugs`, `VAR` privado y limpieza de `_CLAP_*` en los tests) reparados y re-verificados.
- [x] 4.4 `docs/DEFINITION.md` §6 con la fila de `completions` y `README.md` con el ejemplo de instalación del completado.
- [x] 4.5 Commit `✨ feat(completions): …` y PR de feature → `development`. — commit `2e3aa05`, PR #36 fusionado tras CI verde (fmt/clippy/test/doc) y suite en **385 passed / 0 failed**.
- [x] 4.6 `just spec-archive shell-completions`. — delta fundido en `openspec/specs/shell-completions/` en el PR de archivo.
- [x] 4.7 PR `development` → `main`; release **minor** (`0.7.0`) publicada. — PR #37; `release-prepare` subió a `0.7.0`, generó el CHANGELOG, etiquetó `v0.7.0` y sincronizó `development`; `release.yml` publicó el crate y creó la GitHub Release.
- [x] 4.8 `plans/PLAN-010.md` → `plans/PLAN-010-COMPLETED.md`. — `just plan-done 10` en el PR de archivo.
