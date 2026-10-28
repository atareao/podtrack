# Proposal

## Why

El trabajo sobre un episodio es casi siempre trabajo **alrededor** de un episodio: al escribir el
guion de uno se mira qué se publicó justo antes y qué toca después, al publicar se quiere leer la
serie de corrido y al revisar la numeración se quieren ver los vecinos y no el hueco.

Hoy eso se hace con `list-range <n-4> <n+4>` o leyendo a ojo la tabla de `list`, y ninguno de los dos
sirve. `list-range` filtra por rango numérico pero devuelve menos filas de las esperadas cuando la
parrilla tiene huecos —sin decir si el hueco es una ausencia real o un episodio que se movió—, no
marca cuál de los episodios es la referencia y obliga a calcular el rango a mano en cada invocación.

`neighborhood` responde a la pregunta «¿qué rodea al episodio 678?» con un solo comando.

## What Changes

1. Comando nuevo `podtrack neighborhood <target>`, con `target` por **número o slug** (la misma
   resolución que `get`, `where` y `open`).
2. Flags `--before N` y `--after M`, ambos con `4` por defecto y `0` válido (ventana de un solo lado).
3. La ventana SHALL ser **numérica**: hasta N episodios con el número inmediatamente inferior y hasta
   M con el inmediatamente superior. Con huecos, los N/M **episodios existentes más cercanos**; nunca
   filas vacías ni números inventados.
4. El episodio de referencia SHALL aparecer en su sitio, entre los anteriores y los siguientes,
   marcado con `►`; las filas SHALL ir en orden **ascendente de número**.
5. Si hay menos vecinos que los pedidos (principio o final de la parrilla), SHALL mostrarse los que
   haya: NO SHALL ser un error.
6. La salida SHALL ser la tabla de `list`/`list-range`, con la marca provisional `*` y su leyenda. La
   columna del marcador `►` SHALL existir **solo** en `neighborhood`: las filas de `list`,
   `list-range` y `status` SHALL permanecer inalteradas byte a byte.
7. `--json` SHALL ser aditivo: `episode` (el de referencia) más `before` y `after`, en orden
   ascendente de número; `schema_version` SHALL seguir siendo `1`.
8. Errores: un `target` inexistente SHALL ser el mismo error que `get`
   (`ERROR: Episodio '<target>' no encontrado.`); un `target` que resuelva a un episodio **sin
   número** SHALL fallar con un error explícito, porque una vecindad numérica necesita un número.
9. `neighborhood` NO SHALL escribir en el tracker, ni crear material, ni crear el fichero `.bak`, y
   NO SHALL mostrar la ruta de material de ningún episodio (para eso están `get` y `where`).
10. **Divergencia deliberada respecto del orden del dominio:** la vecindad es **numérica**, no el
    orden de parrilla por fecha. El SRS §4.3 fija que el número no ordena la parrilla; `neighborhood`
    no introduce un orden nuevo, responde a otra pregunta («qué números rodean a este»). Las dos
    consecuencias asumidas —un episodio sin número es invisible a la ventana, y el vecindario de un
    número **provisional** cambia cuando ese episodio se publica— SHALL quedar documentadas en
    `design.md`.

## Capabilities

### New Capabilities

Ninguna. `neighborhood` es un comando de **solo lectura** y engrosa la capacidad existente
`tracker-read`, que es la que define la superficie de lectura del binario.

### Modified Capabilities

- `tracker-read`: el `Purpose` se amplía para enumerar `neighborhood` entre los comandos de lectura,
  y se **añade** el requisito «`neighborhood` muestra la vecindad numérica de un episodio».
- `json-output`: el requisito «El documento identifica el show y su carga útil» se amplía para fijar
  la forma del documento de `neighborhood`.

## Impact

- `src/commands/neighborhood.rs` (**nuevo**): el comando, su selector puro de la ventana y su render.
- `src/commands/mod.rs`: `show_episodes` gana un marcador **opcional** del episodio de referencia que,
  sin marcador, produce bytes idénticos a los de hoy; y se registra el módulo nuevo.
- `src/main.rs`: el subcomando, sus dos flags y el despacho.
- `src/error.rs`: variante nueva para el episodio sin número (código de salida distinto de cero).
- `src/json.rs`: `neighborhood_document`.
- `tests/neighborhood.rs` (**nuevo**, un fichero por comando, como `open_presentation.rs`) y casos
  nuevos en `tests/json_output.rs`.
- `openspec/specs/tracker-read/spec.md` y `openspec/specs/json-output/spec.md`: deltas.
- `docs/DEFINITION.md` §6 (fila del comando en la tabla de subcomandos) y `README.md` (ejemplo de uso).
- **Sin impacto** en el tracker, en el material, en los perfiles de show, en la escritura ni en
  ninguno de los comandos existentes.
- **Cero cambio de comportamiento en lo existente:** la suite actual SHALL pasar **sin editar un solo
  fichero de test**.
