use crate::latex::LatexBuilder;
use anyhow::{Context, Result};
use cooklang::{
    ingredient_list::GroupedIngredient, metadata::StdKey, Content, Converter, Item, Metadata,
    Quantity, ScaledRecipe, Step,
};

fn get_u64_meta(meta: &Metadata, key: StdKey) -> Option<u64> {
    meta.get(key).and_then(|x| x.as_u64())
}

#[derive(Debug)]
struct RecipeTime {
    prep_time: Option<u64>,
    cook_time: Option<u64>,
}

impl RecipeTime {
    fn from_metadata(metadata: &Metadata) -> Self {
        Self {
            prep_time: get_u64_meta(metadata, StdKey::PrepTime),
            cook_time: get_u64_meta(metadata, StdKey::CookTime),
        }
    }

    fn format_time(minutes: u64) -> String {
        if minutes < 60 {
            format!("{} mins", minutes)
        } else {
            let hours = minutes / 60;
            let mins = minutes % 60;
            if mins == 0 {
                format!("{} hrs", hours)
            } else {
                format!("{} hrs {} mins", hours, mins)
            }
        }
    }
}

pub fn create_recipe(recipe: ScaledRecipe, converter: &Converter) -> Result<String> {
    let title = recipe
        .metadata
        .title()
        .context("Recipe must have a title")?;
    let description = recipe
        .metadata
        .description()
        .context("Recipe must have a description")?;

    let mut latex = LatexBuilder::new();
    let recipe_content = build_recipe_content(&recipe, converter)?;

    let meta = recipe_meta(&recipe.metadata);
    let meta = meta.iter().map(|x| x.as_str()).collect::<Vec<&str>>();

    Ok(latex
        .add_simple_command("recipeheader", title)
        .add_simple_command("recipedesc", description)
        .add_command("recipemeta", &meta)
        .add_env("recipe", &recipe_content)
        .build())
}

fn build_recipe_content(recipe: &ScaledRecipe, converter: &Converter) -> Result<LatexBuilder> {
    let mut content = LatexBuilder::new();

    let ingredients = ingredient_list(&recipe.group_ingredients(converter));
    let instructions = instruction_list(recipe);

    content
        .add_env("ingredients", &ingredients)
        .add_env("instructions", &instructions);

    Ok(content)
}

fn recipe_meta(meta: &Metadata) -> Vec<String> {
    let servings = meta
        .servings()
        .and_then(|x| x.first().map(|x| x.to_string()))
        .expect("Servings must be defined");

    let times = RecipeTime::from_metadata(meta);
    let prep_time = times
        .prep_time
        .map(RecipeTime::format_time)
        .unwrap_or_default();
    let cook_time = times
        .cook_time
        .map(RecipeTime::format_time)
        .unwrap_or_default();

    vec![servings, prep_time, cook_time, "Moderate".to_string()]
}

fn format_quantity(qty: &Quantity) -> String {
    match qty.unit() {
        Some(unit) => format!("{} {}", qty.value(), unit),
        None => format!("{}", qty.value()),
    }
}

fn ingredient_list(ingredients: &Vec<GroupedIngredient>) -> LatexBuilder {
    let mut latex = LatexBuilder::new();

    for GroupedIngredient {
        ingredient,
        quantity,
        ..
    } in ingredients
    {
        if !ingredient.modifiers().should_be_listed() {
            continue;
        }

        let mut parts = Vec::new();

        if ingredient.modifiers().is_optional() {
            parts.push("\\textit{(optional)}".to_string());
        }

        if let Some(qty_str) = quantity
            .iter()
            .map(format_quantity)
            .reduce(|a, b| format!("{}, {}", a, b))
        {
            parts.push(qty_str);
        }

        parts.push(ingredient.display_name().to_string());
        latex.add_simple_command("item", &parts.join(" "));
    }

    latex
}

fn instruction_list(recipe: &ScaledRecipe) -> LatexBuilder {
    let mut latex = LatexBuilder::new();

    for section in &recipe.sections {
        for content in &section.content {
            let instruction = match content {
                Content::Step(step) => step_text(recipe, step),
                Content::Text(text) => text.clone(),
            };

            latex.add_simple_command("item", &instruction);
        }
    }

    latex
}

fn step_text(recipe: &ScaledRecipe, step: &Step) -> String {
    step.items
        .iter()
        .map(|item| match item {
            Item::Text { value } => value.clone(),
            Item::Ingredient { index } => recipe.ingredients[*index].display_name().to_string(),
            Item::Cookware { index } => recipe.cookware[*index].name.clone(),
            Item::Timer { index } => {
                format_timer(&recipe.timers[*index].quantity, &recipe.timers[*index].name)
            }
            Item::InlineQuantity { index } => format_quantity(&recipe.inline_quantities[*index]),
        })
        .collect()
}

fn format_timer(quantity: &Option<Quantity>, name: &Option<String>) -> String {
    match (quantity, name) {
        (Some(qty), Some(name)) => format!("{} ({})", format_quantity(qty), name),
        (Some(qty), None) => format_quantity(qty),
        (None, Some(name)) => name.clone(),
        (None, None) => unreachable!("Timer must have either quantity or name"),
    }
}
