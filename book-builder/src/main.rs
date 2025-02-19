mod cli;
mod io;
mod latex;

use anyhow::Result;
use clap::Parser;
use cooklang::{convert::System, CooklangParser};
use io::replace_in_main_tex;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    io::clone_folder_to_target(&cli.latex_dir, &cli.latex_out_dir)?;

    let mut latex = latex::LatexBuilder::new();

    for collection in cli.collections {
        let collection_name = io::get_collection_name(&collection);
        latex.add_simple_command("chapter", &collection_name);

        handle_collection(&collection, cli.convert, &cli.latex_out_dir)?
            .iter()
            .for_each(|x| {
                latex.add_simple_command("input", x);
            });
    }

    replace_in_main_tex(&cli.latex_out_dir, &latex.build())?;

    Ok(())
}

fn handle_collection(
    collection: &str,
    convert: Option<System>,
    out_dir: &str,
) -> Result<Vec<String>> {
    let parser = CooklangParser::extended();
    let converter = parser.converter();

    let files = io::list_dir(collection)?;
    let mut result_files = Vec::with_capacity(files.len());

    let collection_name = io::get_collection_name(collection);

    for file in files {
        let contents = io::read_file(&file)?;
        let file_name = file
            .file_name()
            .expect("File must have a name")
            .to_str()
            .expect("Could not convert to str");

        let recipe = match parser.parse(&contents).into_result() {
            Ok((recipe, warnings)) => {
                warnings.eprint(file_name, &contents, true)?;
                recipe
            }
            Err(e) => {
                e.eprint(file_name, &contents, true)?;
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
        result_files.push(io::write_recipe(
            out_dir,
            &collection_name,
            file_name,
            &latex,
        )?);
    }

    Ok(result_files)
}
