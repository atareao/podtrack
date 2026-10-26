# Design

## Context

El port terminó con `retire-python`, pero su alcance excluyó los tests del binario. El arnés de
paridad quedó como estaba: si no encuentra oráculo, imprime el motivo y hace `return`. Como ni
`just check` ni el CI definen `PODTRACK_REQUIRE_ORACLE`, 27 tests —el 8 % de la suite— reportan
`passed` sin ejecutar una sola aserción. La matriz es buena; el lado «esperado» es el que falta.

Medido hoy, los casos que dependen del oráculo son:

| Fichero | Casos dependientes | Tests del fichero |
|---|---|---|
| `tests/parity_oracle.rs` | 2 | 2 |
| `tests/parity_write_oracle.rs` | 9 | 9 |
| `tests/parity_writer.rs` | 2 | 3 |
| `tests/parity_surgery.rs` | 6 | 35 |
| `tests/srt_stage.rs` | 7 | 34 |
| `tests/rebase_references.rs` | 1 | 11 |
| **Total** | **27** | **94** |

### El oráculo es recuperable

Los tres `.py` viven en el histórico de `~/.agents`, en `c70287f^` (`c70287f` es el commit que los
borró):

| Fichero | Bytes | Líneas |
|---|---|---|
| `podcast-tracker.py` | 65 617 | 1 813 |
| `srt-stage.py` | 23 983 | 589 |
| `podcast-tracker.test.py` | 62 166 | 1 817 |

Recuperados a `/tmp/opencode/oracle-recovery/` y verificados: el módulo carga y expone los 17
`cmd_*` (`cmd_add`, `cmd_backfill_slugs`, `cmd_get`, `cmd_insert`, `cmd_list`, `cmd_list_range`,
`cmd_migrate_material`, `cmd_move`, `cmd_next`, `cmd_next_date`, `cmd_publish`,
`cmd_rebase_references`, `cmd_rename_material`, `cmd_renumber`, `cmd_status`, `cmd_update`,
`cmd_where`). Entorno: Python 3.14.8 + PyYAML 6.0.3.

Esto es lo que hace viable el plan: el golden puede grabarlo el oráculo de verdad, una última vez,
en vez de una foto del binario comparándose consigo mismo. Entre «bytes idénticos al oráculo de
referencia» y «bytes idénticos a lo que el binario hizo hoy» está la diferencia entre paridad y
tautología.

### La documentación narra el mundo antes del port

| Sitio | Dice | Realidad |
|---|---|---|
| `AGENTS.md:7` | el script Python «hoy vive» en la skill | no vive |
| `AGENTS.md:21-22` | tabla del oráculo con los dos `.py` | borrados |
| `AGENTS.md:27` | «Regla de oro durante el port» | el port terminó |
| `AGENTS.md:29` | el andaime «se derriba al terminar» | derribado |
| `AGENTS.md:43-53` | cinco fases; «el Python sigue siendo el que corre» | fases archivadas; agentes en `podtrack` |
| `README.md:19` | «cuando el binario lo sustituya…, el script se retira» | hecho |
| `GIT_FLOW.md:5` | última release `v0.5.0` | `v0.5.1` |
| `src/lib.rs:3`, `src/error.rs:3,164`, `src/srt_stage.rs:3` | port en presente | terminado |

## Goals / Non-Goals

**Goals**

- Que los 27 tests vuelvan a comprobar algo, contra un golden de **origen oráculo**, no contra sí
  mismos.
- Que la suite no invoque Python ni dependa de variables de entorno: `cargo test` a secas ejecuta
  todo.
- Que la documentación declare el estado real y no quede ninguna afirmación desfasada.
- Cero cambio de comportamiento del binario.

**Non-Goals**

- Ampliar la cobertura: `status` sigue fuera del golden, como fijó `status-upcoming-window`.
- Cambiar el formato del tracker ni del material: el golden reproduce bytes, no los define.
- Volver a la paridad viva: el oráculo no vuelve; el golden es evidencia congelada.
- Tocar `/data/notas`: el golden se graba sobre fixtures sintéticos.
- Arreglar la deuda viva de `PLAN-008` §7 (envoltorio `collect_slug_renames`, `Fixture::show()`,
  `ErrorKind` de `open`, `dir: "."`, deriva de títulos): queda fuera de alcance.

## Decisions

### D1 — El golden lo graba el oráculo recuperado, no el binario

La grabación se hace con los tres `.py` de `c70287f^` sobre la misma matriz de casos. Grabar desde
el binario sería una tautología: compararía el binario consigo mismo y no habría detectado ni uno
de los bugs que el port cazó.

### D2 — Grabación de un solo uso, sin generador en el repo

El código de grabación vive solo durante el change y se retira antes de commitear. Coherente con la
retirada de la convención muerta `FEATURE-NNN` (`PLAN-007` §7): el repositorio no se queda con una
dependencia de Python recién eliminada. Consecuencia asumida: **el golden no es regenerable**.

### D3 — Los 27 se reescriben en su sitio, con los mismos nombres

