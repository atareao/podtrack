//! `tracker-write` — port 1:1 de los escenarios del oráculo de `add`, `update`,
//! `publish`, `insert` y `move` (texto, *exit codes* y tracker resultante).
//!
//! Se ejercita la **librería** directamente (como la suite de referencia llama a
//! `cmd_*`) sobre fixtures temporales; los escenarios que el oráculo resuelve por
//! CLI se portan por CLI (`assert_cmd`).

mod common;

use common::{
    B4_YAML, Fixture, MINI_YAML, MINI_YAML_V2, build_material, load_ctx, material_snapshot,
    read_tracker, save_tracker_file, snapshot,
};

use podtrack::commands::add::{AddParams, parse_add_positionals};
use podtrack::commands::insert::InsertParams;
use podtrack::commands::move_cmd::MoveParams;
use podtrack::commands::publish::PublishParams;
use podtrack::commands::update::UpdateParams;
use podtrack::commands::{Ctx, add, insert, move_cmd, publish, update};
use podtrack::model::{Date, Slug};
use podtrack::tracker::load_tracker;

// ─── add ────────────────────────────────────────────────────────────

#[test]
fn cmd_add_anade_scripted_y_persiste() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: Some(103),
        title: "Cuarto".to_string(),
        date: None,
        season: None,
        slug: None,
    };
    let rendered = add::run(&ctx, &params, false).expect("add");
    assert!(rendered.stdout.contains("#103"));
    assert!(rendered.stdout.contains("Cuarto"));

    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let new: Vec<_> = reloaded
        .episodes
        .iter()
        .filter(|e| e.number == Some(103))
        .collect();
    assert_eq!(new.len(), 1);
    assert_eq!(new[0].status.as_str(), "scripted");
    assert_eq!(new[0].title, "Cuarto");
    // Fecha por defecto: siguiente hueco de publicación (lunes tras el jueves 10).
    assert_eq!(new[0].date.to_iso(), "2026-09-14");
}

#[test]
fn cmd_add_duplicado_sale_con_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: Some(101),
        title: "Repetido".to_string(),
        date: None,
        season: None,
        slug: None,
    };
    let err = add::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("ya existe"));
    // No escribe.
    assert_eq!(read_tracker(&fx), MINI_YAML);
}

#[test]
fn add_sin_numero_asigna_hueco_provisional_y_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: None,
        title: "Nuevo sin número".to_string(),
        date: None,
        season: None,
        slug: Some("nuevo-slug".to_string()),
    };
    let rendered = add::run(&ctx, &params, false).expect("add");
    assert!(rendered.stdout.contains("#203"));

    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let new: Vec<_> = reloaded
        .episodes
        .iter()
        .filter(|e| e.title == "Nuevo sin número")
        .collect();
    assert_eq!(new.len(), 1);
    assert_eq!(new[0].number, Some(203));
    assert_eq!(new[0].slug.as_ref().map(|s| s.as_str()), Some("nuevo-slug"));
}

#[test]
fn add_con_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: Some(205),
        title: "Con slug".to_string(),
        date: None,
        season: None,
        slug: Some("con-slug".to_string()),
    };
    add::run(&ctx, &params, false).expect("add");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(205))
        .expect("205");
    assert_eq!(ep.slug.as_ref().map(|s| s.as_str()), Some("con-slug"));
}

#[test]
fn parse_add_positionals_acepta_numero_o_solo_titulo() {
    let only = vec!["Título".to_string()];
    assert_eq!(
        parse_add_positionals(&only).expect("ok"),
        (None, "Título".to_string())
    );
    let with_num = vec!["205".to_string(), "Título".to_string()];
    assert_eq!(
        parse_add_positionals(&with_num).expect("ok"),
        (Some(205), "Título".to_string())
    );
}

#[test]
fn parse_add_positionals_demasiados_args_error() {
    let items = vec!["205".to_string(), "Título".to_string(), "extra".to_string()];
    let err = parse_add_positionals(&items).expect_err("err");
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("Uso"));
}

#[test]
fn add_cli_sin_numero() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show Demo");
    common::run_ok(&fx, &["add", "Episodio CLI", "--slug", "cli-ep"]);
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let new: Vec<_> = reloaded
        .episodes
        .iter()
        .filter(|e| e.title == "Episodio CLI")
        .collect();
    assert_eq!(new.len(), 1);
    assert_eq!(new[0].number, Some(203));
    assert_eq!(new[0].slug.as_ref().map(|s| s.as_str()), Some("cli-ep"));
}

