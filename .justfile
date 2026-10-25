# ═══════════════════════════════════════════════════════════════════════════
# PodTrack — recetas de desarrollo
# ═══════════════════════════════════════════════════════════════════════════
#
# `just --list` es el punto de entrada. Lo que hay que saber:
#
#   * Cada LÍNEA de una receta corre en su PROPIO shell: encadena con `&&` o usa
#     una receta con shebang (`#!/usr/bin/env bash`) si necesitas varias.
#   * `just check` es exactamente lo que corre el CI. Si pasa en local, pasa en
#     el PR, porque el workflow ejecuta los mismos comandos.
#   * Este proyecto es un binario único: no hay panel ni contenedor.
#   * Nada se escribe sin un change proposal aprobado en openspec/changes/.

name := "podtrack"

# ── Punto de entrada ────────────────────────────────────────────────────────

# Sin argumentos: lista las recetas disponibles
default:
    @just --list

# ── Calidad (paridad exacta con el CI) ─────────────────────────────────────

# Los tres checks del CI, en su orden. Ejecútalo antes de cada commit
check: fmt clippy test

# Los tres del CI más el rustdoc
check-all: check doc

# Formato: solo comprueba
fmt:
    cargo fmt --all -- --check

# Formato: aplica los cambios. Si toca algo, commit aparte (🎨 style: apply formatting)
fmt-fix:
    cargo fmt --all

# Clippy con cero warnings, con los mismos flags que el CI
clippy:
    cargo clippy --all-targets -- -D warnings

# Tests unitarios (#[cfg(test)]) y de integración (tests/)
test:
    cargo test --locked

# Rustdoc: falla con cualquier warning, enlaces rotos incluidos
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Comprueba el formato del propio .justfile
justfile-check:
    @just --fmt --check

# Aplica formato al .justfile
justfile-fmt:
    @just --fmt

# ── Desarrollo ─────────────────────────────────────────────────────────────

# Ejecuta el binario en local pasando los argumentos tal cual: `just dev status --show x`
dev *args:
    cargo run -- {{ args }}

# ── Gitflow (SIEMPRE: nada directo en main ni en development) ──────────────

# Crea una rama feature desde development
gf-feature feature_name:
    git checkout development
    git pull --ff-only
    git checkout -b feature/{{ feature_name }}
    @echo "✅ feature/{{ feature_name }} creada desde development"

# Crea una rama hotfix desde main
gf-hotfix desc:
    git checkout main
    git pull --ff-only
    git checkout -b hotfix/{{ desc }}
    @echo "✅ hotfix/{{ desc }} creada desde main"

# Trae development a la rama actual
gf-sync:
    git fetch origin
    git merge origin/development

