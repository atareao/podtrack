# PLAN-010 — Feature `shell-completions` (autocompletado)

> Testigo de la sesión del **2026-10-29**. Abre el trabajo pedido como «nueva feature: el
> autocompletado»: completado de shell para `podtrack`, **estático** (subcomandos y flags, que ya
> sabe `clap`) y **dinámico** (números y slugs de los episodios del show, y perfiles de `--show`).
>
> Alcance acordado con el usuario en la fase de planificación: **estáticas + dinámicas**, para
> **bash, zsh y fish**.
>
> Convención del repo: un plan por trabajo en `plans/PLAN-NNN.md`; al cerrarlo se **renombra**
> (nunca se borra) a `-COMPLETED.md` o `-PENDING.md`.

## 0. Estado de partida

| Dato | Valor |
|---|---|
| Rama | `development` @ `cee2e78` (= `origin/development`), árbol limpio |
| `main` | `6514ad9` (`chore: release v0.6.1`) |
| Divergencia | `development` va **4 commits por delante** de `main`, **solo de documentación** (los changes archivados `fix-golden-listing-order` y `episode-neighborhood`, y el cierre de `PLAN-009`) |
| Suite | `cargo test --locked` → **369 passed / 0 failed** |
| Changes activos | ninguno (`openspec list` → «No active changes found») |
| Specs | `openspec validate --specs --strict` → **13 passed / 0 failed** |
| Dependencias | `clap 4.6.7` (derive); **sin** completado de ningún tipo |
| Shells del entorno | `/usr/bin/bash`, `/usr/bin/zsh`, `/usr/bin/fish` (los tres, instalados) |

## 1. Objetivo

Que escribir un episodio deje de ser teclear a mano un slug largo, y que los subcomandos y flags
salgan solos al pulsar `<TAB>`.

1. Un subcomando **`podtrack completions <shell>`** (`bash`, `zsh`, `fish`) que imprime el script de
   registro. Es la puerta de entrada, descubrible por `--help`.
2. **Completado estático**: subcomandos, flags y los valores ya enumerados por `clap`
   (`--status`, `--expect`).
3. **Completado dinámico**:
   - el `target` de `get`, `where`, `open`, `neighborhood`, `update`, `publish` y `move`, con el
     número y el slug de cada episodio del show resuelto;
   - el valor de `--show`, con los slugs de perfil de la cascada del resolver.
4. Cero impacto en el contrato: sin `COMPLETE` en el entorno, el binario se comporta byte a byte
   como hasta ahora y los 369 tests previos siguen verdes.

## 2. Mapa arquitectónico

### 2.1 Mecanismo

`clap_complete` con el feature **`unstable-dynamic`** (el «motor» de completado de `clap`) y, con él,
`clap/unstable-ext`. El feature arrastra tres dependencias nuevas (`is_executable`, `shlex` y
`clap_lex`) que **no** estaban en la caché local: `Cargo.lock` pasa a fijarlas y el job del CI las
descarga (el índice y `static.crates.io` responden `200`).

**No hay dos scripts.** El script que emite `completions` no lleva candidatos incrustados: llama al
binario (`COMPLETE=<shell> podtrack -- <palabras…>`) y **el binario** resuelve el completado
ejecutando el mismo motor de `clap`. Por eso el **estático** (subcomandos, flags y
`value_parser = [...]`) y el **dinámico** (nuestros completadores) salen del mismo sitio: la
definición del `Command`.

### 2.2 Puntos de cambio

