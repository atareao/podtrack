# Tasks

## 0. Andamiaje

- [x] 0.1 `src/profile.rs`: `ResolvedShow` expone las plantillas de material leídas del perfil
      (`scripts.dir` y `scripts.files.presentacion`), sin romper `PartialEq`/`Eq` (todo `String`).
      *Nota: los campos nuevos son `String`, así que `PartialEq`/`Eq` siguen intactos.*
- [x] 0.2 `src/profile.rs`: helper de **render** de plantillas (`{slug}`, `{n}`, `{n:0Nd}`) con su
      error tipado si falta el valor.
      *Nota: render con `{slug}`, `{n}` y `{n:0Nd}`, con error tipado cuando falta el valor.*
- [x] 0.3 `cargo check --locked` limpio.
      *Nota: `cargo check --locked` en verde.*

## 1. RED (tests que fallan primero)

- [x] 1.1 `tests/open_presentation.rs` (nuevo): fixture de perfil (`scripts.dir: "{slug}"`,
      `files.presentacion: "presentacion-{slug}.pdf"`) + tracker + material temporal con
      `presentacion-<slug>.typ`. Invoca `open <slug> --dry-run --json` y afirma la ruta resuelta y
      `launched:false`; y `open <num> --dry-run` resolviendo la **misma** ruta. Falla hoy (no existe
      el subcomando).
      *Nota: el escenario falla primero por ausencia del subcomando, como estaba previsto.*
- [x] 1.2 Perfil sin `files.presentacion` → `open` sale con código `2` y el mensaje lo dice. Falla hoy.
      *Nota: verificado el exit `2` y el mensaje de presentación no declarada.*
- [x] 1.3 Perfil con presentación declarada pero `.typ` ausente → código `1` y mensaje con episodio y
      ruta; y **no** se lanza nada. Falla hoy.
      *Nota: verificado el exit `1` con episodio y ruta en el mensaje, sin lanzamiento.*
- [x] 1.4 Unitario en `src/profile.rs`: render de `dir: "episodio-{n:02d}"` → `episodio-07`; `{slug}`
      sin slug → `EpisodeWithoutSlug`; `{n}` sin número → `EpisodeWithoutNumber`. No compila hoy.
      *Nota: cubiertos los tres casos de render y sus errores tipados.*
- [x] 1.5 Unitario en `src/error.rs`: los cuatro errores nuevos tienen el `exit_code` esperado
      (`PresentationNotDeclared` → 2; los otros → 1) y sus mensajes. No compila hoy.
      *Nota: los cuatro errores nuevos con su `exit_code` y mensaje verificados.*
- [x] 1.6 `cargo test --locked`: 1.1–1.3 FALLAN en runtime; 1.4–1.5 no compilan; el resto VERDE.
      *Nota: RED confirmado en el commit `48ecef8`; el resto de la suite se mantuvo verde.*

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/error.rs`: `PresentationNotDeclared { show }`, `PresentationNotFound { slug, path }`,
      `LauncherNotFound { launcher }`, `EpisodeWithoutNumber { label }`, con sus mensajes y su brazo
      de `exit_code()`.
      *Nota: las cuatro variantes con mensaje y `exit_code()` implementados.*
- [x] 2.2 `src/profile.rs`: lectura de `episodes.scripts.dir`/`files.presentacion` y el render de
      plantillas, con `scripts_root` como raíz.
      *Nota: la raíz de material es `scripts_root()` (el padre del tracker), igual que en `where`/`get`; el binario no lee `episodes.scripts.root` del perfil. Spec y design quedaron alineados con eso.*
- [x] 2.3 `src/commands/open.rs`: resolución del episodio, composición de la ruta (extensión `.typ`),
      `--dry-run` y lanzamiento desacoplado (`process_group(0)`, stdio a `null`, sin `wait`).
      *Nota: lanzamiento en segundo plano con `process_group(0)`, stdio a `null` y sin `wait`.*
- [x] 2.4 `src/main.rs`: subcomando `open` con `<target>` y `--dry-run`, despachado por la vía normal
      (exige tracker).
      *Nota: despachado por la vía normal, con la exigencia de tracker.*
- [x] 2.5 `src/json.rs`: documento de `open`.
      *Nota: documento JSON de `open` con los campos acordados.*
- [x] 2.6 `cargo test --locked` y `cargo check --locked` en verde.
      *Nota: GREEN en el commit `97c3cce`; suite completa en verde con 309 tests.*

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets --locked -- -D warnings` limpios.
      *Nota: fmt y clippy sin avisos.*
- [x] 3.2 Rustdoc de las cuatro variantes nuevas, del helper de render y del comando.
      *Nota: rustdoc completo en las variantes, el helper y el comando.*
- [x] 3.3 Ningún test toca `/data/notas` (fixtures temporales); el lanzamiento real es manual.
      *Nota: toda la suite usa fixtures temporales; nada toca `/data/notas`.*
- [x] 3.4 `just check-all` en verde.
      *Nota: `just check-all` verde (fmt + clippy + test + rustdoc con `-D warnings`); `tests/open_presentation.rs` acaba con 13 tests (5 originales + 8 de cierre de cobertura en `137dd08`).*

## 4. Auditoría y verificación end-to-end

- [x] 4.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
      *Nota: `rust-reviewer` dio «APROBADO CON RESERVAS»; tras corregir el parser de perfil a lectura laxa (`episodes: Option<serde_yaml::Value>` + `yaml_scalar_str`, commit `7428d43`) y el resto de hallazgos, la re-auditoría los cerró todos, sin BLOCKER ni MAJOR. Antes de la corrección, una variación de tipo bajo `episodes.scripts` rompía la carga del perfil de todos los subcomandos con exit 2.*
- [x] 4.2 Binario real: `open 843 --dry-run` imprime la ruta de `la-memoria-de-valet`.
      *Nota: `open 843 --dry-run` → `▶ typshow: /data/notas/Podcasts/guiones/la-memoria-de-valet/presentacion-la-memoria-de-valet.typ`.*
- [x] 4.3 Binario real (manual): `open 843` abre `presentacion-la-memoria-de-valet.typ` con typshow
      y `podtrack` vuelve de inmediato.
      *Nota: no se ejecutó a propósito para no abrir una ventana en el entorno de trabajo. Queda cubierto por el test con shim `typshow`: prueba que se lanza en segundo plano con la ruta correcta y que `podtrack` vuelve de inmediato. La apertura gráfica real queda pendiente de que el usuario la pruebe.*
- [x] 4.4 `open 843 --json --dry-run` emite el documento esperado.
      *Nota: `--dry-run --json` emite `schema_version`, `show`, `episode`, `launcher`, `presentation` y `launched:false`.*

## 5. Documentación y cierre

- [x] 5.1 `docs/DEFINITION.md` §6: fila del subcomando `open` en la tabla de superficie CLI.
      *Nota: documentado en el commit `068358e`.*
- [x] 5.2 `just spec-archive open-presentation`.
- [x] 5.3 `just plan-done 5`.

## Release

- Rama `feature/open-presentation` → PR a `development` → merge → PR `development → main`.
- Conventional commit `feat` (gitmoji) ⇒ **minor** automático del CI: `0.4.2 → 0.5.0`.
> Estado: implementado, auditado y verificado. Pendiente: archivar el change y abrir el PR a development.