#[test]
fn add_deriva_slug_del_titulo() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: None,
        title: "Cómo montar un NAS".to_string(),
        date: None,
        season: None,
        slug: None,
    };
    add::run(&ctx, &params, false).expect("add");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.title == "Cómo montar un NAS")
        .expect("ep");
    assert_eq!(
        ep.slug.as_ref().map(|s| s.as_str()),
        Some("como-montar-un-nas")
    );
}

#[test]
fn add_slug_explicito_no_se_deriva() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: None,
        title: "Otro título".to_string(),
        date: None,
        season: None,
        slug: Some("slug-manual".to_string()),
    };
    add::run(&ctx, &params, false).expect("add");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.title == "Otro título")
        .expect("ep");
    assert_eq!(ep.slug.as_ref().map(|s| s.as_str()), Some("slug-manual"));
}

#[test]
fn add_slug_derivado_colisiona_y_sufija() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    // El oráculo muta `data` in-place; aquí cada alta se hace sobre el tracker
    // recién recargado (equivalente observable tras `save_tracker`).
    for _ in 0..2 {
        let (tracker, show) = load_ctx(&fx);
        let ctx = Ctx {
            tracker: &tracker,
            show: &show,
        };
        let params = AddParams {
            number: None,
            title: "Primer episodio".to_string(),
            date: None,
            season: None,
            slug: None,
        };
        add::run(&ctx, &params, false).expect("add");
    }
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let slugs: Vec<String> = reloaded
        .episodes
        .iter()
        .filter(|e| e.title == "Primer episodio")
        .map(|e| {
            e.slug
                .as_ref()
                .map(|s| s.as_str().to_string())
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(slugs, ["primer-episodio-2", "primer-episodio-3"]);
}

#[test]
fn add_titulo_sin_slug_utilizable_no_persiste_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: None,
        title: "!!!".to_string(),
        date: None,
        season: None,
        slug: None,
    };
    add::run(&ctx, &params, false).expect("add");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.title == "!!!")
        .expect("ep");
    assert!(ep.slug.is_none());
}

#[test]
fn add_sin_numero_asigna_hueco() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: None,
        title: "Cuarto sin número".to_string(),
        date: None,
        season: None,
        slug: None,
    };
    let rendered = add::run(&ctx, &params, false).expect("add");
    assert!(rendered.stdout.contains("#103"));
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.title == "Cuarto sin número")
        .expect("ep");
    assert_eq!(ep.number, Some(103));
}

#[test]
fn add_numero_explicito_se_respeta() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = AddParams {
        number: Some(200),
        title: "Con número explícito".to_string(),
        date: None,
        season: None,
        slug: None,
    };
    add::run(&ctx, &params, false).expect("add");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.title == "Con número explícito")
        .expect("ep");
    assert_eq!(ep.number, Some(200));
}

// ─── update ─────────────────────────────────────────────────────────

#[test]
fn cmd_update_cambia_estado() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = UpdateParams {
        target: "101".to_string(),
        status: "recorded".to_string(),
    };
    let rendered = update::run(&ctx, &params, false).expect("update");
    assert!(rendered.stdout.contains("scripted") && rendered.stdout.contains("recorded"));
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(101))
        .expect("101");
    assert_eq!(ep.status.as_str(), "recorded");
}

#[test]
fn cmd_update_estado_invalido_sale_con_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = UpdateParams {
        target: "101".to_string(),
        status: "inventado".to_string(),
    };
    let err = update::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("Estado inválido"));
}

#[test]
fn cmd_update_inexistente_sale_con_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = UpdateParams {
        target: "999".to_string(),
        status: "recorded".to_string(),
    };
    let err = update::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("no encontrado"));
}

#[test]
fn cmd_update_por_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = UpdateParams {
        target: "sin-numero".to_string(),
        status: "recorded".to_string(),
    };
    let rendered = update::run(&ctx, &params, false).expect("update");
    assert!(rendered.stdout.contains("Sin número"));
    assert!(rendered.stdout.contains("scripted") && rendered.stdout.contains("recorded"));
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    assert_eq!(
        common::by_slug(&reloaded, "sin-numero").status.as_str(),
        "recorded"
    );
}

#[test]
fn cmd_update_por_numero_textual_sigue_funcionando() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = UpdateParams {
        target: "202".to_string(),
        status: "published".to_string(),
    };
    update::run(&ctx, &params, false).expect("update");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(202))
        .expect("202");
    assert_eq!(ep.status.as_str(), "published");
}

#[test]
fn cmd_update_slug_inexistente_error_sin_escribir() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let original = std::fs::read(&fx.tracker).expect("read");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = UpdateParams {
        target: "no-existe".to_string(),
        status: "recorded".to_string(),
    };
    let err = update::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().to_lowercase().contains("no encontrado"));
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), original);
}