| Fichero | Qué |
|---|---|
| `Cargo.toml` / `Cargo.lock` | `clap_complete = { version = "4.6", features = ["unstable-dynamic"] }` |
| `src/main.rs` | `CompleteEnv::with_factory(Cli::command).complete()` como **primera** línea de `main`; variante `Completions { shell }`; atributos `add = ArgValueCompleter::new(...)` en los `target` y en `--show`; despacho **antes** de `resolver.resolve` (como `srt-stage`) |
| `src/completions.rs` **(nuevo)** | candidatos puros (`targets`) + adaptadores al motor (`target_completer` y `show_completer`, este sobre `ShowResolver::available_slugs()`) + `SHELLS` + `write_registration` |
| `src/commands/completions.rs` **(nuevo)** | `CompletionsParams`, `enum Shell` (`ValueEnum`) y `run` → `Rendered::stdout(script)` |
| `src/commands/mod.rs` | `pub mod completions;` |
| `src/lib.rs` | `pub mod completions;` |
| `src/profile.rs` | método público `available_slugs()` (extraído de `available()`, mismo resultado) |
| `tests/common/mod.rs` | `Fixture::cmd()` pasa a **retirar** `COMPLETE` del entorno (guarda dura) |
| `tests/completions.rs` **(nuevo)** | el contrato de la feature |
| `docs/DEFINITION.md` §6 y `README.md` | la fila y el ejemplo de instalación del completado |

### 2.3 Contrato del script

- Determinista: `bin` y `completer` SHALL ser la cadena literal `podtrack` — **nunca** una ruta del
  sistema — de modo que dos invocaciones produzcan bytes idénticos y el script sea congelable.
- La decisión se aplica a los **dos** caminos de emisión: `CompleteEnv` se construye con
  `.bin("podtrack").completer("podtrack")`, así que `podtrack completions bash` y
  `COMPLETE=bash podtrack --` emiten el **mismo** script. Consecuencia documentada: el script asume
  que `podtrack` está en el `PATH`.

### 2.4 Contrato de los candidatos dinámicos

- `target`: para cada episodio del tracker **en orden de fichero**, el número (si lo tiene) y el
  slug (si lo tiene); filtrado por prefijo; sin duplicados.
- `--show`: slugs de `*.yaml` de `candidate_dirs()` de la cascada, deduplicados (gana el primero) y
  ordenados.
- **Nunca falla**: sin show, sin tracker o con YAML ilegible → lista vacía, exit `0`, sin `stderr`.
- Limitación asumida: el completador **no ve** el `--show` de la misma línea (la API del motor solo
  recibe la palabra actual), así que los valores se resuelven con la cascada del **entorno**
  (`PODTRACK_SHOWS_DIR` / `PODTRACK_SHOW` / directorio del ejecutable / `~/.config`).

## 3. Decisiones

- **D1 — Motor de `clap_complete`, no scripts a mano.** La alternativa (tres scripts propios + un
  subcomando oculto `__complete`) da control total y cero dependencias nuevas, pero obliga a
  reproducir y mantener el protocolo de bash (`COMP_WORDS`/`COMP_CWORD`), zsh y fish. Se usa el
  motor de `clap_complete`, que es upstream y ya viaja con `clap`.
- **D2 — El subcomando es la puerta de entrada.** `COMPLETE=bash podtrack` ya funciona por sí solo,
  pero es indescubrible; `completions <shell>` lo hace explícito, validable por `clap` y congelable
  en un test.
- **D3 — El script nombra el binario, no su ruta.** Ver §2.3. El precio es exigir `podtrack` en el
  `PATH`; el beneficio es un contrato determinista y testeable byte a byte.
- **D4 — `--json` no aplica a `completions`.** La salida es un script, no un documento; el flag
  global se ignora (documentado y con test).
- **D5 — La guarda `COMPLETE` vive en la fixture.** `Fixture::cmd()` retira `COMPLETE` del entorno,
  para que un `COMPLETE` exportado en la máquina de quien corre los tests no pueda convertir la
  suite de paridad en un generador de scripts.
- **D6 — Los fallos del completador son silenciosos.** Un completado que escribe en `stderr` o
  falla rompe el prompt del shell; por eso el completador degrada a lista vacía.
- **D7 — Nada de pánicos.** El shell se modela con un `enum` + `ValueEnum`, así que el mapeo es
  exhaustivo y no hay `unreachable!()`.

## 4. Fases (SDD → TDD → auditoría → PR → release)

