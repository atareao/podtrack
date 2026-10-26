# Git Flow — PodTrack

Este proyecto sigue **Git Flow** con versionado semántico. Igual que el resto de proyectos del entorno, se usa **siempre**: ninguna feature se commitea directamente en `main` ni en `development`.

**Estado actual:** remoto en `github.com/atareao/podtrack` (público), ramas `main` y `development`, pipeline de GitHub Actions en verde y release automatizada operativa: los secretos `GH_PAT` y `CARGO_REGISTRY_TOKEN` están creados y el crate se publica en crates.io en cada release (última: `v0.5.1`).

## Ramas

| Rama | Propósito | Base |
|---|---|---|
| `main` | Producción. Cada merge aquí dispara una release. | — |
| `development` | Integración de features en curso. Rama activa de trabajo. | `main` |
| `feature/<kebab-case>` | Nuevas funcionalidades. | `development` |
| `hotfix/<kebab-case>` | Correcciones urgentes a producción. | `main` |

## Flujo diario

### Features

```bash
git checkout development && git pull --ff-only
git checkout -b feature/mi-feature
# ... trabajo ...
git commit -m "✨ feat: add json output"
git push -u origin feature/mi-feature
gh pr create --base development --head feature/mi-feature
```

### Hotfixes

```bash
git checkout main && git pull --ff-only
git checkout -b hotfix/arreglo-critico
git commit -m "🚑️ hotfix: crash on empty tracker"
git push -u origin hotfix/arreglo-critico
gh pr create --base main --head hotfix/arreglo-critico
# tras el merge, sincronizar la integración:
git checkout development && git merge main && git push origin development
```

### Releases

Están automatizadas: **un merge a `main` es una release**. No se etiqueta a mano.

1. PR `development` → `main`.
2. Al mergear, `release-prepare.yml` (trigger: push a `main`):
   - compara `.vampus.yml:current_version` con el tag `vX.Y.Z` y decide entre `skip` (ya está publicada y HEAD es ese tag), `bump` (hay commits nuevos) o `tag` (la versión ya se subió a mano en el PR, solo falta etiquetar);
   - deduce el bump de los conventional commits: `✨ feat` → minor, `!:` o `BREAKING CHANGE` → major, el resto → patch;
   - verifica que vampus realmente subió la versión, resincroniza `Cargo.lock`, genera `CHANGELOG.md` con git-cliff y commitea `chore: release vX.Y.Z`;
   - empuja el tag `vX.Y.Z` y sincroniza `development` con `main`.
3. El tag dispara `release.yml`:
   - **crate** a crates.io;
   - **GitHub Release** con las notas del changelog.

### Secretos de release (ya configurados)

| Secreto | Para qué |
|---|---|
| `GH_PAT` | PAT con `contents: write`. Lo necesita `release-prepare` en el checkout: un tag empujado con el `GITHUB_TOKEN` por defecto **no** dispara `release.yml`. |
| `CARGO_REGISTRY_TOKEN` | Token de API de crates.io para el job `publish` de `release.yml`. |

Ambos secretos existen ya en el repositorio: no hay nada que configurar antes de una release.

### Publicación del crate

El crate se publica en crates.io en cada release. Hasta `v0.4.x`, `Cargo.toml` llevaba
`publish = false` a propósito —para que la primera release no dejara un `0.1.0` provisional
publicado para siempre (crates.io **no permite borrar** una versión, solo yank)— y el job
`publish` de `release.yml` leía esa línea para saltarse solo. La guarda se retiró al preparar los
metadatos de publicación y, desde entonces, cada tag `vX.Y.Z` publica la versión.

## Conventional Commits con gitmoji

El formato del mensaje determina el bump automático:

| Mensaje | Bump |
|---|---|
| `✨ feat: ...` | minor (0.Y.0) |
| `🐛 fix: ...` | patch (0.0.Z) |
| `💥 feat!: ...` o `BREAKING CHANGE` | major (X.0.0) |
| cualquier otro tipo (`refactor`, `docs`, `chore`, `test`…) | patch (0.0.Z) |

| Tipo | Emoji | Tipo | Emoji |
|---|---|---|---|
| `feat` | ✨ | `style` | 💄 |
| `fix` | 🐛 | `test` | ✅ |
| `hotfix` | 🚑️ | `chore` | 🔧 |
| `docs` | 📝 | `ci` | 👷 |
| `refactor` | ♻️ | `revert` | ⏪️ |
| `perf` | ⚡ | `deps` | ⬆️ |
| `breaking` | 💥 | `release` | 🔖 |

## Formateo previo obligatorio

Antes de crear ramas, commitear o abrir PRs, formatea para que el CI no falle. Aquí el layout es **solo `src/`** (un binario, sin panel):

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

En corto: `just check` encadena los tres en el orden exacto del CI; `just check-all` añade el rustdoc. Para aplicar formato en vez de comprobarlo: `just fmt-fix` y `just justfile-fmt`.

Si el formateo produce cambios, commitéalos aparte antes de cualquier otro cambio:

```bash
git commit -m "🎨 style: apply formatting"
```

## CI

| Workflow | Cuándo | Qué |
|---|---|---|
| `ci.yml` | PR y push a `development`/`main` | `fmt`, `clippy` y `test` |
| `release-prepare.yml` | push a `main` | versión, changelog y tag |
| `release.yml` | tag `v*` | crate a crates.io y GitHub Release |

## Planes

- Un plan por trabajo en `plans/PLAN-NNN.md`; al terminar `plans/PLAN-NNN-COMPLETED.md`, si se aparca `plans/PLAN-NNN-PENDING.md`.
- Automatizado en el `.justfile`: `just plans`, `just plan-new`, `just plan-done N` y `just plan-pending N`.

## Resumen visual

```text
main        ──●───────────────────────●────────────
               \                      /
development ────●──●──●──●──●──●──────●
                 \    /   \  /
feature           ●──●     ●─●
```
