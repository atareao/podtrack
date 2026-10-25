//! CLI de `podtrack` (`clap` derive). Solo parsea y despacha a la librería.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use podtrack::commands::add::{AddParams, parse_add_positionals};
use podtrack::commands::backfill_slugs::BackfillParams;
use podtrack::commands::insert::InsertParams;
use podtrack::commands::list::Filters;
use podtrack::commands::migrate_material::MigrateParams;
use podtrack::commands::move_cmd::MoveParams;
use podtrack::commands::publish::PublishParams;
use podtrack::commands::rebase_references::RebaseParams;
use podtrack::commands::rename_material::RenameMaterialParams;
use podtrack::commands::renumber::RenumberParams;
use podtrack::commands::srt_stage::{SrtStageParams, run as run_srt_stage};
use podtrack::commands::update::UpdateParams;
use podtrack::commands::{
    Ctx, Rendered, add, backfill_slugs, get, insert, list, list_range, migrate_material, move_cmd,
    next, next_date, publish, rebase_references, rename_material, renumber, status, update,
    where_cmd,
};
use podtrack::error::PodtrackError;
use podtrack::json;
use podtrack::profile::ShowResolver;
use podtrack::tracker;

#[derive(Parser)]
#[command(
    name = "podtrack",
    version,
    about = "Rastreador de la parrilla y del ciclo de vida de los episodios de un podcast"
)]
struct Cli {
    /// Perfil de podcast a usar (`shows/<SLUG>.yaml`).
    #[arg(long, global = true, value_name = "SLUG")]
    show: Option<String>,