# Empuja la rama actual y abre el PR (a development; a main si es hotfix/*)
gf-pr:
    #!/usr/bin/env bash
    set -euo pipefail
    branch="$(git rev-parse --abbrev-ref HEAD)"
    case "${branch}" in
        feature/*|chore/*) base="development" ;;
        hotfix/*)          base="main" ;;
        *) echo "❌ Estás en '${branch}'. gf-pr solo actúa en feature/*, chore/* o hotfix/*."; exit 1 ;;
    esac
    git push -u origin "${branch}"
    gh pr create --base "${base}" --head "${branch}" --fill

# Fusiona en su base el PR de la rama actual, borra la rama y deja development al día
gf-merge:
    #!/usr/bin/env bash
    set -euo pipefail
    branch="$(git rev-parse --abbrev-ref HEAD)"
    case "${branch}" in
        feature/*|chore/*) base="development" ;;
        hotfix/*)          base="main" ;;
        *) echo "❌ Estás en '${branch}'. Solo se fusionan PRs de feature/*, chore/* o hotfix/*."; exit 1 ;;
    esac
    # Fusión condicionada al CI: `gh pr checks` sale distinto de cero si algún
    # check falla o sigue pendiente.
    gh pr checks >/dev/null 2>&1 || { echo "❌ El PR no tiene todos los checks en verde:"; gh pr checks; exit 1; }
    gh pr merge --merge --delete-branch
    git checkout "${base}"
    git pull --ff-only
    # Un hotfix entra en main y hay que devolverlo a la integración.
    if [ "${base}" = "main" ]; then
        git checkout development
        git pull --ff-only
        git merge origin/main --no-edit
        git push origin development
    fi
    echo "✅ PR de ${branch} fusionado en ${base}."

# No etiquetes a mano: mergear este PR dispara la release automática (bump + tag + crate).
#
# Abre el PR de release development → main.
gf-release:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! gh secret list -R atareao/{{ name }} | grep -q '^GH_PAT'; then
        echo "❌ Falta el secreto GH_PAT en atareao/{{ name }}."
        echo "   Sin él, el tag que empuja release-prepare no dispara release.yml"
        echo "   (un tag empujado con el GITHUB_TOKEN por defecto no dispara workflows)."
        echo "   Créalo con: gh secret set GH_PAT -R atareao/{{ name }}"
        exit 1
    fi
    gh pr create --base main --head development \
        --title "release: $(vampus show)" \
        --body "Release automática: al mergear, release-prepare sube la versión, genera el CHANGELOG, etiqueta v$(vampus show) y release.yml publica el crate."

# Árbol de ramas
gf-graph:
    git log --oneline --graph --all --decorate=short -30

# ── Versión y release ──────────────────────────────────────────────────────

# Muestra la versión actual (la fuente es .vampus.yml)
version:
    @vampus show

# La versión vive en TRES sitios que deben ir sincronizados: Cargo.toml, la
# entrada [[package]] de Cargo.lock y .vampus.yml:current_version.
#
# Normalmente no hace falta: release-prepare lo hace solo en cada release.
#
# Sube la versión y resincroniza Cargo.lock.
bump type="patch":
    #!/usr/bin/env bash
    set -euo pipefail
    before="$(vampus show)"
    vampus upgrade --{{ type }}
    cargo update --workspace
    after="$(vampus show)"
    if [ "${before}" = "${after}" ]; then
        echo "❌ vampus no cambió la versión (revisa los 'replaces' de .vampus.yml)."
        exit 1
    fi
    echo "✅ ${before} → ${after}"
    echo "   Commitea los tres ficheros: Cargo.toml, Cargo.lock y .vampus.yml"

# Vista previa del CHANGELOG sin escribir nada
changelog:
    @git-cliff --unreleased --strip header

# ── Pipeline (GitHub Actions) ──────────────────────────────────────────────

# Últimos runs del CI
pipeline:
    @gh run list -R atareao/{{ name }} --limit 8

# Espera al run de la rama actual y falla si no acaba en verde
pipeline-watch:
    #!/usr/bin/env bash
    set -euo pipefail
    branch="$(git rev-parse --abbrev-ref HEAD)"
    id="$(gh run list -R atareao/{{ name }} --branch "${branch}" --limit 1 --json databaseId --jq '.[0].databaseId // empty')"
    if [ -z "${id}" ]; then
        echo "❌ No hay ningún run para la rama ${branch}."
        exit 1
    fi
    gh run watch "${id}" -R atareao/{{ name }} --exit-status

# Lista los secretos del repo (los releases necesitan GH_PAT y CARGO_REGISTRY_TOKEN)
secrets:
    @gh secret list -R atareao/{{ name }} || true

# ── OpenSpec (protocolo SDD) ───────────────────────────────────────────────

# En este repo NO se escribe código ni tests antes de tener un change proposal
# aprobado en openspec/changes/.
#
# Puerta del protocolo: falla si no hay ningún change proposal activo.
check-spec:
    @ls openspec/changes/*/proposal.md >/dev/null 2>&1 || (echo "❌ No hay ningún change proposal activo en openspec/changes/." && echo "   Crea uno con: just spec-new <feature>" && echo "   Y después rellena openspec/changes/<feature>/{proposal,tasks}.md y specs/<capability>/spec.md" && exit 1)
    @echo "✅ Hay change proposal activo."

# `openspec init` configura los ficheros de instrucciones de la herramienta que
# le pases, y `--tools opencode` escribe en AGENTS.md, que aquí es un documento
# vivo del repo. Con `--tools none` solo crea la estructura openspec/.
#
# Inicializa OpenSpec (una sola vez), exigiendo el árbol limpio a propósito.
spec-init:
    @test -z "$(git status --porcelain)" || (echo "❌ Hay cambios sin commitear. Commitea antes: openspec init puede reescribir ficheros." && exit 1)
    openspec init --tools none --language es

# Crea el directorio de un change proposal nuevo
spec-new change_name:
    openspec new change {{ change_name }}

# Lista los change proposals activos
spec-list:
    @openspec list

# Valida un change proposal en modo estricto
spec-validate change_name:
    openspec validate {{ change_name }} --strict

# Estado de los artefactos de un change (qué falta por escribir)
spec-status change_name:
    openspec status --change {{ change_name }}

