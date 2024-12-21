//! Automation tools and scripts.

use std::{fs::create_dir_all, path::PathBuf};

use clap::Parser;
use json::object;
use leveldb::Options;
use miette::IntoDiagnostic;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum DbKind {
    LevelDb,
}

#[derive(Parser)]
pub struct GenerateArgs {
    /// Kind of embedded database to generate.
    #[clap(long, value_enum, default_value_t=DbKind::LevelDb)]
    pub kind: DbKind,
    /// Path to store DB
    #[clap(long)]
    pub output: PathBuf,
}

#[derive(Parser)]
pub enum Cmd {
    Generate(GenerateArgs),
}

fn main() -> miette::Result<()> {
    let cmd = Cmd::parse();

    match cmd {
        Cmd::Generate(args) => generate(args),
    }
}

fn generate(args: GenerateArgs) -> miette::Result<()> {
    create_dir_all(&args.output).into_diagnostic()?;

    let mut db =
        leveldb::DB::open(args.output.join("leveldb"), Options::default()).into_diagnostic()?;

    // JSON
    let json_value = object! {
        "value": 0.5,
        "map": {
            "value-inner": "123"
        },
    };

    let mut buf = Vec::new();
    json_value
        .write(&mut buf)
        .expect("in memory writers never return errors");
    db.put(b"json-small", &buf).into_diagnostic()?;

    const BIG_JSON: &str = include_str!("../assets/big.json");

    db.put(b"json-big", BIG_JSON.as_bytes()).into_diagnostic()?;

    // CBOR
    let mut e = cbor::Encoder::from_memory();
    e.encode(&[vec![(true, (), 1), (false, (), 2)]]).unwrap();
    db.put(b"cbor", e.as_bytes()).into_diagnostic()?;

    // some random unparseble bytes which should be marked as HEX
    db.put(b"hex", &[1, 2, 3, 4, 5, 5, 6,]).into_diagnostic()?;

    db.flush().into_diagnostic()?;

    Ok(())
}
