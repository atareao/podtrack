# Design

## Context

`podtrack` ya resuelve **dónde vive el material** de un episodio, pero solo por el `slug`
(`scripts_root()/<slug>/`, la regla de `where`/`get`/`status`). El esquema del perfil, sin embargo,
declara ese layout de forma explícita y **parametrizada**:

- `atareao-con-linux` → `scripts.dir: "{slug}"`, `scripts.files.presentacion: "presentacion-{slug}.pdf"`.
- `la-era-de-las-distros` → `scripts.dir: "episodio-{n:02d}"`, `scripts.files: {}` (sin presentación).

La presentación de un episodio es un `.typ` cuya **fuente** se llama igual que su PDF entregable
(`files.presentacion`), y que se abre con **typshow** (`typshow <ruta.typ>`), un visor de doble
ventana que enraíza la compilación en el directorio del propio `.typ` (por eso funcionan sus
`#import "./template/slydst.typ"` e `images/`): **no hace falta tocar el *cwd*** del proceso.

Restricciones que dan forma al diseño:

- El Python **no** tiene `open`: no hay contrato de paridad que reproducir. Todo es extensión aditiva.
- Los tests **nunca** tocan `/data/notas` (SRS §11.5): la resolución se prueba con perfiles y
  material de fixtures temporales; el lanzamiento real es manual.
- Lanzar una GUI bloqueando al invocador es inaceptable para los agentes que usan el binario:
  `open` debe devolver el control de inmediato.

## Goals / Non-Goals

**Goals:**

- `podtrack open <num|slug>` que resuelva la presentación del episodio **desde el layout del
  perfil** y la abra con typshow sin bloquear.
- `--dry-run` para resolver y mostrar la ruta sin lanzar (testeable sin GUI).
- `--json` aditivo, coherente con el resto de comandos.
- Errores tipados que digan la verdad (no declarada / no existe / lanzador ausente / falta un dato
  del episodio) con códigos de salida coherentes.

**Non-Goals:**

- Abrir otras piezas del material (guion, escaleta, PDF…) o con `xdg-open`. **Solo** la presentación.
- Esperar a que se cierre typshow, ni heredar su código de salida.
- Override del binario por env/flag (`PODTRACK_TYPSHOW`, `--typshow`): el usuario eligió **solo PATH**.
- Alinear `where`/`get` con `scripts.dir` (hoy usan el `slug`): es una limitación **preexistente**
  que este change NO corrige (ver D8).
- Material legacy numerado (`guiones/0819/`) para episodios sin migrar: igual que `where`, queda
  fuera; lo arregla `migrate-material`.
- Windows/macOS: `process_group` es Unix.

## Decisions

### D1 — La ruta sale del layout del perfil, no de una regla cableada

La fuente de la presentación se compone así, en este orden:

1. **Carpeta:** `<scripts.root>/<scripts.dir>` con `scripts.dir` **renderizado**; si el perfil no
   declara `dir`, se asume `{slug}` (la convención de `episode-material-layout`).
2. **Fichero:** `episodes.scripts.files.presentacion` **renderizado**, con la extensión
   **reescrita a `.typ`** (`Path::with_extension("typ")`).

`scripts.root` es, como siempre, el directorio padre del `tracker` resuelto (`ResolvedShow::scripts_root`).
La clave `episodes.scripts.root` del perfil **no** se consulta: la raíz de material es siempre el
padre del tracker, por coherencia con `where`/`get` (que resuelven igual). Honrar esa clave en `open`
haría que dos comandos del mismo binario discreparan sobre dónde vive el material.

### D2 — Render de plantillas mínimo y explícito

Se sustituyen los marcadores que el esquema documenta:

- `{slug}` → el slug del episodio;
- `{n}` y `{n:0Nd}` (p. ej. `{n:02d}`) → el número del episodio, con relleno opcional a `N` dígitos.

Si la plantilla pide un marcador cuyo valor no existe en el episodio, es un error explícito:
`{slug}` sin slug → `EpisodeWithoutSlug`; `{n}` sin número → `EpisodeWithoutNumber`. Un marcador
**desconocido** (`{algo}`) se deja literal (no es responsabilidad de este comando definir el formato
completo del esquema). El render vive en `src/profile.rs` (junto a la lectura del perfil), no en el
comando.