#[test]
fn update_cli_por_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show Demo");
    let out = common::run_ok(&fx, &["update", "sin-numero", "--status", "recorded"]);
    assert!(out.contains("Sin número"));
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    assert_eq!(
        common::by_slug(&reloaded, "sin-numero").status.as_str(),
        "recorded"
    );
}

// ─── publish ────────────────────────────────────────────────────────

#[test]
fn publish_estampa_max_publicados_mas_uno() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "101".to_string(),
        dry_run: false,
    };
    let rendered = publish::run(&ctx, &params, false).expect("publish");
    assert!(
        rendered
            .stdout
            .contains("Publicado provisional-uno como #101")
    );

    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = common::by_slug(&reloaded, "provisional-uno");
    assert_eq!(ep.status.as_str(), "published");
    assert_eq!(ep.number, Some(101));
}

#[test]
fn publish_no_reutiliza_un_numero_ya_publicado() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let (mut tracker, show) = load_ctx(&fx);
    tracker.episodes.push(podtrack::model::Episode {
        number: Some(105),
        slug: Some(Slug::parse("alto").expect("slug")),
        season: 9,
        date: Date::parse("2026-09-24"),
        title: "Alto".to_string(),
        status: podtrack::model::Status::Published,
    });
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "101".to_string(),
        dry_run: false,
    };
    let rendered = publish::run(&ctx, &params, false).expect("publish");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = common::by_slug(&reloaded, "provisional-uno");
    assert_eq!(ep.number, Some(106));
    assert_eq!(ep.status.as_str(), "published");
    assert!(rendered.stdout.contains("provisional #101 → #106"));
}

#[test]
fn publish_sin_publicados_empieza_en_uno() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let (mut tracker, show) = load_ctx(&fx);
    tracker.episodes[0].status = podtrack::model::Status::Scripted;
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "102".to_string(),
        dry_run: false,
    };
    publish::run(&ctx, &params, false).expect("publish");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    assert_eq!(
        common::by_slug(&reloaded, "provisional-dos").number,
        Some(1)
    );
}

#[test]
fn publish_anula_provisional_ajeno_en_colision() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "102".to_string(),
        dry_run: false,
    };
    let rendered = publish::run(&ctx, &params, false).expect("publish");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let dos = common::by_slug(&reloaded, "provisional-dos");
    assert_eq!(dos.number, Some(101));
    assert_eq!(dos.status.as_str(), "published");
    // El provisional ajeno queda anulado, no duplicado.
    assert_eq!(common::by_slug(&reloaded, "provisional-uno").number, None);
    assert!(rendered.stdout.contains("#101"));
}

#[test]
fn publish_por_slug() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "provisional-uno".to_string(),
        dry_run: false,
    };
    publish::run(&ctx, &params, false).expect("publish");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    assert_eq!(
        common::by_slug(&reloaded, "provisional-uno")
            .status
            .as_str(),
        "published"
    );
}

#[test]
fn publish_ya_publicado_sale_con_error() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let original = std::fs::read(&fx.tracker).expect("read");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "100".to_string(),
        dry_run: false,
    };
    let err = publish::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("ya está publicado"));
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), original);
}

#[test]
fn publish_inexistente_sale_con_error() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "999".to_string(),
        dry_run: false,
    };
    let err = publish::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().to_lowercase().contains("no encontrado"));
}

#[test]
fn publish_dry_run_no_escribe() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let original = std::fs::read(&fx.tracker).expect("read");
    let (tracker, show) = load_ctx(&fx);
    let before: Vec<_> = tracker.episodes.clone();
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = PublishParams {
        target: "101".to_string(),
        dry_run: true,
    };
    let rendered = publish::run(&ctx, &params, false).expect("publish");
    assert!(rendered.stdout.contains("DRY-RUN"));
    assert!(
        rendered
            .stdout
            .contains("Publicado provisional-uno como #101")
    );
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), original);
    // Tampoco muta la estructura en memoria.
    assert_eq!(tracker.episodes, before);
}

#[test]
fn publish_cli_despacha() {
    let fx = Fixture::new(B4_YAML, "demo", "Show Demo");
    let out = common::run_ok(&fx, &["publish", "provisional-uno"]);
    assert!(out.contains("Publicado provisional-uno como #101"));
}

// ─── insert ─────────────────────────────────────────────────────────