# Archiva un change completado y funde su delta en openspec/specs/
spec-archive change_name:
    openspec archive --yes {{ change_name }}

# ── Planes y documentación (convención fija del repo) ─────────────────────
# Un plan por trabajo en plans/PLAN-NNN.md; al cerrarlo se RENOMBRA (nunca se
# borra) a -COMPLETED o -PENDING. Un documento por feature en
# docs/FEATURE-NNN-<slug>.md. Ver AGENTS.md.

# Lista los planes con su estado
plans:
    #!/usr/bin/env bash
    set -euo pipefail
    shopt -s nullglob
    files=(plans/PLAN-*.md)
    if [ ${#files[@]} -eq 0 ]; then
        echo "No hay planes todavía. La ruta del primero la da: just plan-new"
        exit 0
    fi
    printf '%-30s %s\n' "FICHERO" "ESTADO"
    for f in "${files[@]}"; do
        base="$(basename "${f}" .md)"
        case "${base}" in
            *-COMPLETED) state="terminado" ;;
            *-PENDING)   state="aparcado" ;;
            *)           state="en curso" ;;
        esac
        printf '%-30s %s\n' "$(basename "${f}")" "${state}"
    done

# Imprime la ruta del siguiente plan (PLAN-NNN, contador sin huecos)
plan-new:
    #!/usr/bin/env bash
    set -euo pipefail
    shopt -s nullglob
    next=1
    for f in plans/PLAN-*.md; do
        n="${f#plans/PLAN-}"
        n="${n%%-*}"
        n="${n%.md}"
        [[ "${n}" =~ ^[0-9]+$ ]] || continue
        if (( 10#${n} >= next )); then
            next=$(( 10#${n} + 1 ))
        fi
    done
    printf 'plans/PLAN-%03d.md\n' "${next}"

# Marca un plan como terminado (renombra con git mv, nunca borra)
plan-done num:
    #!/usr/bin/env bash
    set -euo pipefail
    # 10# evita que bash lea 010 como octal.
    n="$(printf '%03d' "$(( 10#{{ num }} ))")"
    file="plans/PLAN-${n}.md"
    test -f "${file}" || { echo "❌ No existe ${file}"; exit 1; }
    if git ls-files --error-unmatch "${file}" >/dev/null 2>&1; then
        git mv "${file}" "plans/PLAN-${n}-COMPLETED.md"
    else
        mv "${file}" "plans/PLAN-${n}-COMPLETED.md"
    fi
    echo "✅ plans/PLAN-${n}-COMPLETED.md"

# Marca un plan como aparcado (renombra con git mv, nunca borra)
plan-pending num:
    #!/usr/bin/env bash
    set -euo pipefail
    n="$(printf '%03d' "$(( 10#{{ num }} ))")"
    file="plans/PLAN-${n}.md"
    test -f "${file}" || { echo "❌ No existe ${file}"; exit 1; }
    if git ls-files --error-unmatch "${file}" >/dev/null 2>&1; then
        git mv "${file}" "plans/PLAN-${n}-PENDING.md"
    else
        mv "${file}" "plans/PLAN-${n}-PENDING.md"
    fi
    echo "✅ plans/PLAN-${n}-PENDING.md"

# Crea el stub del documento de la siguiente feature entregada
feature-new slug:
    #!/usr/bin/env bash
    set -euo pipefail
    shopt -s nullglob
    next=1
    for f in docs/FEATURE-*.md; do
        n="${f#docs/FEATURE-}"
        n="${n%%-*}"
        [[ "${n}" =~ ^[0-9]+$ ]] || continue
        if (( 10#${n} >= next )); then
            next=$(( 10#${n} + 1 ))
        fi
    done
    path="$(printf 'docs/FEATURE-%03d-%s.md' "${next}" "{{ slug }}")"
    if [ -e "${path}" ]; then
        echo "❌ ${path} ya existe."
        exit 1
    fi
    {
        printf '# FEATURE-%03d — %s\n\n' "${next}" "{{ slug }}"
        printf '## Qué entrega\n\n'
        printf '## Contratos (tipos, endpoints, esquema)\n\n'
        printf '## Decisiones\n\n'
        printf '## Deuda pendiente\n'
    } > "${path}"
    echo "✅ ${path}"
    echo "   Plan asociado: $(just plan-new)"

# ── Limpieza ───────────────────────────────────────────────────────────────

# Limpia los artefactos de compilación
clean:
    cargo clean
