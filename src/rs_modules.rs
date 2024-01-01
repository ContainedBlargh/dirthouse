use crate::config::DirtConfig;
use walkdir::WalkDir;
use std::string::String;

#[derive(Clone, Debug)]
pub struct RsModuleDesc {
    pub path: String,
    pub name: String
}

pub fn find_rs_modules(dirt_config: &DirtConfig) -> Vec<RsModuleDesc> {
    let current_dir = std::env::current_dir().unwrap();
    let mut modules = Vec::new();
    let path = std::path::Path::new(&dirt_config.serve_dir);
    let search_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };
    let walker = WalkDir::new(&search_path).into_iter().filter_map(|entry| entry.ok());
    for entry in walker {
        let entry = entry.path();
        if let Some(extension) = entry.extension() {
            if extension != "rs" {
                continue;
            }
            if let Some(file_name) = entry.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let (name, _) = file_name_str.rsplit_once('.').unwrap();
                    modules.push(RsModuleDesc {
                        path: entry.to_string_lossy().to_string(),
                        name: name.to_string(),
                    });
                }
            }
        }
    }
    modules
}