La matriz de casos es el activo. No se mueven ficheros ni se renombran tests: cambia solo el lado
«esperado», que pasa del oráculo al golden. Así el `git diff` de cada caso muestra qué se conserva.

### D4 — Un JSON por caso, con la forma del golden

`tests/golden/<área>/<caso>.json`, un fichero por caso, con:

```json
{
  "code": 0,
  "stdout": ["línea 1", "línea 2"],
  "stderr": [],
  "tracker": ["…"],
  "material": { "episodio-07/guion.md": ["…"] }
}
```

- `code` y `stdout`/`stderr` siempre; `tracker` y `material` solo donde hoy se comparan; en el
  writer canónico, `yaml` con los bytes del YAML.
- **Campos de texto como arrays de líneas**, no cadenas: `serde_json` escapa `\n`, así que una
  cadena deja el caso en una sola línea y `git diff` se vuelve ilegible. Reconstruir con
  `join("\n")` es sin pérdida, incluido el salto final.
- Claves ordenadas: el `Map` de `serde_json` está respaldado por un `BTreeMap`, así que dos
  grabaciones del mismo caso producen el mismo fichero.

### D5 — Guardián de no-Python

Un test de integración falla si reaparece bajo `tests/` cualquiera de las referencias prohibidas. Se
articula en dos familias:

- **Familia A — referencias inequívocas** (`importlib` y las cuatro variables `PODTRACK_*`): se
  buscan en el texto de **todo** fichero bajo `tests/`, con cualquier nombre y extensión, recorriendo
  recursivamente.
- **Familia B — el intérprete**: la aguja es el **nombre sin versión** (cubre `python`, `python3`,
  `python3.12`, `pythonw`) y se busca en todo fichero bajo `tests/` **menos `tests/fixtures/**`** y
  `tests/golden/**`.

**Por qué la familia B excluye `tests/fixtures/**`** (razonado, no implícito): bajo ese árbol solo hay
**datos** —12 `.srt` y 6 `.yaml`, transcripciones y perfiles—, hoy sin un solo ejecutable y sin una
sola coincidencia (`grep -rli python tests/fixtures/` → 0). La exclusión es **previsión declarada**:
una transcripción futura puede citar el intérprete en minúsculas en prosa, y eso no debe romper el CI.
El residuo —un script plantado a mano bajo `fixtures/`— queda como **límite conocido** del guardián,
no como descuido. La asimetría con la familia A es deliberada: esta **sí** escanea `fixtures/`, porque
las referencias inequívocas del oráculo (`importlib`, `PODTRACK_ORACLE*`) no pueden aparecer en una
transcripción; la palabra del intérprete, en cambio, sí.

Los **errores de lectura son infracciones**: un fichero o un directorio ilegible **falla** el
guardián, no lo deja pasar en vacío. Se comprueba también que ningún **nombre** —de fichero **o de
directorio**— **termine en `.py`** (sin distinguir mayúsculas): el **nombre completo**, no solo la
extensión. Las agujas se construyen por **fragmentos** para que el guardián no se delate a sí mismo.

Porqué (auditoría de 2026-10-26): la versión de solo `*.rs` dejaba fuera la reintroducción más
plausible (un script en `tests/`) y, sobre todo, **pasaba en silencio ante un árbol ilegible**, que es
el patrón que este change existe para eliminar. Es el criterio de cierre mecánico de
`python-retirement`, ahora automatizado.

### D6 — `status` sigue fuera del golden

`status-upcoming-window` lo dejó fuera por diseño. Este change **congela** la cobertura, no la
amplía: añadir casos aquí sería otro change con su propia justificación.

### D7 — La documentación se arregla declarando hechos verificables

Nada de prosa nueva que envejezca. Cada sitio pasa a decir lo que hoy es cierto y comprobable:
cinco fases archivadas en `openspec/changes/archive/`, agentes invocando `podtrack`, release
`v0.5.1`, port terminado.

### D8 — `/data/notas` no se escribe

El golden se graba sobre fixtures sintéticos. Los datos reales se usan solo como comprobación de
no-regresión, en lectura.

### D9 — Nada de dependencias nuevas

`serde_json` ya es dependencia de producción y `tempfile` de desarrollo. El lector del golden son
unas decenas de líneas en `tests/common/mod.rs`.

## Detalles de borde

- **Lectura en tiempo de ejecución**, no `include_str!`: un golden ausente SHALL ser un fallo de
  test con el nombre del fichero, no un error de compilación. Es lo que hace visible el RED y lo
  que permite añadir casos sin recompilar.
- **Un caso agrupa varios escenarios** cuando el test hoy itera sobre fixtures: el golden guarda el
  agregado tal como se compara; no se parte en un fichero por aserción.
- **El guardián suma un test**: el guardián es un test de integración **nuevo**, así que la suite
  pasa de 339 a 340 sin que se haya borrado ni un test. El criterio de cierre se lee como «los
  339 previos, ninguno saltado, más el guardián».
- **Prosa permitida**: la cabecera de los seis ficheros dice «oráculo recuperado» y cita `c70287f^`
  sin nombrar el intérprete. «El oráculo Python» en un comentario no es una invocación y no debe
  romper el guardián; por eso las agujas son literales ejecutables (`python3`), no la palabra
  «Python».