#[test]
fn insert_desplaza_numeros_fechas_y_material() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 101,
        title: "Nuevo".to_string(),
        slug: None,
        date: None,
        dry_run: false,
    };
    insert::run(&ctx, &params, false).expect("insert");

    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let by_num = |n: u32| {
        reloaded
            .episodes
            .iter()
            .find(|e| e.number == Some(n))
            .expect("num")
    };
    assert_eq!(by_num(101).title, "Nuevo");
    assert_eq!(by_num(101).date.to_iso(), "2026-09-07");
    assert_eq!(by_num(102).title, "Segundo");
    assert_eq!(by_num(102).date.to_iso(), "2026-09-10");
    assert_eq!(by_num(103).title, "Tercero");
    assert_eq!(by_num(103).date.to_iso(), "2026-09-14");

    // El material se renombra para no quedar desincronizado.
    assert!(!fx.root.join("0101").exists());
    assert!(fx.root.join("0102/guion-0102.md").exists());
    assert!(fx.root.join("0102/ejemplos-0102").is_dir());
    assert!(fx.root.join("0102/working-e102.md").exists());
    assert!(fx.root.join("0103/escaleta-0103.md").exists());
    assert!(fx.root.join("0100/guion-0100.md").exists());
    // El texto NO se rebasa (fuera de alcance).
    assert_eq!(
        std::fs::read_to_string(fx.root.join("0102/guion-0102.md")).expect("read"),
        "guion 101\n"
    );
}

#[test]
fn insert_con_slug_lo_persiste() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 101,
        title: "Nuevo".to_string(),
        slug: Some("nuevo-ep".to_string()),
        date: None,
        dry_run: false,
    };
    insert::run(&ctx, &params, false).expect("insert");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(101))
        .expect("101");
    assert_eq!(ep.slug.as_ref().map(|s| s.as_str()), Some("nuevo-ep"));
}

#[test]
fn insert_date_que_no_cuadra_no_toca_nada() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let before = snapshot(&fx.root);
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 101,
        title: "Nuevo".to_string(),
        slug: None,
        date: Some("2026-09-08".to_string()),
        dry_run: false,
    };
    let err = insert::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("no coincide"));
    assert_eq!(snapshot(&fx.root), before);
    assert_eq!(
        load_tracker(&fx.tracker)
            .expect("reload")
            .episodes
            .iter()
            .filter_map(|e| e.number)
            .collect::<Vec<_>>(),
        vec![100, 101, 102]
    );
}

#[test]
fn insert_before_inexistente_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 999,
        title: "Nuevo".to_string(),
        slug: None,
        date: None,
        dry_run: false,
    };
    let err = insert::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().to_lowercase().contains("no existe"));
}

#[test]
fn insert_dry_run_no_escribe_nada() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let before = snapshot(&fx.root);
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 101,
        title: "Nuevo".to_string(),
        slug: None,
        date: None,
        dry_run: true,
    };
    let rendered = insert::run(&ctx, &params, false).expect("insert");
    assert!(rendered.stdout.contains("DRY-RUN"));
    assert_eq!(snapshot(&fx.root), before);
    // Ni en disco ni en memoria.
    assert_eq!(
        load_tracker(&fx.tracker)
            .expect("reload")
            .episodes
            .iter()
            .filter_map(|e| e.number)
            .collect::<Vec<_>>(),
        vec![100, 101, 102]
    );
    assert_eq!(
        tracker
            .episodes
            .iter()
            .filter_map(|e| e.number)
            .collect::<Vec<_>>(),
        vec![100, 101, 102]
    );
}

// ─── move ───────────────────────────────────────────────────────────

#[test]
fn move_cambia_fecha() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = MoveParams {
        target: "101".to_string(),
        date: "2026-09-14".to_string(),
        dry_run: false,
    };
    let rendered = move_cmd::run(&ctx, &params, false).expect("move");
    assert!(rendered.stdout.contains("2026-09-14"));
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(101))
        .expect("101");
    assert_eq!(ep.date.to_iso(), "2026-09-14");
}

#[test]
fn move_por_slug() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let (mut tracker, _show) = load_ctx(&fx);
    for ep in &mut tracker.episodes {
        let slug = format!("ep-{}", ep.number.expect("num"));
        ep.slug = Some(Slug::parse(&slug).expect("slug"));
    }
    save_tracker_file(&fx, &tracker);

    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = MoveParams {
        target: "ep-101".to_string(),
        date: "2026-09-14".to_string(),
        dry_run: false,
    };
    move_cmd::run(&ctx, &params, false).expect("move");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(101))
        .expect("101");
    assert_eq!(ep.date.to_iso(), "2026-09-14");
}

