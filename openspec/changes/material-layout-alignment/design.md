# Design

## Context

El change `open-presentation` introdujo, para **un solo** comando, la resolución correcta:
`<scripts_root>/<render(episodes.scripts.dir)>`. El resto del binario se quedó con la fórmula vieja
(`<scripts_root>/<slug>`), que coincide mientras `dir` sea `{slug}`.

Medido antes de escribir este change, los puntos que cablean el slug son seis:

| Punto | Sitio |
|---|---|
| `where` | `src/commands/where_cmd.rs:22` |
| `get` | `src/commands/get.rs:42` |
| `status` (bloque «Siguiente a publicar») | `src/commands/status.rs:155` |
| JSON de `where` | `src/json.rs:45` |
| JSON de `get` | `src/json.rs:148` |
| `rebase-references` | `src/commands/rebase_references.rs:76` |
| plan de migración de material | `src/material.rs:218` (`collect_slug_renames`) |

Y el único que renderiza es `open` (`src/commands/open.rs:63-72`).

## Goals / Non-Goals

**Goals**

- Que la carpeta de material de un episodio tenga **una sola definición** en el binario.
- Que las superficies que **informan** de una ruta (`where`, `get`, `status`, `--json`) digan la
  verdad para cualquier perfil, no solo para `{slug}`.
- Que las **cirugías** de material escriban donde luego se lee.
- Cero cambio de salida para los perfiles actuales (`dir: "{slug}"` o sin `dir`).

**Non-Goals**

- Cambiar el esquema del perfil ni la función de render: `render_material_template` ya existe y no
  se toca.
- Leer `episodes.scripts.root` del perfil: la raíz sigue siendo el directorio padre del tracker
  (`scripts_root()`), como decidió `open-presentation`.
- Reapuntar agentes del pipeline ni tocar `/data/notas`.
- Renombrar el material ya migrado en la fase D.

## Decisions

### D1 — Una única fuente de verdad: `ResolvedShow::material_dir`

`material_dir(&self, slug: Option<&str>, number: Option<u32>) -> Result<PathBuf, PodtrackError>`
devuelve `scripts_root().join(render(dir_template, slug, number, &label))`, donde `dir_template` es
`scripts_dir` si no está vacío y `"{slug}"` en caso contrario. La etiqueta (`label`) se construye
dentro con `render_episode_label` (el `slug` si lo hay, si no el número); solo alimenta los mensajes
de error, **no** es un marcador de la plantilla.

**Por qué**: es el único modo de que el bug no vuelva por un séptimo punto de llamada.

### D2 — Fallback a `{slug}`

Igual que `open` (`DEFAULT_SCRIPTS_DIR`). Un perfil sin `dir` sigue funcionando y sigue dando la ruta
de siempre. El fallback se declara **una vez** y lo usan todos los subcomandos.

### D3 — El render no se reimplementa

Se reutiliza `render_material_template` / `render_marker` tal cual, incluido su comportamiento
documentado para marcadores desconocidos (ancho mayor de 32 o no parseable ⇒ se deja el marcador
literal).

### D4 — Se alinean también las cirugías, no solo la lectura

`collect_slug_renames` (el destino de `migrate-material`) escribe en `root.join(slug)`. Si `where`
informa de `episodio-07` y la migración crea `<slug>`, el binario se contradice a sí mismo. Se
alinean las dos mitades en el mismo change.

*Alternativa descartada*: dejar las cirugías como están y anotar la divergencia. Se descarta porque
deja el bug a medias y obliga a otro change para cerrarlo.

### D5 — Endurecimiento: el `dir` renderizado debe ser relativo

Si el `dir` renderizado es absoluto o contiene componentes `..`, se devuelve
`MaterialDirNotRelative` (código de configuración, `2`). `<scripts_root>` pasa a ser una frontera de
verdad y no una convención. Afecta igual a `open`, que hoy no lo comprueba.

### D6 — Errores tipados reutilizados

`EpisodeWithoutSlug` y `EpisodeWithoutNumber` ya existen —los introdujo `open-presentation`— y
describen exactamente los casos de render. No se inventan variantes nuevas para eso.

### D7 — Detalles de borde

- El **render corre antes** de la comprobación de relatividad: con un `dir` absoluto *y* un episodio
  sin número gana `EpisodeWithoutNumber` (exit 1), porque el marcador `{n}` es el que primero falla.
- `where --json` sin `slug` emite el **documento de error** (exit ≠ 0), no `material: null`: `where`
  nunca informa de una ruta que no puede construir; es intencional y lo fija `tracker-read`.
- `{episode}` **no** es un marcador de la plantilla de material (el render solo sustituye `{slug}`,
  `{n}` y `{n:0Nd}`); la etiqueta del episodio solo alimenta los mensajes de error.

## Risks / Trade-offs

- **Riesgo**: un perfil con `dir` que use `{n}` en un comando que no tenga el número a mano (por
  ejemplo `status` sobre un episodio sin número) pasará de informar una ruta a fallar con
  `EpisodeWithoutNumber`. *Mitigación*: es el comportamiento correcto —no se inventa una ruta— y el
  caso se cubre con test.
- **Trade-off**: el endurecimiento D5 cambia el comportamiento de `open` ante un perfil patológico.
  Se acepta: ningún perfil real lo usa y el error es explícito.
- **Riesgo de paridad**: nulo para `dir: "{slug}"` y para perfiles sin `dir`. Criterio de aceptación:
  los tests existentes pasan **sin tocarlos**.

## Migration Plan

1. Helper + tests unitarios del render y del rechazo (RED).
2. Reapuntar los seis puntos y añadir los escenarios de integración con `dir: "episodio-{n:02d}"`
   (RED → GREEN).
3. Comprobar que **toda** la suite previa sigue verde sin editar ningún test existente.
4. Auditoría con `rust-reviewer` y cierre de hallazgos.
5. Verificación con el binario real; `/data/notas` solo en lectura y sin escribir nada.

Rollback: revertir el commit. No hay datos que deshacer.