    /// Emitir un documento JSON (opt-in) en lugar del texto.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Mostrar el estado general del tracker.
    Status,
    /// Mostrar el hueco y el próximo número a publicar.
    Next,
    /// Mostrar la siguiente fecha de publicación.
    NextDate,
    /// Listar episodios (filtrable).
    List {
        /// Filtrar por estado.
        #[arg(long, value_parser = ["pending", "scripted", "recorded", "published"])]
        status: Option<String>,
        /// Filtrar por temporada.
        #[arg(long)]
        season: Option<u16>,
        /// Mostrar solo los últimos N episodios.
        #[arg(long)]
        last: Option<usize>,
    },
    /// Listar episodios en un rango numérico.
    ListRange {
        /// Número inicial.
        start: u64,
        /// Número final.
        end: u64,
    },
    /// Mostrar el detalle de un episodio (por número o slug).
    Get {
        /// Número o slug del episodio.
        target: String,
    },
    /// Mostrar la ruta del material de un episodio (no crea nada).
    Where {
        /// Número o slug del episodio.
        target: String,
    },
    /// Añadir un episodio (`scripted`), con número opcional.
    Add {
        /// Número (opcional) y título: `add 823 "Título"` o `add "Título"`.
        #[arg(value_name = "[num] título", num_args = 1..)]
        items: Vec<String>,
        /// Slug kebab-case del episodio (opcional).
        #[arg(long)]
        slug: Option<String>,
        /// Fecha de publicación `YYYY-MM-DD` (por defecto, la siguiente disponible).
        #[arg(long, short = 'd')]
        date: Option<String>,
        /// Temporada (por defecto, la actual).
        #[arg(long, short = 's')]
        season: Option<u16>,
    },
    /// Cambiar el estado de un episodio.
    Update {
        /// Número o slug del episodio.
        target: String,
        /// Nuevo estado.
        #[arg(
            long,
            short = 's',
            value_parser = ["pending", "scripted", "recorded", "published"]
        )]
        status: String,
    },
    /// Publicar un episodio estampando su número público autoritativo.
    Publish {
        /// Número o slug del episodio.
        target: String,
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Insertar un episodio en el hueco de otro (desplaza números, fechas y material).
    Insert {
        /// Número del episodio ante el que insertar.
        #[arg(long)]
        before: u32,
        /// Título del episodio nuevo.
        #[arg(long)]
        title: String,
        /// Slug kebab-case del episodio (opcional).
        #[arg(long)]
        slug: Option<String>,
        /// Fecha del slot (debe coincidir con la de `--before`).
        #[arg(long, short = 'd')]
        date: Option<String>,
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Cambiar la fecha de emisión de un episodio.
    Move {
        /// Número o slug del episodio.
        target: String,
        /// Nueva fecha `YYYY-MM-DD` (lunes o jueves).
        #[arg(long, short = 'd')]
        date: String,
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Renumerar la parrilla (solo números, sin fechas ni material).
    Renumber {
        /// Primer número afectado (`>=`).
        #[arg(long, value_name = "NUM", allow_hyphen_values = true)]
        from: i64,
        /// Desplazamiento a aplicar (±d).
        #[arg(long, allow_hyphen_values = true)]
        delta: i64,
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Renombrar carpetas `0NNN` y ficheros `0NNN`/`eNNN` del árbol de guiones.
    RenameMaterial {
        /// Primer número afectado (`>=`).
        #[arg(long, value_name = "NUM", allow_hyphen_values = true)]
        from: Option<i64>,
        /// Desplazamiento a aplicar (±d).
        #[arg(long, allow_hyphen_values = true)]
        delta: Option<i64>,
        /// Solo detectar y listar desajustes, sin tocar nada.
        #[arg(long)]
        check: bool,
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Derivar y persistir el slug de los episodios que no lo tienen.
    BackfillSlugs {
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Renombrar el material de guiones a `<slug>` (carpetas y ficheros).
    MigrateMaterial {
        /// Incluir también los episodios publicados (por defecto se omiten).
        #[arg(long)]
        include_published: bool,
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Reescribir los hooks «Próximo episodio: NNN» del material no publicado.
    RebaseReferences {
        /// Previsualizar los cambios sin escribir nada.
        #[arg(long)]
        dry_run: bool,
    },
    /// Clasificar la etapa (raw/final) de los SRT de un podcast.
    SrtStage {
        /// Ruta a un SRT concreto: clasifica ese fichero.
        #[arg(long, value_name = "RUTA")]
        srt: Option<String>,
        /// Número de episodio: clasifica todos sus SRT.
        #[arg(long, value_name = "NN")]
        episode: Option<i64>,
        /// Exigir que el conjunto contenga esa etapa (`raw` o `final`).
        #[arg(long, value_parser = ["raw", "final"])]
        expect: Option<String>,
    },
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(rendered) => {
            print!("{}", rendered.stdout);
            eprint!("{}", rendered.stderr);
            std::process::ExitCode::from(u8::try_from(rendered.exit_code).unwrap_or(0))
        }
        Err(err) => {
            // `anyhow` solo en el borde (D4): recuperamos el error tipado para
            // conservar su mensaje exacto y su *exit code* de contrato.
            let (message, code) = match err.downcast_ref::<PodtrackError>() {
                Some(typed) => (typed.to_string(), typed.exit_code()),
                None => (format!("ERROR: {err}"), 1),
            };
            if cli.json {
                eprint!("{}", json::error_document(&message));
            } else {
                eprintln!("{message}");
            }
            std::process::ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
    }
}

fn run(cli: &Cli) -> anyhow::Result<Rendered> {
    let resolver = ShowResolver::from_env(exe_dir());

    // `srt-stage` no opera sobre la parrilla: resuelve el perfil **sin exigir
    // `tracker`** (spec `show-profile`), así que se despacha antes de cargarlo.
    if let Command::SrtStage {
        srt,
        episode,
        expect,
    } = &cli.command
    {
        let params = SrtStageParams {
            show: cli.show.clone(),
            srt: srt.clone(),
            episode: *episode,
            expect: expect.clone(),
        };
        return Ok(run_srt_stage(&resolver, &params, cli.json)?);
    }

    let show = resolver.resolve(cli.show.as_deref())?;
    let tracker = tracker::load_tracker(&show.tracker)?;
    let ctx = Ctx {
        tracker: &tracker,
        show: &show,
    };

    let rendered = match &cli.command {
        Command::Status => status::run(&ctx, cli.json)?,
        Command::Next => next::run(&ctx, cli.json)?,
        Command::NextDate => next_date::run(&ctx, cli.json)?,
        Command::List {
            status,
            season,
            last,
        } => {
            let filters = Filters {
                status: status.clone(),
                season: *season,
                last: *last,
            };
            list::run(&ctx, &filters, cli.json)?
        }
        Command::ListRange { start, end } => list_range::run(&ctx, *start, *end, cli.json)?,
        Command::Get { target } => get::run(&ctx, target, cli.json)?,
        Command::Where { target } => where_cmd::run(&ctx, target, cli.json)?,
        Command::Add {
            items,
            slug,
            date,
            season,
        } => {
            let (number, title) = parse_add_positionals(items)?;
            let params = AddParams {
                number,
                title,
                date: date.clone(),
                season: *season,
                slug: slug.clone(),
            };
            add::run(&ctx, &params, cli.json)?
        }
        Command::Update { target, status } => {
            let params = UpdateParams {
                target: target.clone(),
                status: status.clone(),
            };
            update::run(&ctx, &params, cli.json)?
        }
        Command::Publish { target, dry_run } => {
            let params = PublishParams {
                target: target.clone(),
                dry_run: *dry_run,
            };
            publish::run(&ctx, &params, cli.json)?
        }
        Command::Insert {
            before,
            title,
            slug,
            date,
            dry_run,
        } => {
            let params = InsertParams {
                before: *before,
                title: title.clone(),
                slug: slug.clone(),
                date: date.clone(),
                dry_run: *dry_run,
            };
            insert::run(&ctx, &params, cli.json)?
        }
        Command::Move {
            target,
            date,
            dry_run,
        } => {
            let params = MoveParams {
                target: target.clone(),
                date: date.clone(),
                dry_run: *dry_run,
            };
            move_cmd::run(&ctx, &params, cli.json)?
        }
        Command::Renumber {
            from,
            delta,
            dry_run,
        } => {
            let params = RenumberParams {
                from: *from,
                delta: *delta,
                dry_run: *dry_run,
            };
            renumber::run(&ctx, &params, cli.json)?
        }
        Command::RenameMaterial {
            from,
            delta,
            check,
            dry_run,
        } => {
            let params = RenameMaterialParams {
                from: *from,
                delta: *delta,
                check: *check,
                dry_run: *dry_run,
            };
            rename_material::run(&ctx, &params, cli.json)?
        }
        Command::BackfillSlugs { dry_run } => {
            let params = BackfillParams { dry_run: *dry_run };
            backfill_slugs::run(&ctx, &params, cli.json)?
        }
        Command::MigrateMaterial {
            include_published,
            dry_run,
        } => {
            let params = MigrateParams {
                include_published: *include_published,
                dry_run: *dry_run,
            };
            migrate_material::run(&ctx, &params, cli.json)?
        }
        Command::RebaseReferences { dry_run } => {
            let params = RebaseParams { dry_run: *dry_run };
            rebase_references::run(&ctx, &params, cli.json)?
        }
        // `srt-stage` se despacha antes de cargar el tracker; no llega aquí.
        Command::SrtStage { .. } => {
            anyhow::bail!("srt-stage se despacha antes de cargar el tracker")
        }
    };
    Ok(rendered)
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}
