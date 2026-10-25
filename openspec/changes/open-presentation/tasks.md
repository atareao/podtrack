# Tasks

## 0. Andamiaje

- [ ] 0.1 `src/profile.rs`: `ResolvedShow` expone las plantillas de material leídas del perfil
      (`scripts.dir` y `scripts.files.presentacion`), sin romper `PartialEq`/`Eq` (todo `String`).
- [ ] 0.2 `src/profile.rs`: helper de **render** de plantillas (`{slug}`, `{n}`, `{n:0Nd}`) con su
      error tipado si falta el valor.
- [ ] 0.3 `cargo check --locked` limpio.

## 1. RED (tests que fallan primero)

- [ ] 1.1 `tests/open_presentation.rs` (nuevo): fixture de perfil (`scripts.dir: "{slug}"`,
      `files.presentacion: "presentacion-{slug}.pdf"`) + tracker + material temporal con
      `presentacion-<slug>.typ`. Invoca `open <slug> --dry-run --json` y afirma la ruta resuelta y
      `launched:false`; y `open <num> --dry-run` resolviendo la **misma** ruta. Falla hoy (no existe
      el subcomando).
- [ ] 1.2 Perfil sin `files.presentacion` → `open` sale con código `2` y el mensaje lo dice. Falla hoy.
- [ ] 1.3 Perfil con presentación declarada pero `.typ` ausente → código `1` y mensaje con episodio y
      ruta; y **no** se lanza nada. Falla hoy.
- [ ] 1.4 Unitario en `src/profile.rs`: render de `dir: "episodio-{n:02d}"` → `episodio-07`; `{slug}`
      sin slug → `EpisodeWithoutSlug`; `{n}` sin número → `EpisodeWithoutNumber`. No compila hoy.
- [ ] 1.5 Unitario en `src/error.rs`: los cuatro errores nuevos tienen el `exit_code` esperado
      (`PresentationNotDeclared` → 2; los otros → 1) y sus mensajes. No compila hoy.
- [ ] 1.6 `cargo test --locked`: 1.1–1.3 FALLAN en runtime; 1.4–1.5 no compilan; el resto VERDE.

## 2. GREEN (implementación mínima)

- [ ] 2.1 `src/error.rs`: `PresentationNotDeclared { show }`, `PresentationNotFound { slug, path }`,
      `LauncherNotFound { launcher }`, `EpisodeWithoutNumber { label }`, con sus mensajes y su brazo
      de `exit_code()`.
- [ ] 2.2 `src/profile.rs`: lectura de `episodes.scripts.dir`/`files.presentacion` y el render de
      plantillas, con `scripts_root` como raíz.
- [ ] 2.3 `src/commands/open.rs`: resolución del episodio, composición de la ruta (extensión `.typ`),
      `--dry-run` y lanzamiento desacoplado (`process_group(0)`, stdio a `null`, sin `wait`).
- [ ] 2.4 `src/main.rs`: subcomando `open` con `<target>` y `--dry-run`, despachado por la vía normal
      (exige tracker).
- [ ] 2.5 `src/json.rs`: documento de `open`.
- [ ] 2.6 `cargo test --locked` y `cargo check --locked` en verde.

## 3. REFACTOR

- [ ] 3.1 `cargo fmt --all` y `cargo clippy --all-targets --locked -- -D warnings` limpios.
- [ ] 3.2 Rustdoc de las cuatro variantes nuevas, del helper de render y del comando.
- [ ] 3.3 Ningún test toca `/data/notas` (fixtures temporales); el lanzamiento real es manual.
- [ ] 3.4 `just check-all` en verde.

## 4. Auditoría y verificación end-to-end

- [ ] 4.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
- [ ] 4.2 Binario real: `open 843 --dry-run` imprime la ruta de `la-memoria-de-valet`.
- [ ] 4.3 Binario real (manual): `open 843` abre `presentacion-la-memoria-de-valet.typ` con typshow
      y `podtrack` vuelve de inmediato.
- [ ] 4.4 `open 843 --json --dry-run` emite el documento esperado.

## 5. Documentación y cierre

- [ ] 5.1 `docs/DEFINITION.md` §6: fila del subcomando `open` en la tabla de superficie CLI.
- [ ] 5.2 `just spec-archive open-presentation`.
- [ ] 5.3 `just plan-done 5`.

## Release

- Rama `feature/open-presentation` → PR a `development` → merge → PR `development → main`.
- Conventional commit `feat` (gitmoji) ⇒ **minor** automático del CI: `0.4.2 → 0.5.0`.
