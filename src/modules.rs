use std::string::String;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::Command;
use tl::NodeHandle;
use walkdir::WalkDir;
use crate::config::DirtConfig;

#[derive(Clone, Debug)]
pub struct Module {
    path: String,
    name: String
}

pub fn find_modules(dirt_config: &DirtConfig) -> Vec<Module> {
    let current_dir = std::env::current_dir().unwrap();
    let mut modules = Vec::new();
    let path = std::path::Path::new(&dirt_config.serve_dir);
    let search_path = if path.is_absolute() { path.to_path_buf() } else { current_dir.join(path) };
    println!("Looking for files in path {:?}", search_path);
    let walker = WalkDir::new(&search_path)
        .into_iter()
        .filter_map(|entry| entry.ok());

    for entry in walker {
        println!("{:?}", entry);
        let entry = entry.path();
        if let Some(extension) = entry.extension() {
            if extension != "rsr" {
                continue;
            }

            if let Some(file_name) = entry.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let (name, _) = file_name_str
                        .rsplit_once('.')
                        .unwrap();
                    modules.push(
                        Module {
                            path: entry.to_string_lossy().to_string(),
                            name: name.to_string()
                        }
                    );
                }
            }
        }
    }
    println!("Found modules: {:#?}!", modules);
    modules
}

fn ensure_lines_in_file(file_path: &Path, lines: Vec<String>) -> std::io::Result<()> {
    // Read existing lines from the file
    let existing_lines: Vec<String> = {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        reader.lines().filter_map(|line| line.ok()).collect()
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
            file.write(line.as_bytes())?;
            file.write("\n".as_bytes())?;
        }
    }
    Ok(())
}

pub const DEPS: [(&str, &str); 3] = [
    ("actix-web", "\"4.4.1\""),
    ("serde", "{ version = \"1.0.193\", features = [\"derive\"] }"),
    ("serde_json", "\"1.0.108\"")
];

fn write_deps(path: &Path) -> std::io::Result<()> {
    let dep_lines: Vec<String> = DEPS
        .into_iter()
        .map(|(package, ver)| format!("\"{}\" = {}", package, ver).to_string())
        .collect();
    ensure_lines_in_file(path, dep_lines.into())
}

macro_rules! replace_file {
    ($file_path:expr, $content:expr) => {{
        use std::fs::OpenOptions;
        use std::io::Write;

        // Open the file in write mode, creating it if it doesn't exist
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // Truncate the file if it already exists
            .open($file_path)
            .unwrap();

        // Write the new content to the file
        file.write_all($content.as_bytes()).unwrap();
    }};
}

fn extract_src_and_markup(path: &String) -> anyhow::Result<(String, String)> {
    let file_content = fs::read_to_string(path)?;
    let dom = tl::parse(file_content.as_str(), tl::ParserOptions::default())?;
    let parser = dom.parser();
    let rust_tags: Vec<NodeHandle> = dom.query_selector("rust").unwrap().collect();
    let rust = rust_tags.first().unwrap();
    let rust_parsed = rust.get(parser).unwrap();
    println!("{:#?}", rust_parsed.outer_html(parser));
    // TODO finish this implementation
    return Ok((String::new(), String::new()))
}

pub fn compile_modules(config: &DirtConfig, modules: &Vec<Module>) {
    let serve_dir_str = config.serve_dir.to_string();
    let serve_dir = Path::new(&serve_dir_str);
    let path = Path::new("app");
    let dir_builder = fs::DirBuilder::new();// tempfile::tempdir().unwrap();
    if let Ok(()) = dir_builder.create(path) {
        // If the directory was just created, run cargo init.
        Command::new("cargo")
            .arg("init")
            .arg("--lib")
            .arg("--vcs")
            .arg("none")
            .arg(path)
            .spawn()
            .expect("Could not run cargo init for module folder")
            .wait()
            .expect("Could not wait for cargo init to complete for some reason?");
    }
    // Make sure that cargo has the minimal dependencies
    let cargo_pb = path.join("Cargo.toml");
    let cargo_path = cargo_pb.as_path();
    write_deps(cargo_path).expect(format!("Could not add deps to {:?}/Cargo.toml", path).as_str());

    // Now run through each of the modules and move their Rust implementation into files.
    for module in modules {
        if let Ok((src, markup)) = extract_src_and_markup(&module.path) {
            let markup_path = serve_dir.join(format!("{}.html", module.name));
            replace_file!(markup_path, markup);
            let src_path = path.join(format!("{}.rs", module.name));
            replace_file!(src_path, src);
        }
    }
}