- **Rutas temporales normalizadas en el golden**: el `TempDir` de la fixture es distinto en cada
  ejecución, así que congelar su ruta haría el golden irreproducible. `stdout` y `stderr` se
  normalizan sustituyendo la raíz de la fixture por el marcador `<FIXTURE>`; `tracker`, `material`
  y `yaml` no la contienen (verificado). Solo se normaliza el prefijo correcto: una ruta distinta
  sigue haciendo fallar el caso.
- **Cabecera de los seis ficheros**: procedencia (grabado con el oráculo de `c70287f^`), aviso de
  que ya no hay paridad viva y de que el golden no se regenera solo.

## Risks / Trade-offs

- **El golden no es regenerable** (D2). Si cambia el formato, se regenera a mano o se recupera otra
  vez el oráculo del histórico. *Mitigación*: procedencia en cabeceras y en este `design.md`; el
  golden es evidencia histórica, no una herramienta.
- **Congela el comportamiento del oráculo, no el nuestro**: una mejora deliberada del binario hará
  fallar el caso y habrá que actualizar el golden a mano. Es el precio de no tener oráculo vivo. Se
  acepta: la paridad ya está demostrada (14/14 contra `v0.5.1`) y el golden queda como candado
  contra regresiones.
- **JSON verboso**: 27 ficheros con salidas completas. Se acepta por la legibilidad del `git diff`;
  la alternativa (cadenas esperadas inline en Rust) habría perdido la trazabilidad del cambio.
- **Falso positivo del guardián**: una nota histórica que cite `python3` rompería el test.
  *Mitigación*: la cabecera evita el token; si hiciera falta citarlo, se construye en tiempo de
  ejecución.
- **Riesgo de paridad**: nulo. No se toca ni una línea de lógica de `src/` (solo doc-comments) ni
  ninguna salida del binario. Criterio de aceptación: los 312 tests que ya comprobaban algo siguen
  verdes **sin editarlos**, y los 27 empiezan a comprobar contra el mismo golden que produjo el
  oráculo.
- **Divergencia medida del oráculo en `get` (RED, 2026-10-25)**: el escenario `get 100` de
  `parity_oracle/paridad_oraculo_fixture_mini` no coincide con el oráculo recuperado: el binario
  imprime `Slug:` y `Ruta:` desde el change archivado `get-shows-slug-and-path` (commit `c7a15e9`),
  que se mergeó con la red de paridad **ya inerte**. Es la materialización del riesgo «congela el
  comportamiento del oráculo, no el nuestro»: la lista completa se obtiene con un modo auditoría
  temporal y su tratamiento —excluir el escenario como se excluyó `status`, o congelarlo marcándolo
  como `source: binary`— se decide antes de cerrar la fase 3. Ver `plans/PLAN-008-COMPLETED.md` §9.

## Alternatives considered (descartadas)

- *Dejar los 27 saltando y documentarlo*: deja el 8 % de la suite como decorado y el CI en verde
  sin red. Descartada.
- *Grabar el golden desde el binario*: tautología. Descartada por D1.
- *Guardar el generador en el repo*: contradice la retirada de `FEATURE-NNN` y reintroduce Python.
  Descartada por D2.
- *Añadir una crate de snapshots (`insta`, `goldenfile`)*: dependencia nueva para 27 ficheros que
  se leen con `serde_json` en unas decenas de líneas. Descartada por D9.
- *Ampliar la cobertura con `status`*: fuera de alcance por D6.

## Migration Plan

0. **Prerrequisito** (temporal, fuera del repo): recuperar los tres `.py` de `c70287f^` a
   `/tmp/opencode/oracle-recovery/`, verificar bytes/líneas, Python + PyYAML y los 17 `cmd_*`;
   medir la baseline (`just check-all`, `cargo test` → 339/0, `PODTRACK_REQUIRE_ORACLE=1 cargo test
   --no-fail-fast` → 312/27).
1. **RED**: definir el lector y la forma del golden; reescribir los 27 en su sitio; comprobar que
   fallan **por golden ausente**.
2. **Grabación** (one-off): serializar la salida del oráculo sobre la misma matriz; grabar
   `tests/golden/`; revisar la cobertura; **retirar el generador**; verificar que el golden no
   salió del binario.
3. **GREEN**: `cargo test` sin ninguna variable `PODTRACK_*` y sin saltos.
4. **REFACTOR**: retirar el arnés en vivo de `tests/common/mod.rs` y las variables; guardián de
   no-Python; cabeceras de los seis ficheros.
5. **F — documentación**: `AGENTS.md`, `README.md`, `GIT_FLOW.md` y los cuatro doc-comments.
6. **Verificación de cierre**: la lista de `tasks.md` §6, incluida la paridad 14/14 con datos
   reales.

**Rollback**: revertir el commit. Se borran `tests/golden/` y el guardián; los 27 vuelven a saltar.
No hay datos que deshacer ni nada que restaurar fuera del repositorio.
