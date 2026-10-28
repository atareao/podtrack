# Design

## Context

`podtrack` tiene siete comandos de lectura (`status`, `next`, `next-date`, `list`, `list-range`, `get`
y `where`) y ninguno responde a «¿qué episodios rodean a este?». La pregunta se resuelve hoy con
`list-range <n-4> <n+4>`, que filtra por rango numérico duro: con huecos en la numeración (un
`renumber`, un `insert`, números provisionales) devuelve menos filas de las previstas sin decir por
qué, no distingue cuál es el episodio de referencia y obliga a calcular el rango a mano.

El orden canónico del dominio es **por fecha ascendente, con el número como desempate** (SRS §4.3).
`neighborhood` no lo usa y eso es una decisión consciente, no un descuido: `list` y `status` ya
responden a «qué emite antes y después»; `neighborhood` responde a «qué números rodean a este», que
es la pregunta del operador cuando trabaja sobre la numeración de la serie.

## Goals / Non-Goals

**Goals**

- Un comando, una pregunta: la ventana de vecinos alrededor de un episodio, legible de un vistazo.
- Reutilizar la tabla de `list`/`list-range` sin introducir formato nuevo ni tocar sus bytes.
- Degradar bien en los extremos y con huecos: nunca fallar, nunca mentir sobre huecos.
- `--json` coherente con el resto de comandos de lectura.

**Non-Goals**

- No se introduce un orden nuevo de parrilla: el orden del dominio sigue siendo la fecha.
- No se cambia ningún comando existente ni su salida.
- No se muestra la ruta de material: una ventana de 9 filas con 9 rutas deja de leerse de un vistazo,
  y la ruta de un vecino se pide con `get`/`where`.
- No se añade un modo «ventana de parrilla» ni un `--sort`: si algún día se quiere, será otro change.

## Decisions

### D1 — La vecindad es numérica, no de parrilla

El usuario pidió literalmente `podtrack neighborhood 678`. La pregunta es del **espacio numérico**
(«los cuatro anteriores y los cuatro siguientes a este número»), no del calendario de emisión. Con
las fechas ya ordenan `list`, `list-range` y la ventana de `status`.

**Consecuencias asumidas (y de por qué se aceptan):**

- Un episodio **sin número** es invisible a la ventana: no tiene posición numérica. Es coherente con
  que el número provisional sea informativo hasta `publish`.
- El vecindario de un episodio con número **provisional** cambia cuando ese episodio se publica (su
  número se sobrescribe). No se «arregla»: es la consecuencia de preguntar por números.

**Alternativas descartadas:**

- *Vecindad por fecha:* responde a otra pregunta («qué emite antes y después de este») y ya la
  responde `list`/`status`; además, sin un `--around` temporal no hay forma de nombrar la ventana.
- *Vecindad numérica solo entre publicados:* deja fuera la cola de trabajo, que es justo lo que el
  operador mira al escribir el guion de un episodio futuro.

### D2 — Huecos: los N/M existentes más cercanos, no una ventana estricta

`neighborhood 678` con `--before 4` devuelve los **cuatro episodios con número inferior más altos**
que existan, aunque sean 672, 670, 669 y 668. Una ventana estricta (674-677) mostraría 2 filas y no
distinguiría «no hay cuatro» de «hay un hueco»; imprimir los huecos haría de la tabla una lista de
ausencias y rompería la reutilización de `show_episodes`.

### D3 — El episodio de referencia sale, en su sitio, marcado con `►`

Sin él, la ventana no dice dónde estás en la parrilla y hay que recordar el argumento. Con él, la
pregunta y la respuesta se leen en la misma pantalla. El marcador sustituye el **primer carácter de la
indentación** de la fila, de modo que la anchura de todas las columnas queda intacta y no hay
desplazamiento de tabla.

La referencia se reconoce por **su identidad** (su posición en el tracker), no por su `number`: con dos
episodios que compartan número —una anomalía de datos que el cargador no rechaza— marcar por número
pondría `►` en varias filas y, peor, los duplicados consumirían el presupuesto de `--before`/`--after`
mientras el reparto por número los descartaba de las dos listas, con lo que la tabla y el `--json`
mostrarían episodios distintos. Con identidad, la ventana contiene exactamente los vecinos de número
estrictamente menor, la referencia y los de número estrictamente mayor; los demás episodios con el mismo
número quedan fuera.

### D4 — El marcador no contamina a los demás comandos

