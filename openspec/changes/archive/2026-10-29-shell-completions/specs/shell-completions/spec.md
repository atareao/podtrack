# Spec Delta

## Purpose

Definir el **autocompletado de shell** de `podtrack`: el subcomando que emite el script de registro,
el completado **estático** (subcomandos y flags, de `clap`) y el completado **dinámico** (los
números y slugs de los episodios del show y los perfiles de `--show`), sin que nada de eso altere el
comportamiento del binario cuando no se está completando.

## ADDED Requirements

### Requirement: `podtrack completions <shell>` emite el script de registro

`podtrack completions <shell>` SHALL aceptar `bash`, `zsh` y `fish` como únicos valores de `<shell>`;
cualquier otro valor SHALL ser un error de uso de `clap` (exit code `2`) y NO SHALL producir script
alguno.

SHALL escribir en `stdout` el **script de registro** de `clap_complete` para ese shell: el que, al
ser evaluado por el shell, registra el completado de `podtrack` y hace que el shell llame al binario
en tiempo de completado. El script SHALL estar **determinista** —dos invocaciones del mismo shell
SHALL producir bytes idénticos— y NO SHALL contener rutas del sistema de ficheros: SHALL referirse
al binario por su nombre, `podtrack`.

El subcomando SHALL NO depender de un show resuelto ni de un tracker existente: SHALL funcionar
fuera de cualquier show, SHALL NO escribir ningún fichero y SHALL NO tocar `/data/notas`.

#### Scenario: los tres shells producen un script

- **GIVEN** un directorio de trabajo sin ningún show
- **WHEN** se ejecuta `podtrack completions bash` (y `zsh`, y `fish`)
- **THEN** SHALL salir con código `0` y escribir en `stdout` un script no vacío
- **AND** NO SHALL escribir nada en `stderr`

#### Scenario: el script es determinista y sin rutas absolutas

- **GIVEN** `podtrack completions bash`
- **WHEN** se ejecuta dos veces desde directorios de trabajo distintos
- **THEN** los bytes de `stdout` SHALL ser idénticos
- **AND** el script SHALL nombrar el binario como `podtrack`, NO mediante una ruta del sistema

#### Scenario: el script se evalúa y registra el completado

- **GIVEN** el script de `podtrack completions bash`
- **WHEN** se evalúa en `bash` y se consulta `complete -p podtrack`
- **THEN** la consulta SHALL salir con código `0` e informar de un manejador de completado
  registrado para `podtrack`

#### Scenario: un shell no soportado es un error de uso

- **GIVEN** `podtrack completions tcsh`
- **WHEN** se ejecuta
- **THEN** SHALL salir con código `2`
- **AND** NO SHALL escribir un script en `stdout`

### Requirement: el `target` se completa con los números y slugs del show

Cuando el shell completa el argumento `target` de `get`, `where`, `open`, `neighborhood`, `update`,
`publish` y `move`, el binario SHALL ofrecer, para cada episodio del show resuelto, su **número**
(si lo tiene) y su **slug** (si lo tiene) —los dos identificadores que esos comandos aceptan—
filtrados por el prefijo ya escrito.

El orden SHALL ser el del fichero del tracker; dentro de cada episodio el número SHALL preceder al
slug, y la lista NO SHALL contener duplicados.

El completado NO SHALL fallar por el estado del show: si el show no resuelve, el tracker no existe o
el YAML no carga, SHALL ofrecer una lista **vacía**, salir con `0` y NO SHALL escribir en `stderr`
nada que no sean candidatos.

#### Scenario: se ofrecen números y slugs

- **GIVEN** un show cuyo tracker tiene un episodio `200` con slug `primer-episodio`
- **WHEN** se completa el `target` de `get` con el prefijo vacío
- **THEN** la lista de candidatos SHALL incluir `200` y `primer-episodio`

#### Scenario: el prefijo filtra

- **GIVEN** ese mismo show
- **WHEN** se completa el `target` de `get` con el prefijo `primer`
- **THEN** la lista SHALL incluir `primer-episodio`
- **AND** NO SHALL incluir `200`

#### Scenario: sin show resoluble la lista queda vacía y en silencio

- **GIVEN** un entorno cuyo `PODTRACK_SHOWS_DIR` apunta a un directorio que no contiene el perfil
- **WHEN** se completa el `target` de `get`
- **THEN** el binario SHALL salir con `0`
- **AND** NO SHALL emitir candidatos ni mensajes

### Requirement: el flag `--show` se completa con los perfiles disponibles

Cuando el shell completa el valor de `--show`, el binario SHALL ofrecer los **slugs de perfil**
(`*.yaml`) presentes en los directorios de la cascada del resolver, **sin duplicados** (gana el
primero de la cascada) y ordenados alfabéticamente, filtrados por el prefijo ya escrito.

#### Scenario: se ofrecen los perfiles de la cascada

- **GIVEN** un `PODTRACK_SHOWS_DIR` con `demo.yaml` y `otro.yaml`
- **WHEN** se completa el valor de `--show`
- **THEN** la lista SHALL incluir `demo` y `otro`

#### Scenario: los duplicados se colapsan

- **GIVEN** el mismo slug de perfil presente en dos directorios de la cascada
- **WHEN** se completa el valor de `--show`
- **THEN** SHALL aparecer una sola vez

### Requirement: el completado no altera el comportamiento del binario

La feature NO SHALL cambiar la salida de ningún comando: sin la variable `COMPLETE` en el entorno,
toda invocación SHALL comportarse byte a byte como antes. Cuando `COMPLETE` nombra un shell y el
binario recibe una petición de completado, SHALL resolverla, escribir los candidatos en `stdout` y
salir con `0`, **sin** ejecutar el comando pedido.

El camino de completado NO SHALL escribir en `stdout` antes de resolver el completado.

`COMPLETE` vacío o con el valor `0` SHALL desactivar el completado y dejar que el binario ejecute el
comando con normalidad.

#### Scenario: la suite de paridad sigue verde

- **GIVEN** el entorno sin `COMPLETE`
- **WHEN** se ejecuta la suite completa
- **THEN** los 369 tests previos SHALL pasar sin cambios
- **AND** ningún test SHALL depender de que `COMPLETE` esté o no exportado

#### Scenario: `COMPLETE` desactivable

- **GIVEN** `COMPLETE` vacío o con el valor `0`
- **WHEN** se ejecuta cualquier comando
- **THEN** el binario SHALL ejecutar el comando con normalidad
