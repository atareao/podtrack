# Tasks

## 0. Prerrequisito: las dependencias

- [x] 0.1 Resolver, contra la caché local de `~/.cargo/registry`, las versiones exactas de `clap` (derive), `serde` (derive), `serde_json`, el backend YAML (`serde_yaml` o `serde_yml`), la librería de fechas (`time` o `chrono`) y `anyhow`/`thiserror`; en dev, `assert_cmd`, `predicates` y `tempfile`. Escribirlas en `Cargo.toml` sin fijarlas de memoria.
- [x] 0.2 `cargo check` limpio con las dependencias puestas.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 Portar a `tests/parity/` los escenarios de la suite de referencia correspondientes a **solo lectura** (`status`, `next`, `next-date`, `list`, `list-range`, `get`, `where`), sobre fixtures temporales.
- [x] 1.2 `show-profile`: test que fija la cascada de búsqueda (env → junto al ejecutable → config), el show por defecto (`PODTRACK_SHOW` y el compilado) y que un perfil ausente o inválido falla nombrando la causa.
- [x] 1.3 `tracker-store`: test que fija el modelo (número opcional, slug opcional, estado cerrado), el orden por fecha con número como desempate y `TBD` al final.
- [x] 1.4 `tracker-store`: test que fija el *writer* canónico —cabecera de comentarios conservada, orden de campos, `title` entrecomillado, `slug` solo si existe— y el backup `.bak` con la escritura atómica.
- [x] 1.5 `tracker-read`: test que fija el texto exacto de `status` (resumen + bloque «Siguiente a publicar» con slug y ruta) y su omisión cuando no hay pendientes.
- [x] 1.6 `tracker-read`: test que fija `next` (hueco + autoritativo), `next-date` y la marca `*` con leyenda en `list`/`list-range`, incluyendo que `--status published` no marca nada.
- [x] 1.7 `tracker-read`: test que fija `get` por número y por slug, `where` (imprime la ruta y **no** crea el directorio) y los *exit codes* de error con mensaje en `stderr`.
- [x] 1.8 `json-output`: test que fija que sin `--json` no hay JSON, que con `--json` hay un documento con `schema_version` y `show`, y que el error con `--json` es JSON en `stderr`.
- [x] 1.9 Ejecutar `cargo test`: las pruebas nuevas fallan por no existir el código.

## 2. GREEN (implementación mínima)

- [x] 2.1 `model.rs`: `Episode`, `Status`, `Slug` (newtype kebab-case validado) y `Date` con el orden del dominio.
- [x] 2.2 `profile.rs`: resolución del show y carga del perfil por la cascada, con errores tipados.
- [x] 2.3 `tracker.rs`: lectura por `serde` y **escritura canónica a mano**, con backup `.bak` y escritura atómica.
- [x] 2.4 `commands/`: `status`, `next`, `next-date`, `list`, `list-range`, `get`, `where`, reproduciendo el texto de referencia.
- [x] 2.5 `json.rs` + flag global `--json`: emisión versionada de las cargas de lectura y de los errores.
- [x] 2.6 `main.rs`: `clap` derive (subcomandos y banderas) y despacho.
- [x] 2.7 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en código de producción; errores tipados con `thiserror` y `anyhow` solo en el borde.
- [x] 3.3 El *writer* del tracker no usa `serde` para escribir: reproduce el formato canónico de forma explícita.
- [x] 3.4 Ningún test ni ejecución toca `/data/notas`: todo corre sobre fixtures temporales.

## 4. Verificación de paridad (operador)

- [x] 4.1 Para cada comando de lectura, comparar con `diff` la salida del binario y la del Python sobre el **mismo fixture** congelado; SHALL ser idéntica.
- [x] 4.2 Comprobar los *exit codes* de los casos de error contra los del Python.
- [x] 4.3 `podtrack status --json --show <fixture>` devuelve JSON válido (`jq .`) con `schema_version` y `show`.
- [x] 4.4 `just check-all` en verde.
