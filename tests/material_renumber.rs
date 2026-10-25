//! `material-renumber` — renombrado de los tokens de número del material de guion.

mod common;

use std::path::Path;

use common::{Fixture, MINI_YAML};
use podtrack::material::{
    apply_renames, apply_slug_renames, collect_slug_renames, orphan_numeric_dirs,
    plan_number_renames, renamed_component, slug_collisions, slug_component,
};

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

#[test]
fn orden_anidado_de_mas_profundo_a_menos_y_tokens_reescritos() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    common::build_nested_material(&fx.root, 843);

    let pairs = plan_number_renames(&fx.root, 843, 844);
    let position = |needle: &str| {
        pairs
            .iter()
            .position(|(src, _)| src.ends_with(needle))
            .unwrap_or_else(|| panic!("no está en el plan: {needle}"))
    };

    let deep = position("zdir/deep/e843-deep.md");
    let z = position("zdir/e843-z.md");
    let a = position("adir/e843-a.md");
    let m = position("mdir/e843-m.md");
    let top_file = position("top-0843.md");
    let e_top = position("e843-top.md");
    let top = position("0843");

    // zdir/deep/ (más profundo) antes que zdir/, adir/ y mdir/…
    assert!(deep < z && deep < a && deep < m);
    // …éstos antes que los ficheros de la raíz…
    assert!(z < top_file && z < e_top && a < top_file && m < e_top);
    // …y la carpeta del episodio, la última.
    assert_eq!(pairs.last().expect("last").0, fx.root.join("0843"));
    assert_eq!(top, pairs.len() - 1);

    // La secuencia más larga no aparece en el plan.
    assert!(
        pairs
            .iter()
            .all(|(src, _)| !src.ends_with("08430-largo.md"))
    );

    // Se aplica y todos los tokens quedan reescritos; el largo, intacto.
    let mut out = Vec::new();
    apply_renames(&pairs, &fx.root, &mut out).expect("aplicar");
    let dest = fx.root.join("0844");
    assert!(dest.join("top-0844.md").exists());
    assert!(dest.join("e844-top.md").exists());
    assert!(dest.join("zdir/e844-z.md").exists());
    assert!(dest.join("adir/e844-a.md").exists());
    assert!(dest.join("mdir/e844-m.md").exists());
    assert!(dest.join("zdir/deep/e844-deep.md").exists());
    assert!(dest.join("zdir/deep/08430-largo.md").exists());
    assert!(!fx.root.join("0843").exists());
}

// ── por slug (fase 3) ───────────────────────────────────────────────

#[test]
fn slug_component_reescribe_las_formas_de_numero() {
    assert_eq!(slug_component("0843", 843, "mi-episodio"), "mi-episodio");
    assert_eq!(
        slug_component("e843-articulo.md", 843, "mi-episodio"),
        "mi-episodio-articulo.md"
    );
    assert_eq!(
        slug_component("0843-notas.md", 843, "mi-episodio"),
        "mi-episodio-notas.md"
    );
    assert_eq!(
        slug_component("843-notas.md", 843, "mi-episodio"),
        "mi-episodio-notas.md"
    );
    // `episodio-NNN` conserva la palabra.
    assert_eq!(
        slug_component("episodio-843-guion.md", 843, "mi-episodio"),
        "episodio-mi-episodio-guion.md"
    );
    // No toca secuencias de dígitos más largas.
    assert_eq!(slug_component("08430", 843, "mi-episodio"), "08430");
}

#[test]
fn collect_slug_renames_termina_en_la_carpeta_del_episodio() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 843);
    let src = fx.root.join("0843");

    let pairs = collect_slug_renames(&fx.root, &src, 843, "mi-episodio");
    // El plan termina en la carpeta del episodio (`0843` → `mi-episodio`).
    let last = pairs.last().expect("last");
    assert_eq!(last.0, src);
    assert_eq!(last.1, fx.root.join("mi-episodio"));
    // Los tokens internos se reescriben (destino intermedio, dentro de `0843`).
    assert!(
        pairs
            .iter()
            .any(|(_, dst)| dst.ends_with("0843/guion-mi-episodio.md"))
    );
    assert!(
        pairs
            .iter()
            .any(|(_, dst)| dst.ends_with("0843/investigacion-mi-episodio.md"))
    );
    assert!(
        pairs
            .iter()
            .any(|(_, dst)| dst.ends_with("0843/ejemplos-mi-episodio"))
    );
    // El contenido sin número no aparece en el plan.
    assert!(!pairs.iter().any(|(src, _)| src.ends_with("datos.db")));
    assert!(
        !pairs
            .iter()
            .any(|(src, _)| src.ends_with("presentacion.typ"))
    );

    // Aplicado, el árbol final queda con tokens por slug y contenido intacto.
    let mut out = Vec::new();
    apply_slug_renames(&pairs, &fx.root, &mut out).expect("apply");
    assert!(!fx.root.join("0843").exists());
    let dest = fx.root.join("mi-episodio");
    assert!(dest.join("guion-mi-episodio.md").exists());
    assert!(dest.join("investigacion-mi-episodio.md").exists());
    assert!(dest.join("notas-mi-episodio.md").exists());
    assert!(dest.join("episodio-mi-episodio.md").exists());
    assert!(dest.join("ADR-episodio-mi-episodio.md").exists());
    assert!(dest.join("ejemplos-mi-episodio/demo.sh").exists());
    assert!(dest.join("datos.db").exists());
    assert!(dest.join("presentacion.typ").exists());
    assert!(dest.join("template/base.typ").exists());
}

#[test]
fn slug_collisions_detecta_convergencia_y_destino_existente() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let root = fx.root.join("0843");
    std::fs::create_dir_all(&root).expect("mkdir");
    std::fs::write(root.join("e843-x.md"), "x\n").expect("write");
    std::fs::write(root.join("0843-x.md"), "x\n").expect("write");

    let pairs = collect_slug_renames(&fx.root, &root, 843, "dup");
    let problems = slug_collisions(&pairs);
    assert!(
        problems.iter().any(|p| p.contains("varios orígenes")),
        "problemas: {problems:?}"
    );

    // Destino ya existente.
    let fx2 = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let root2 = fx2.root.join("0843");
    std::fs::create_dir_all(&root2).expect("mkdir");
    std::fs::write(root2.join("guion-0843.md"), "x\n").expect("write");
    std::fs::create_dir_all(fx2.root.join("dup")).expect("mkdir dest");
    let pairs2 = collect_slug_renames(&fx2.root, &root2, 843, "dup");
    let problems2 = slug_collisions(&pairs2);
    assert!(
        problems2.iter().any(|p| p.contains("el destino ya existe")),
        "problemas: {problems2:?}"
    );
}

#[test]
fn orphan_numeric_dirs_lista_solo_cuatro_digitos_sin_episodio() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    for name in ["0100", "0150", "08430", "notas"] {
        std::fs::create_dir_all(fx.root.join(name)).expect("mkdir");
    }
    let tracker = podtrack::tracker::load_tracker(&fx.tracker).expect("load");
    let numbers: std::collections::HashSet<u32> =
        tracker.episodes.iter().filter_map(|e| e.number).collect();

    let orphans = orphan_numeric_dirs(&fx.root, &numbers);
    // 0100 está en el tracker; 08430 no casa `\d{4}`; notas tampoco.
    assert_eq!(orphans, vec![("0150".to_string(), 150)]);
}
