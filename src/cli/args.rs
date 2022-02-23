use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]

/// Holds application-wide command line arguments.
pub struct Args {
    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str))]
    pub(crate) config: Option<PathBuf>,

    #[clap(short, long)]
    /// Only preview current action.
    preview: bool,

    #[clap(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug, PartialEq)]
/// Holds per-subcommand command line arguments.
pub enum Command {
    /// Clears the history
    ClearHistory {
        #[clap(short, long)]
        /// Only preview current action.
        preview: bool,
    },
    /// Lists all scripts.
    ListScripts,
    /// Inspects script {name}.
    InspectScript {
        /// Name of script.
        name: String,

        /// Render Abstract Syntax Tree
        #[clap(short, long)]
        render_ast: bool,
    },
    /// Undo {times} times.
    Undo {
        #[clap(short, long)]
        /// Only preview current action.
        preview: bool,
        /// Times to undo.
        #[clap(default_value_t = 1)]
        times: usize,
    },
    /// Redo {times} times.
    Redo {
        #[clap(short, long)]
        /// Only preview current action.
        preview: bool,
        /// Times to redo
        times: usize,
    },
    /// Rename files according to their tags.
    Rename {
        #[clap(short, long)]
        /// Only preview current action.
        preview: bool,

        #[clap(short, long, default_value_t=Args::DEFAULT_RECURSION_DEPTH)]
        /// Maximum recursion depth when gathering files.
        recursion_depth: usize,

        /// Name of script.
        name: String,

        /// Arguments of script.
        arguments: Vec<String>,
    },
}

impl Args {
    pub const DEFAULT_PREVIEW_AMOUNT: usize = 8;
    pub const DEFAULT_RECURSION_DEPTH: usize = 4;

    fn aggregate_preview(mut self, preview_override: bool) -> Self {
        let preview_aggregate = preview_override
            || self.preview
            || match self.command {
                Command::ClearHistory { preview, .. }
                | Command::Undo { preview, .. }
                | Command::Redo { preview, .. }
                | Command::Rename { preview, .. } => preview,
                _ => false,
            };

        self.preview = preview_aggregate;

        match &mut self.command {
            Command::ClearHistory { preview, .. }
            | Command::Undo { preview, .. }
            | Command::Redo { preview, .. }
            | Command::Rename { preview, .. } => *preview = preview_aggregate,
            _ => (),
        };

        self
    }
}

/// Parses arguments
pub(crate) fn parse_args(preview_override: bool) -> Args {
    Args::parse().aggregate_preview(preview_override)
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse_custom_args(args: &[&str], preview_override: bool) -> Args {
        Args::parse_from(args).aggregate_preview(preview_override)
    }

    #[test]
    fn test_preview_aggregate() {
        let args_in =
            ["tfmttest clear-history -p", "tfmttest -p clear-history"];

        let args_out: Vec<Args> = args_in
            .into_iter()
            .map(|a| {
                parse_custom_args(
                    &a.split_whitespace().collect::<Vec<&str>>(),
                    false,
                )
            })
            .collect();

        let equal = args_out.windows(2).all(|w| w[0] == w[1]);

        assert!(equal)
    }
}
