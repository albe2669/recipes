use clap::Parser;
use cooklang::convert::System;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub latex_main: String,

    pub collections: Vec<String>,

    /// Convert to a unit system
    #[arg(short, long, alias = "system", value_name = "SYSTEM")]
    pub convert: Option<System>,
}
