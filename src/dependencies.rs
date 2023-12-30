use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use crate::config::DirtConfig;

fn ensure_lines_in_file(file_path: &Path, lines: Vec<String>) -> std::io::Result<()> {
    // Read existing lines from the file
    let existing_lines: Vec<String> = {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        reader.lines().map(|line| line.ok()).flatten().collect()
    };

    // Identify missing lines
    let missing_lines: Vec<String> = lines
        .into_iter()
        .filter(|line| !existing_lines.contains(line))
        .collect();

    // If there are missing lines, append them to the file
    if !missing_lines.is_empty() {
        let mut file = fs::OpenOptions::new().append(true).open(file_path)?;
        for line in missing_lines.into_iter() {
            file.write_all(line.as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }
    }
    Ok(())
}

const DEPS: [(&str, &str); 5] = [
    ("actix-web", "\"4.4.1\""),
    ("serde", "{ version = \"1.0.193\", features = [\"derive\"] }"),
    ("serde_json", "\"1.0.108\""),
    ("handlebars", "\"4.5.0\""),
    ("lazy_static", "\"1.4.0\""),
];

pub fn write_deps(config: &DirtConfig, path: &Path) -> std::io::Result<()> {
    let mut dep_lines: Vec<String> = DEPS
        .into_iter()
        .map(|(package, ver)| format!("\"{}\" = {}", package, ver))
        .collect();
    let additional_deps: Vec<String> = config.additional_packages.iter()
        .map(|package| format!("\"{}\" = {}", package.name, package.descriptor))
        .collect();
    dep_lines.extend(additional_deps);
    ensure_lines_in_file(path, dep_lines)
}