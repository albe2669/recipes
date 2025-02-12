mod cli;
mod io;
mod latex;

use anyhow::Result;
use clap::Parser;
use cooklang::{convert::System, CooklangParser};

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    for collection in cli.collections {
        handle_collection(&collection, cli.convert)?;
    }

    Ok(())
}

fn handle_collection(collection: &str, convert: Option<System>) -> Result<()> {
    let parser = CooklangParser::extended();
    let converter = parser.converter();

    let files = io::list_dir(collection)?;
    for file in files {
        let contents = io::read_file(&file)?;

        let recipe = match parser.parse(&contents).into_result() {
            Ok((recipe, warnings)) => {
                warnings.eprint(&file, &contents, true)?;
                recipe
            }
            Err(e) => {
                e.eprint(&file, &contents, true)?;
                return Err(e.into());
            }
        };

        // TODO: Implement the rest of the logic
        let mut scaled = recipe.default_scale();

        if let Some(system) = convert {
            let to = match system {
                System::Metric => cooklang::convert::System::Metric,
                System::Imperial => cooklang::convert::System::Imperial,
            };
            let _ = scaled.convert(to, converter);
        }

        let latex = latex::create_recipe(scaled, converter)?;
        println!("{}", latex);
    }

    Ok(())
}
