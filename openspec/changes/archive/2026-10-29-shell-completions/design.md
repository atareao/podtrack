# Design

## Context

`podtrack` es un binario único con `clap` derive (`clap 4.6.7`) y 20 subcomandos. Siete de ellos
identifican el episodio con un `target` que acepta **número o slug** (`get`, `where`, `open`,
`neighborhood`, `update`, `publish`, `move`), y hay un flag **global** `--show <SLUG>` que resuelve
el perfil con una cascada de tres directorios (`PODTRACK_SHOWS_DIR` → directorio del ejecutable →
`~/.config/podtrack/shows`; `src/profile.rs`, `ShowResolver::candidate_dirs`).

No existe ningún completado: `grep -ri complet` sobre `src/`, `tests/` y `Cargo.toml` no devuelve
nada propio.

El contrato del binario es **estricto**: la salida de cada comando está congelada en
`tests/golden/` y la suite tiene 369 tests verdes. Todo lo que se añada tiene que ser **invisible**
cuando no se está completando.

## Goals / Non-Goals

**Goals**

- Un script de registro para `bash`, `zsh` y `fish`, **determinista** y congelable en un test.
- Completado estático de subcomandos, flags y los valores ya enumerados por `clap`.
- Completado dinámico del `target` (número y slug de cada episodio del show) y de `--show` (slugs
  de perfil de la cascada).
- Cero cambios observables en el binario cuando no hay `COMPLETE` en el entorno.

**Non-Goals**

- No se completa `add` (su posición es `[num] título`, una mezcla de número y texto libre) ni los
  subcomandos sin `target`.
- No se completan `--date`, `--slug`, `--season`, `--last`, `--from` ni `--delta`.
- No se generan scripts para `elvish` ni `powershell`.
- El binario no persiste ningún script en disco: si el usuario quiere un fichero, lo redirige él.

## Decisions

### D1 — Motor de `clap_complete` (`unstable-dynamic`), no scripts propios

El feature `unstable-dynamic` de `clap_complete` es el motor de completado de `clap`: aporta el
protocolo shell↔binario (bash con `COMP_WORDS`/`COMP_CWORD`, zsh, fish), la lista de candidatos a
partir de la definición del `Command` y la API para candidatos propios
(`ArgValueCompleter`/`CompletionCandidate`). Se habilita, además, `clap/unstable-ext`, que el
feature reclama.

El feature arrastra tres dependencias nuevas —`is_executable`, `shlex` y `clap_lex`— y la versión
queda fijada en `Cargo.lock`; el CI las descarga (índice y `static.crates.io` responden `200`).

**Alternativas descartadas:**

- *Scripts escritos a mano + un subcomando oculto `__complete`*: control total y cero dependencias
  nuevas, pero obliga a reproducir y mantener el protocolo de los tres shells (índices de palabra,
  variables `COMP_*`, escapeo, `nospace`, `nosort`). El motor ya lo hace y es upstream.
- *Solo generación AOT (`clap_complete::aot::generate`), sin `unstable-dynamic`*: produce un script
  con los candidatos **congelados** en el momento de generarlo; no puede completar slugs y números
  leídos del tracker. Descartado por el alcance elegido (dinámicas).
- *Embeber la ruta absoluta del ejecutable en el script* (`current_exe()`, que es lo que hace
  `clap_complete` por defecto): robusto ante `PATH`, pero el script deja de ser determinista y no
  se puede congelar en un test. Ver D4.

### D2 — Un solo script sirve lo estático y lo dinámico

El script no lleva candidatos incrustados: registra una función que llama al binario
(`COMPLETE=<shell> podtrack -- <palabras…>`) y **el binario** resuelve el completado ejecutando el
mismo motor de `clap`. En esa llamada, el **estático** (nombres de subcomando, flags, y los valores
de un `value_parser = [...]`) y el **dinámico** (nuestros completadores) salen del mismo sitio: la
definición del `Command`. No hay, por tanto, dos artefactos que puedan desincronizarse entre sí.

Corolario de diseño: el motor **no filtra** por prefijo los candidatos de un completador propio, así
que el filtrado por la palabra ya escrita es responsabilidad nuestra.

`CompleteEnv` se construye además con `.shells(SHELLS)`, la misma lista de tres shells (`bash`, `zsh`,
`fish`) que gobierna el subcomando: así el protocolo de entorno no atiende un shell que `completions`
rechaza, y los dos caminos coinciden también en qué shells soportan.

### D3 — `podtrack completions <shell>` como puerta de entrada