`show_episodes` es compartida por `list`, `list-range` y `status`. El marcador es un parámetro
**opcional** (`None` por defecto) y, sin él, la función produce **bytes idénticos** a los de hoy.

**Guarda dura:** la suite actual (340 tests, incluidos los de paridad y los de `list`/`status`) SHALL
pasar sin editar un solo fichero de test. Si al añadir el parámetro hay que tocar un test existente,
el diseño está mal.

**Alternativa descartada:** un renderer propio en `neighborhood` (duplicaría cabecera, anchos de
columna, iconos, leyenda y el aviso de colisión de fechas: cuatro copias de un contrato).

### D5 — `--before`/`--after` con `4`/`4` por defecto y `0` válido

El `4` del ejemplo es el caso de uso, no una constante sagrada: dos `usize` con `clap` dan ventana
asimétrica (mirar solo hacia delante mientras se planifica la cola, por ejemplo) sin código extra.
`0` es un valor legítimo (ventana de un solo lado) y no necesita un flag distinto; los negativos los
rechaza `clap` (código de salida 2).

**Alternativas descartadas:** fijo 4 (menos superficie, pero la asimetría es gratis); `--window N`
(no permite asimetría).

### D6 — Degradación silenciosa en los extremos

Principio o final de la parrilla → se muestran los vecinos que haya, sin error y sin aviso. Es donde
más se consulta (el episodio recién añadido, el que acaba de publicarse) y un error ahí sería el peor
de los comportamientos. La cabecera declara los recuentos **reales** (`antes: 1, después: 4`), así
que el operador sabe qué está viendo sin que el comando tenga que disculparse.

### D7 — Sin ruta de material en la salida

`neighborhood` es una vista de parrilla. La ruta del material de un vecino se obtiene con `get` o
`where`, que ya existen y ya la calculan con la regla única. No se duplica (ni en texto ni en JSON) y
se mantiene el formato exacto de `list`.

### D8 — Error propio para un episodio sin número (no un fallback)

`neighborhood <slug>` sobre un episodio sin número falla con un error explícito. **No** se reutiliza
`EpisodeWithoutNumber` (su mensaje habla de la **ruta de material**, otro dato y otro comando) y
**no** se cae a una vecindad por fecha como plan B: responder con un criterio distinto y el mismo
formato es la peor clase de silencio.

### D9 — `--json` aditivo: `episode`, `before`, `after`

Cada lista va en orden **ascendente de número**, igual que la tabla, de modo que el **último**
elemento de `before` es el vecino inmediatamente inferior y el **primero** de `after` el
inmediatamente superior: la misma lectura que en texto, sin invertir nada según el formato. Los
campos de cada episodio son los compartidos (`number`, `slug`, `season`, `date`, `title`, `status`),
sin `material` (D7). `schema_version` NO SHALL cambiar: es un comando nuevo, no una forma alterada.

## Divergencia respecto al oráculo (registro)

No aplica: `neighborhood` es un **comando nuevo**, sin contrapartida en el oráculo Python retirado
(no existía `neighborhood` en `podcast-tracker.py`). La única divergencia relevante no es con el
oráculo sino con el **orden del dominio** (SRS §4.3), y se registra aquí:

| Aspecto | Orden del dominio (`list`, `status`) | `neighborhood` |
|---|---|---|
| Criterio | fecha ascendente, número como desempate | número, y solo número |
| Episodios sin número | aparecen, al final | invisibles (no tienen posición numérica) |
| Número provisional | informativo | es la clave de la ventana (reordenable hasta `publish`) |

## Risks / Trade-offs

| Riesgo | Mitigación |
|---|---|
| Que la vecindad numérica se lea como un orden de parrilla alternativo | El `Purpose` y el requisito lo dicen explícitamente; escenario propio con fechas invertidas para fijarlo |
| Que el marcador cambie bytes de `list`/`list-range`/`status` | Parámetro opcional + guarda dura: los 340 tests actuales pasan sin editarlos |
| Que la ventana quede muda con un tracker de un solo episodio | Se muestra la tabla con la única fila (la referencia); nunca se inventan vecinos |

## Migración / compatibilidad

Comando **nuevo**: nada existente cambia de forma, de salida ni de fichero. `docs/DEFINITION.md` §6
gana una fila en la tabla de subcomandos y `README.md` un ejemplo de uso. No hay ruptura para ningún
consumidor: los agentes del pipeline siguen usando los mismos comandos y `--json` sigue en
`schema_version: 1`.
