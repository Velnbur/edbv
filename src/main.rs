use cli::Cli;

mod cli;
mod view;

fn main() -> miette::Result<()> {
    Cli::parse().run()
}
