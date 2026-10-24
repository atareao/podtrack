//! `material-renumber` — renombrado de los tokens de número del material de guion.

mod common;

use std::path::Path;

use common::{Fixture, MINI_YAML};
use podtrack::material::{apply_renames, plan_number_renames, renamed_component};

/// Construye un material con los tokens reales del árbol de guiones.
fn build(root: &Path, num: u32) -> std::path::PathBuf {
    let folder = root.join(format!("{num:04}"));
    std::fs::create_dir_all(&folder).expect("mkdir");
    std::fs::write(folder.join(format!("guion-{num:04}.md")), "x\n").expect("write");
    std::fs::write(folder.join(format!("investigacion-e{num}.md")), "x\n").expect("write");
    // Secuencia más larga: NO debe tocarse.
    std::fs::write(folder.join(format!("{num:04}0.md")), "x\n").expect("write");
    let sub = folder.join(format!("ejemplos-{num:04}"));
    std::fs::create_dir_all(&sub).expect("mkdir sub");
    std::fs::write(sub.join("demo.sh"), "x\n").expect("write");
    // Un fichero MÁS profundo que sigue llevando el token (para el orden del plan).
    std::fs::write(sub.join(format!("demo-{num:04}.sh")), "x\n").expect("write");
    folder
}

#[test]
fn token_0nnn_y_ennn_se_reescriben() {
    assert_eq!(renamed_component("0843", 843, 844), "0844");
    assert_eq!(
        renamed_component("e843-articulo.md", 843, 844),
        "e844-articulo.md"
    );
    assert_eq!(
        renamed_component("guion-0843.md", 843, 844),
        "guion-0844.md"
    );
}

#[test]
fn no_toca_secuencias_mas_largas() {
    // `08430` no es el token `0843` (va seguido de un dígito).
    assert_eq!(renamed_component("08430", 843, 844), "08430");
    assert_eq!(renamed_component("e8430", 843, 844), "e8430");
    // Precedido de un alfanumérico tampoco.
    assert_eq!(renamed_component("x0843", 843, 844), "x0843");
    // Un nombre sin el token no cambia.
    assert_eq!(renamed_component("assets", 843, 844), "assets");
}

#[test]
fn el_plan_va_de_mas_profundo_a_menos() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build(&fx.root, 101);
    let pairs = plan_number_renames(&fx.root, 101, 102);
    assert!(!pairs.is_empty());

    // La carpeta superior (`0101` → `0102`) es el ÚLTIMO par.
    let last = pairs.last().expect("last");
    assert_eq!(last.0, fx.root.join("0101"));
    assert_eq!(last.1, fx.root.join("0102"));

    // Hay un elemento más profundo (el fichero dentro de `ejemplos-0101`) antes
    // que la carpeta del episodio y antes que la subcarpeta.
    let deep = pairs
        .iter()
        .position(|(src, _)| src.ends_with("ejemplos-0101/demo-0101.sh"))
        .expect("deep");
    let sub = pairs
        .iter()
        .position(|(src, _)| src.ends_with("ejemplos-0101"))
        .expect("sub");
    assert!(deep < sub, "el fichero profundo va antes que su padre");
    assert!(
        sub < pairs.len() - 1,
        "la subcarpeta va antes que la del episodio"
    );
}

#[test]
fn destino_existente_aborta() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build(&fx.root, 101);
    std::fs::create_dir_all(fx.root.join("0102")).expect("mkdir ocupado");

    let pairs = plan_number_renames(&fx.root, 101, 102);
    let mut out = Vec::new();
    let err = apply_renames(&pairs, &fx.root, &mut out).expect_err("conflicto");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("el destino ya existe"));
    assert!(err.to_string().contains("0102"));
}

#[test]
fn se_reporta_cada_par() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build(&fx.root, 101);
    let pairs = plan_number_renames(&fx.root, 101, 102);
    let mut out = Vec::new();
    apply_renames(&pairs, &fx.root, &mut out).expect("aplicar");
    let text = String::from_utf8(out).expect("utf8");

    // Cada par aparece con origen y destino separados por la flecha del oráculo.
    assert!(text.contains("0101") && text.contains("0102"));
    assert!(text.contains("  →  "), "texto:\n{text}");
    let lines = text.lines().filter(|l| l.contains("  →  ")).count();
    assert_eq!(lines, pairs.len());

    // Y el árbol quedó renombrado.
    assert!(!fx.root.join("0101").exists());
    assert!(fx.root.join("0102/guion-0102.md").exists());
    assert!(fx.root.join("0102/ejemplos-0102/demo.sh").exists());
    // La secuencia larga quedó intacta.
    assert!(fx.root.join("0102/01010.md").exists());
}

#[test]
fn plan_vacio_si_no_existe_la_carpeta() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let pairs = plan_number_renames(&fx.root, 42, 43);
    assert!(pairs.is_empty());
}