`COMPLETE=<shell> podtrack` ya funciona por sí solo, pero no se descubre. `completions <shell>`
—validado por `clap` contra los tres valores— lo hace explícito en `--help`, imprimible a `stdout` y
congelable en un test. Se despacha **antes** de resolver el show, como `srt-stage`, para que funcione
fuera de todo show y sin tracker.

### D4 — El script nombra el binario, no su ruta

`write_registration` recibe `bin = "podtrack"` y `completer = "podtrack"`, no la ruta del ejecutable.
El script queda **determinista** (dos invocaciones producen bytes idénticos) y **libre de rutas del
sistema**, que es lo que permite congelarlo byte a byte en un test. El precio es documentado: el
script asume que `podtrack` está en el `PATH`.

La decisión se aplica a **los dos caminos** de emisión: `CompleteEnv` se construye con
`.bin("podtrack").completer("podtrack")`, así que `podtrack completions bash` y
`COMPLETE=bash podtrack --` producen el **mismo** script, y los dos son deterministas. Sin ese
`override`, el camino de entorno embebería la ruta con la que se invocó al binario y los dos scripts
divergirían entre sí y entre máquinas.

### D5 — Qué se completa, exactamente

- **`target`** de `get`, `where`, `open`, `neighborhood`, `update`, `publish` y `move`: para cada
  episodio del tracker **en orden de fichero**, el **número** (si lo tiene) y el **slug** (si lo
  tiene) —los dos identificadores que esos comandos aceptan—, filtrados por el prefijo escrito y sin
  duplicados. Dentro de cada episodio, el número precede al slug.
- **`--show`**: los slugs de perfil (`*.yaml`) de los directorios de la cascada, sin duplicados
  (gana el primero de la cascada) y ordenados alfabéticamente. La lista sale de `ShowResolver::available_slugs()`, la misma fuente que el mensaje de perfiles disponibles, para no duplicar el recorrido de la cascada ni su deduplicado.
- **Estático**: lo que ya aporta `clap` (subcomandos, `--long`, y los `value_parser` de `--status`
  y `--expect`).

### D6 — Resolución y política de fallo del completador

El completador del `target` resuelve el show con la cascada del **entorno** y carga el tracker. Como
la API del motor solo le pasa la palabra en curso, **no ve** un `--show` escrito en la misma línea;
se documenta como limitación asumida.

Política de fallo: si el show no resuelve, el tracker no existe o el YAML no carga, el completador
devuelve **lista vacía** y el binario sale con `0` sin escribir en `stderr`. Un completado que
falla o que habla rompe el prompt del shell; degradar en silencio es el único comportamiento
aceptable aquí.

### D7 — `--json` no aplica a `completions`

La salida de `completions` es un script de shell, no un documento. El flag global `--json` se
**ignora** y así se documenta y se prueba; `completions` no participa del contrato de
`json-output`.

### D8 — La guarda `COMPLETE` vive en la fixture

`Fixture::cmd()` pasa a ejecutar `env_remove("COMPLETE")`. Si quien corre los tests tiene `COMPLETE`
exportado en su shell, sin esa guarda la suite de paridad se convertiría en un generador de scripts
y los 369 tests fallarían de forma desconcertante. Ningún test de la feature depende de que el
entorno esté «limpio»: los que necesitan el protocolo lo fijan ellos mismos.

### D9 — Nada de pánicos en el camino de completado

El shell se modela con un `enum Shell` + `ValueEnum`: `clap` valida el valor y el mapeo al
adaptador de shell es exhaustivo, así que no hay `unreachable!()` ni `panic!()`.

## Risks / Trade-offs

- **El feature es `unstable-dynamic`.** Mitigación: es el camino oficial de `clap_complete` para
  completado dinámico, se fija con `--locked`, y el contrato propio se prueba por **protocolo** y
  por **candidatos**, no por los bytes del script (que son de upstream).
- **Desincronización script↔binario tras un cambio.** Mitigación: el script se genera *sourceando
  en el arranque del shell* (`source <(podtrack completions bash)`), que es autocorregible, como
  recomienda upstream.
- **`COMPLETE` exportado por accidente en la máquina.** Mitigación: D8 para la suite, y `COMPLETE=`
  (vacío) o `COMPLETE=0` para desactivar el completado en el shell.
- **Trade-off de D4:** el script depende del `PATH`. Se acepta a cambio del determinismo y de que los
  dos caminos de emisión coincidan; queda documentado en el `README.md` que se sourcee desde
  el arranque del shell.
