//! CLI de `podtrack` (`clap` derive). Solo parsea y despacha a la librería.

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use podtrack::commands::list::Filters;
use podtrack::commands::{
    Ctx, Rendered, get, list, list_range, next, next_date, status, where_cmd,
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
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(rendered) => {
            print!("{}", rendered.stdout);
            eprint!("{}", rendered.stderr);
            std::process::ExitCode::SUCCESS
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
    };
    Ok(rendered)
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}