El change vive en `openspec/changes/shell-completions/` (proposal, design, delta de la capacidad
nueva `shell-completions`, tasks). **No se escribe código ni tests sin su aprobación.**

### 4.1 RED

- 1.1 `tests/completions.rs`: los escenarios del delta antes de que exista la implementación.
- 1.2 Candidatos puros: `targets` (prefijo, orden, duplicados) y el filtrado por prefijo de los perfiles de `--show`.
- 1.3 Extremo a extremo con el protocolo: `COMPLETE=bash` + `_CLAP_COMPLETE_INDEX` + `podtrack -- …`.
- 1.4 `podtrack completions bash|zsh|fish`: script no vacío, determinista, sin rutas.
- 1.5 Shell no soportado → exit `2`.
- 1.6 `bash -c 'source <(podtrack completions bash); complete -p podtrack'` → exit `0`.
- 1.7 RED medido: los tests de la feature fallan y los **369 previos siguen verdes**.

### 4.2 GREEN

- 2.1 `Cargo.toml` + `Cargo.lock`.
- 2.2 `src/completions.rs` y `src/commands/completions.rs`.
- 2.3 `src/main.rs`: `CompleteEnv`, la variante nueva, los `add` y el despacho temprano.
- 2.4 `src/profile.rs`: `available_slugs()`.
- 2.5 `tests/common/mod.rs`: la guarda `COMPLETE`.
- 2.6 Suite en verde (369 + los nuevos).

### 4.3 REFACTOR

- 3.1 `cargo fmt`, `clippy -D warnings`, `rustdoc -D warnings`.
- 3.2 Sin `unwrap`/`expect`/`panic!` en el camino de completado.
- 3.3 Determinismo del script: dos ejecuciones, bytes idénticos.
- 3.4 Ningún test toca `/data/notas`; completar en el árbol real no escribe nada.

### 4.4 Verificación y cierre

- 4.1 Auditoría `rust-reviewer` sobre el diff completo.
- 4.2 `docs/DEFINITION.md` §6 con la fila de `completions` y `README.md` con el ejemplo.
- 4.3 Commit `✨ feat(completions): …`, PR de feature → `development`.
- 4.4 Archivado del change (`just spec-archive shell-completions`).
- 4.5 Release: PR `development` → `main` (minor, `0.7.0`) y publicación.
- 4.6 Testigo cerrado (`just plan-done 10`).

## 5. Criterios de cierre

| # | Criterio | Cómo se comprueba |
|---|---|---|
| 1 | Los escenarios del delta están cubiertos por tests | `tests/completions.rs` + unitarios de `src/completions.rs` |
| 2 | Los 369 tests previos pasan sin editar ningún test de paridad | `cargo test --locked` |
| 3 | `just check-all` verde | fmt + clippy + test + rustdoc |
| 4 | El script es determinista y sin rutas | dos ejecuciones, `cmp` de bytes |
| 5 | `bash`/`zsh`/`fish` registran el completado; `bash` además se prueba end-to-end | `complete -p podtrack` en `bash` |
| 6 | Cero cambios de salida en el binario sin `COMPLETE` | suite de paridad + `git diff tests/golden/` vacío |
| 7 | Auditoría sin hallazgos bloqueantes | informe de `rust-reviewer` |

## 6. Riesgos

- **Dependencia con feature inestable (`unstable-dynamic`).** Mitigación: es el camino oficial de
  `clap_complete` para completado dinámico; se fija con `--locked` y el contrato propio (tests) no
  depende de los bytes del script, sino del protocolo y de los candidatos.
- **El script embebido deja de cuadrar con el binario tras un cambio.** Mitigación: el script se
  genera *sourceando en el arranque del shell* (`source <(podtrack completions bash)`), que es
  «autocorregible», como recomienda upstream.
- **`COMPLETE` exportado por accidente.** Mitigación: D5 (la suite se protege sola) y la
  documentación de `COMPLETE=` vacío para desactivar.
