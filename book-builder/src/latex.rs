use anyhow::Result;
use cooklang::{
    ingredient_list::GroupedIngredient, metadata::StdKey, Content, Converter, Item, Metadata,
    ScaledRecipe, Section, Step,
};

pub fn create_recipe(recipe: ScaledRecipe, converter: &Converter) -> Result<String> {
    let latex = [
        &simple_command(
            "recipeheader",
            recipe.metadata.title().expect("Title must be defined"),
        ),
        &simple_command(
            "recipedesc",
            recipe
                .metadata
                .description()
                .expect("Description must be defined"),
        ),
        &recipe_meta(&recipe.metadata),
        &environ_begin("recipe"),
        &ingredient_list(&recipe.group_ingredients(converter)),
        &instruction_list(&recipe.sections()),
        &environ_end("recipe"),
    ];

    Ok(latex
        .iter()
        .map(|x| x.to_string())
        .reduce(|a, b| format!("{}\n{}", a, b))
        .unwrap())
}

fn command(command: &str, args: Vec<&str>) -> String {
    format!(
        "\\{}{}",
        command,
        args.iter()
            .map(|x| format!("{{{}}}", x))
            .collect::<Vec<String>>()
            .join("")
    )
}

fn simple_command(acommand: &str, arg: &str) -> String {
    command(acommand, vec![arg])
}

fn environ_begin(env: &str) -> String {
    format!("\\begin{{{}}}", env)
}

fn environ_end(env: &str) -> String {
    format!("\\end{{{}}}", env)
}

fn environment(env: &str, content: &str) -> String {
    format!("{}\n{}\n{}", environ_begin(env), content, environ_end(env))
}

fn format_time(time: u64) -> String {
    format!("{} mins", time)
}

fn get_u64_meta(meta: &Metadata, key: StdKey) -> Option<u64> {
    match meta.get(key) {
        Some(value) => value.as_u64(),
        None => None,
    }
}

fn recipe_meta(meta: &Metadata) -> String {
    let mut args = vec!["".to_string(); 4];

    if let Some(servings) = meta.servings() {
        args[0] = servings
            .first()
            .expect("Servings must be defined")
            .to_string()
    };

    if let Some(time) = get_u64_meta(meta, StdKey::PrepTime) {
        args[1] = format_time(time)
    };

    if let Some(time) = get_u64_meta(meta, StdKey::CookTime) {
        args[2] = format_time(time)
    };

    args[3] = "Difficulty".to_string(); // TODO: Difficulty

    command("recipemeta", args.iter().map(|x| x.as_str()).collect())
}

fn ingredient_list(ingredients: &Vec<GroupedIngredient>) -> String {
    let mut latex = Vec::with_capacity(ingredients.len() + 2);
    latex.push(environ_begin("ingredients"));

    for entry in ingredients {
        let GroupedIngredient {
            ingredient,
            quantity,
            ..
        } = entry;

        if !ingredient.modifiers().should_be_listed() {
            continue;
        }

        let mut igr = vec![];

        if ingredient.modifiers().is_optional() {
            igr.push("\textit{(optional)}");
        }

        let content = quantity
            .iter()
            .map(|qty| {
                if let Some(unit) = qty.unit() {
                    format!("{} {}", qty.value(), unit)
                } else {
                    format!("{}", qty.value())
                }
            })
            .reduce(|a, b| format!("{}, {}", a, b))
            .unwrap();
        igr.push(&content);

        let display_name = ingredient.display_name().to_string();
        igr.push(&display_name);

        latex.push(simple_command("ingredient", &igr.join(" ")));
    }

    latex.push(environ_end("ingredients"));
    latex.join("\n")
}

fn instruction_list() -> String {
    let mut latex = Vec::with_capacity(sections.len() + 2);
    latex.push(environ_begin("instructions"));

    for section in sections {
        for content in section.content.clone() {
            let instruction = match content {
                Content::Step(step) => step_text(&step),
                Content::Text(text) => text,
            };

            latex.push(simple_command("item", &instruction));
        }
    }

    latex.push(environ_end("instructions"));
    latex.join("\n")
}

fn step_text(step: &Step) -> String {
    let mut step_text = String::new();

    for item in &step.items {
        match item {
            Item::Text { value } => step_text += value,
            &Item::Ingredient { index } => {
                let igr = &recipe.ingredients[index];
                write!(
                    &mut step_text,
                    "{}",
                    igr.display_name().paint(styles().ingredient)
                )
                .unwrap();
                let pos = write_igr_count(&mut step_text, &step_igrs_dedup, index, &igr.name);
                if step_igrs_dedup[igr.name.as_str()].contains(&index) {
                    step_igrs_line.push((igr, pos));
                }
            }
            &Item::Cookware { index } => {
                let cookware = &recipe.cookware[index];
                write!(&mut step_text, "{}", cookware.name.paint(styles().cookware)).unwrap();
            }
            &Item::Timer { index } => {
                let timer = &recipe.timers[index];

                match (&timer.quantity, &timer.name) {
                    (Some(quantity), Some(name)) => {
                        let s = format!(
                            "{} ({})",
                            quantity_fmt(quantity).paint(styles().timer),
                            name.paint(styles().timer),
                        );
                        write!(&mut step_text, "{}", s).unwrap();
                    }
                    (Some(quantity), None) => {
                        write!(
                            &mut step_text,
                            "{}",
                            quantity_fmt(quantity).paint(styles().timer)
                        )
                        .unwrap();
                    }
                    (None, Some(name)) => {
                        write!(&mut step_text, "{}", name.paint(styles().timer)).unwrap();
                    }
                    (None, None) => unreachable!(), // guaranteed in parsing
                }
            }
            &Item::InlineQuantity { index } => {
                let q = &recipe.inline_quantities[index];
                write!(
                    &mut step_text,
                    "{}",
                    quantity_fmt(q).paint(styles().inline_quantity)
                )
                .unwrap()
            }
        }
    }

    step_text
}
