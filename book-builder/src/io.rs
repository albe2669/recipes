use std::fs;

pub fn list_dir(path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let path = path.to_str().unwrap().to_string();
        files.push(path);
    }
    Ok(files)
}

pub fn read_file(path: &str) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}
