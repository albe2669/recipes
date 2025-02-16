use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn list_dir(path: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        files.push(path);
    }
    Ok(files)
}

pub fn read_file(path: &PathBuf) -> Result<String> {
    let contents = fs::read_to_string(path).context(format!("Could not read file: {:?}", path))?;
    Ok(contents)
}

pub fn clone_folder_to_target(source: &str, target: &str) -> Result<()> {
    fs::create_dir_all(target)?;

    let files = list_dir(source)?;
    for file in files {
        let target_path =
            std::path::Path::new(target).join(file.file_name().expect("File must have a name"));
        println!("Copying {:?} to {:?}", file, target_path);
        fs::copy(&file, &target_path).with_context(|| {
            format!(
                "Could not copy file {:?} to {:?}",
                file,
                target_path.display()
            )
        })?;
    }

    Ok(())
}

pub fn get_collection_name(collection_path: &str) -> String {
    let parsed_path = std::path::Path::new(collection_path);
    parsed_path
        .file_name()
        .expect("Could not get file name")
        .to_str()
        .expect("Could not convert to str")
        .to_string()
}

pub fn write_recipe(
    out_dir: &str,
    collection_name: &str,
    file_name: &str,
    contents: &str,
) -> Result<String> {
    let file_name = format!(
        "{}.tex",
        std::path::Path::new(file_name)
            .file_stem()
            .expect("File name must be present")
            .to_str()
            .expect("Could not convert to str")
    );

    let out_path = std::path::Path::new(out_dir);

    let relative_path = Path::new(collection_name).join(file_name);
    let target_dir = out_path.join(collection_name);
    let target_file = out_path.join(&relative_path);

    fs::create_dir_all(target_dir).context("Could not create target directory")?;
    fs::write(target_file, contents).context("Could not write to target file")?;

    Ok(relative_path
        .to_str()
        .expect("Could not compute relative path")
        .to_string())
}

pub fn replace_in_main_tex(out_dir: &str, new_content: &str) -> Result<()> {
    let main_tex = std::path::Path::new(out_dir).join("main.tex");
    let main_tex_contents = fs::read_to_string(&main_tex).expect("Could not read main.tex");

    let new_main_tex_contents = main_tex_contents.replace(r"%{{recipes}}", new_content);

    fs::write(main_tex, new_main_tex_contents).expect("Could not write to main.tex");

    Ok(())
}
