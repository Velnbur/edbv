use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use miette::Result;

use crate::view::view;

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(flatten)]
    verbosity: Verbosity,

    /// Path to database.
    #[clap(long, short = 'p')]
    path: PathBuf,

    #[clap(subcommand)]
    cmd: Command,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    pub fn run(self) -> Result<()> {
        if self.verbosity.is_present() {
            env_logger::init();
        }

        match self.cmd {
            Command::View => self.view(),
        }
    }

    fn view(self) -> Result<()> {
        view(self.path)
    }
}

#[derive(Debug, Subcommand, Default)]
pub enum Command {
    #[default]
    View,
}
