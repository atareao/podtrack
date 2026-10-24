//! Guardián de paridad en vivo: compara, comando a comando, la salida del
//! binario con la del **oráculo** Python sobre el **mismo fixture** (texto +
//! exit codes).
//!
//! Cuando el oráculo o Python/PyYAML no están disponibles (p. ej. en CI), el
//! test **se salta** imprimiendo el motivo. Para que un *skip* no pase en vacío,
//! definiendo `PODTRACK_REQUIRE_ORACLE=1` el test **falla** si no puede comparar
//! (ver `common::require_oracle`).

mod common;

use std::path::Path;
use std::process::Command;

use common::{Fixture, MINI_YAML, MINI_YAML_V2};

const HARNESS: &str = r#"
import importlib.util, sys
from pathlib import Path
spec = importlib.util.spec_from_file_location("pt", sys.argv[1])
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
m.TRACKER_PATH = Path(sys.argv[2])
m.SHOW_NAME = sys.argv[3]
data = m.load_tracker()
cmd = sys.argv[4]
args = sys.argv[5:]
if cmd == "status":
    m.cmd_status(data)
elif cmd == "next":
    m.cmd_next(data)
elif cmd == "list":
    kw = {"status": None, "season": None, "last": None}
    i = 0
    while i < len(args):
        a = args[i]
        if a == "--status":
            kw["status"] = args[i + 1]; i += 2
        elif a == "--season":
            kw["season"] = int(args[i + 1]); i += 2
        elif a == "--last":
            kw["last"] = int(args[i + 1]); i += 2
        else:
            i += 1
    m.cmd_list(data, **kw)
elif cmd == "list-range":
    m.cmd_list_range(data, int(args[0]), int(args[1]))
elif cmd == "get":
    m.cmd_get(data, args[0])
elif cmd == "where":
    m.cmd_where(data, args[0])
else:
    print("unknown", file=sys.stderr); sys.exit(2)
"#;

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_oracle(
    python: &Path,
    module: &Path,
    harness: &Path,
    tracker: &Path,
    name: &str,
    args: &[&str],
) -> Outcome {
    let output = Command::new(python)
        .arg(harness)
        .arg(module)
        .arg(tracker)
        .arg(name)
        .args(args)
        .output()
        .expect("python oracle");
    Outcome {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn run_podtrack(fx: &Fixture, harness_dir: &Path, args: &[&str]) -> Outcome {
    let output = Command::new(env!("CARGO_BIN_EXE_podtrack"))
        .args(args)
        .env("PODTRACK_SHOWS_DIR", &fx.shows)
        .env("PODTRACK_SHOW", &fx.slug)
        .env("HOME", harness_dir)
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("podtrack");
    Outcome {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Especificaciones deterministas (sin dependencia del reloj de pared).
const SPECS: &[&[&str]] = &[
    &["status"],
    &["next"],
    &["list"],
    &["list", "--status", "published"],
    &["list", "--season", "8", "--last", "2"],
    &["list-range", "100", "102"],
    &["get", "100"],
    &["get", "101"],
    &["get", "999"],
    &["where", "100"],
    &["where", "no-existe"],
];

fn assert_parity(fx: &Fixture, name: &str) {
    let Some(module) = common::require_oracle() else {
        return;
    };
    let harness = fx.dir.path().join("oracle_harness.py");
    std::fs::write(&harness, HARNESS).expect("harness");
    let python = Path::new("python3");

    for spec in SPECS {
        let expected = run_oracle(python, &module, &harness, &fx.tracker, name, spec);
        let actual = run_podtrack(fx, fx.dir.path(), spec);
        assert_eq!(
            expected.stdout, actual.stdout,
            "stdout difiere para {spec:?}\n--- oráculo ---\n{}\n--- podtrack ---\n{}",
            expected.stdout, actual.stdout
        );
        assert_eq!(
            expected.stderr, actual.stderr,
            "stderr difiere para {spec:?}"
        );
        assert_eq!(
            expected.code, actual.code,
            "exit code difiere para {spec:?}"
        );
    }
}

#[test]
fn paridad_oraculo_fixture_mini() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    assert_parity(&fx, "Show de prueba");
}

#[test]
fn paridad_oraculo_fixture_v2() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    assert_parity(&fx, "Show de prueba");
}
