use crate::core::cli::CliPaths;
use crate::core::context::AppContext;
use crate::logging::LogTarget;
use crate::prompter::flows::main_flow::MainFlow;
use crate::prompter::prompter::Prompter;

pub mod arg;
pub mod command;
pub mod config;
pub mod core;
pub mod errors;
pub mod extensions;
pub mod logging;
mod scheduler;
pub mod ui;

pub mod prompter;

fn main() {
    let paths = match CliPaths::from_env() {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    let mut ctx =
        match AppContext::new_with_paths(paths.config_path, paths.schedules_dir, paths.logs_dir) {
            Ok(ctx) => ctx,
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        };
    let prompter = Prompter::new();
    let flow = MainFlow::new(&mut ctx);

    if let Err(err) = prompter.run(flow, false) {
        ctx.logger
            .error(format!("{err}"), LogTarget::ConsoleAndFile);
    }
}