#[test]
fn move_colision_avisa_y_no_escribe() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let original = std::fs::read(&fx.tracker).expect("read");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = MoveParams {
        target: "101".to_string(),
        date: "2026-09-10".to_string(),
        dry_run: false,
    };
    let err = move_cmd::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().to_lowercase().contains("colisión"));
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), original);
}

#[test]
fn move_dia_no_publicable_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = MoveParams {
        target: "101".to_string(),
        date: "2026-09-08".to_string(),
        dry_run: false,
    };
    let err = move_cmd::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().to_lowercase().contains("publicación"));
}

#[test]
fn move_dry_run_no_escribe() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let original = std::fs::read(&fx.tracker).expect("read");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = MoveParams {
        target: "101".to_string(),
        date: "2026-09-14".to_string(),
        dry_run: true,
    };
    move_cmd::run(&ctx, &params, false).expect("move");
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), original);
}

#[test]
fn move_inexistente_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = MoveParams {
        target: "999".to_string(),
        date: "2026-09-14".to_string(),
        dry_run: false,
    };
    let err = move_cmd::run(&ctx, &params, false).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().to_lowercase().contains("no encontrado"));
}

#[test]
fn move_por_numero_sigue_funcionando_con_nulos() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = MoveParams {
        target: "202".to_string(),
        date: "2026-09-14".to_string(),
        dry_run: false,
    };
    move_cmd::run(&ctx, &params, false).expect("move");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(202))
        .expect("202");
    assert_eq!(ep.date.to_iso(), "2026-09-14");
}

// ─── --dry-run no crea `.bak` ni material ───────────────────────────

#[test]
fn dry_run_no_crea_backup_ni_toca_material() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let material_before = material_snapshot(&fx.root);
    let tracker_before = std::fs::read(&fx.tracker).expect("read");

    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    insert::run(
        &ctx,
        &InsertParams {
            before: 102,
            title: "X".to_string(),
            slug: None,
            date: None,
            dry_run: true,
        },
        false,
    )
    .expect("insert dry-run");
    move_cmd::run(
        &ctx,
        &MoveParams {
            target: "100".to_string(),
            date: "2026-09-14".to_string(),
            dry_run: true,
        },
        false,
    )
    .expect("move dry-run");
    publish::run(
        &ctx,
        &PublishParams {
            target: "101".to_string(),
            dry_run: true,
        },
        false,
    )
    .expect("publish dry-run");

    assert_eq!(std::fs::read(&fx.tracker).expect("read"), tracker_before);
    assert_eq!(material_snapshot(&fx.root), material_before);
    assert!(!fx.root.join("podcast-tracker.yaml.bak").exists());
}

// ─── insert: derivación del slug (change `insert-derives-slug`) ──────

#[test]
fn insert_deriva_el_slug_del_titulo() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 101,
        title: "Cómo montar un NAS".to_string(),
        slug: None,
        date: None,
        dry_run: false,
    };
    insert::run(&ctx, &params, false).expect("insert");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(101))
        .expect("101");
    assert_eq!(
        ep.slug.as_ref().map(Slug::as_str),
        Some("como-montar-un-nas")
    );
}

#[test]
fn insert_sufija_el_slug_derivado_si_colisiona() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    // `sin-numero` ya existe en el fixture → el derivado debe sufijarse.
    let params = InsertParams {
        before: 202,
        title: "Sin número".to_string(),
        slug: None,
        date: None,
        dry_run: false,
    };
    insert::run(&ctx, &params, false).expect("insert");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(202))
        .expect("202");
    assert_eq!(ep.slug.as_ref().map(Slug::as_str), Some("sin-numero-2"));
}

#[test]
fn insert_titulo_sin_slug_utilizable_lo_deja_sin_slug() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 101,
        title: "¡¡¡".to_string(),
        slug: None,
        date: None,
        dry_run: false,
    };
    insert::run(&ctx, &params, false).expect("insert");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(101))
        .expect("101");
    assert_eq!(ep.slug, None);
}

#[test]
fn insert_slug_explicito_no_se_sufija_aunque_colisione() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (tracker, show) = load_ctx(&fx);
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };
    let params = InsertParams {
        before: 202,
        title: "Sin número".to_string(),
        slug: Some("sin-numero".to_string()),
        date: None,
        dry_run: false,
    };
    insert::run(&ctx, &params, false).expect("insert");
    let reloaded = load_tracker(&fx.tracker).expect("reload");
    let ep = reloaded
        .episodes
        .iter()
        .find(|e| e.number == Some(202))
        .expect("202");
    assert_eq!(ep.slug.as_ref().map(Slug::as_str), Some("sin-numero"));
}
