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
    #[clap(name = "clear")]
    ClearHistory {
        #[clap(short, long)]
        /// Only preview current action.
        preview: bool,
    },
    /// Lists all scripts.
    #[clap(name = "list")]
    ListScripts,
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
        #[clap(default_value_t = 1)]
        times: usize,
    },
    /// Rename files according to their tags.
    Rename {
        #[clap(short, long)]
        /// Only preview current action.
        preview: bool,

        #[clap(short, long, default_value_t=Args::DEFAULT_RECURSION_DEPTH)]
        /// Maximum recursion depth when gathering files.
        recurse: usize,

        /// Name of script.
        name: String,

        /// Arguments of script.
        arguments: Vec<String>,
    },
    #[clap(hide = true)]
    /// Adds my personal sync.tfmt to the filesystem.
    Seed,
    /// Renders script {name} abstract syntax tree.
    #[cfg(feature = "graphviz")]
    #[clap(name = "render")]
    RenderScript {
        /// Name of script.
        name: String,
    },
}

impl Args {
    pub(crate) const DEFAULT_PREVIEW_AMOUNT: usize = 8;
    pub(crate) const DEFAULT_RECURSION_DEPTH: usize = 4;

    /// If one preview is true, also sets the other preview.
    #[must_use]
    pub fn aggregate_preview(mut self, preview_override: bool) -> Self {
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
    use anyhow::Result;

    fn parse_custom_args(
        args: &[&str],
        preview_override: bool,
    ) -> Result<Args> {
        let args =
            Args::try_parse_from(args)?.aggregate_preview(preview_override);
        Ok(args)
    }

    #[test]
    fn test_preview_aggregate() -> Result<()> {
        let args_in = ["tfmttest clear -p", "tfmttest -p clear"];

        let args_out: Result<Vec<Args>> = args_in
            .into_iter()
            .map(|a| {
                parse_custom_args(
                    &a.split_whitespace().collect::<Vec<&str>>(),
                    false,
                )
            })
            .collect();

        let equal = args_out?.windows(2).all(|w| w[0] == w[1]);

        assert!(equal);

        Ok(())
    }
}