### D3 — `typshow` se busca en el PATH, sin override

Se resuelve por `PATH` (el binario está en `~/.cargo/bin/typshow`). Sin variable de entorno ni flag.
Si no aparece, error tipado `LauncherNotFound` (exit 1) con un mensaje accionable.

### D4 — Lanzamiento desacoplado, no bloqueante

`std::process::Command::new("typshow").arg(<ruta>)`, con:

- `stdin`/`stdout`/`stderr` → `Stdio::null()` (los logs de typshow no contaminan la salida del
  binario, y `--json` sigue emitiendo un único documento);
- `process_group(0)` (`std::os::unix::process::CommandExt`) para que el visor sobreviva a señales
  dirigidas al grupo de procesos del invocador (los agentes suelen matar su grupo al terminar);
- `spawn()` **sin** `wait()`: `podtrack` sale con `0` de inmediato.

### D5 — `--dry-run` resuelve y muestra, sin lanzar

`open <target> --dry-run` imprime la ruta resuelta y sale con `0` sin invocar a typshow. Es la vía
determinista de testear la resolución, porque lanzar una GUI no es viable en CI.

### D6 — Contrato de salida y errores

**Texto (stdout):**

- normal: `▶ typshow: <ruta>\n`
- `--dry-run`: `▶ typshow: <ruta>\n` seguido de `  ⚠ DRY-RUN: no se lanza nada\n`

**JSON (`--json`), aditivo:**

```json
{ "schema_version": 1, "show": "…", "episode": { "number": 843, "slug": "…" },
  "presentation": "<ruta.typ>", "launcher": "typshow", "launched": true }
```

con `"launched": false` en `--dry-run`.

**Códigos de salida:**

| Caso | Error tipado | Exit |
|---|---|---|
| El perfil no declara `files.presentacion` | `PresentationNotDeclared { show }` | 2 |
| El `.typ` no existe en disco | `PresentationNotFound { slug, path }` | 1 |
| `typshow` no está en el PATH | `LauncherNotFound { launcher }` | 1 |
| El layout necesita `{slug}` y el episodio no tiene | `EpisodeWithoutSlug { label }` (existente) | 1 |
| El layout necesita `{n}` y el episodio no tiene número | `EpisodeWithoutNumber { label }` | 1 |
| Episodio objetivo inexistente | `EpisodeNotFound { target }` (existente) | 1 |

`PresentationNotDeclared` sale con `2` porque es un defecto del **perfil** (su esquema), no del disco.

### D7 — Extensión aditiva, no paridad

El oráculo Python no conoce `open`. Este change NO toca la salida, los códigos ni el YAML de ningún
subcomando portado; añade un subcomando nuevo. La suite de paridad sigue verde sin cambios.

### D8 — Material legacy numerado: fuera de alcance (y aviso)

Como `where`/`get`, `open` resuelve por el layout del perfil; para `atareao-con-linux` eso es
`<root>/<slug>/`. Los episodios con material **sin migrar** (`guiones/0819/…`) no se resuelven:
si su `.typ` no está bajo la carpeta esperada, será `PresentationNotFound`. Se alineará cuando se
aliene `where`; **no** es parte de este change.

## Risks / Trade-offs

- **Divergencia `open` vs `where` en `scripts.dir`:** `open` honra `dir` (p. ej.
  `episodio-{n:02d}`); `where` sigue cableando el `slug`. Hoy no hay show con presentación y `dir`
  no-`{slug}`, así que no se observa; queda anotado como deuda para un change futuro.
- **Zombis al no reaped:** `spawn` sin `wait` deja que el hijo sea reaped por init al salir
  `podtrack`. Con `process_group(0)` y stdio a `null` el riesgo práctico es nulo (el proceso es
  corto y no interactúa).
- **`process_group` solo en Unix:** el crate ya es Linux-only en la práctica; se documenta.

## Migration Plan

No hay migración de datos ni de perfiles: la ruta se deriva de lo que los perfiles ya declaran.
Los perfiles sin `files.presentacion` sencillamente no admiten `open` (error explícito).
