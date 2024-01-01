use std::string::String;
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::config::DirtConfig;

#[derive(Clone, Debug)]
pub struct RsrModuleDesc {
    pub path: String,
    pub name: String,
    pub route: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Service {
    pub method: String,
    pub route: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RsrModule {
    pub path: String,
    pub name: String,
    pub source: Option<String>,
    pub markup: String,
    pub has_template_fn: bool,
    pub is_index: bool,
    pub services: Vec<Service>,
    pub route: String,
}

pub fn find_rsr_modules(dirt_config: &DirtConfig) -> Vec<RsrModuleDesc> {
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
            if extension != "rsr" {
                continue;
            }
            if let Some(file_name) = entry.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let (name, _) = file_name_str.rsplit_once('.').unwrap();
                    let route = entry
                        .strip_prefix(&search_path)
                        .unwrap_or(entry)
                        .to_string_lossy()
                        .to_string();
                    let route = route.strip_suffix(".rsr").unwrap_or(route.as_str());
                    let route = route.replace('\\', "/");
                    let route = if !route.starts_with('/') {
                        format!("/{}", route)
                    } else {
                        route
                    };
                    let route = if name.eq("index") { String::from("/") } else { route };
                    modules.push(RsrModuleDesc {
                        path: entry.to_string_lossy().to_string(),
                        name: name.to_string(),
                        route,
                    });
                }
            }
        }
    }
    modules
}

struct RsrModuleContent {
    source: Option<String>,
    markup: String,
}

fn extract_src_and_markup(path: &String) -> anyhow::Result<RsrModuleContent> {
    let file_content = fs::read_to_string(path)?;
    let xml_content = file_content.as_str();
    // Define the regex patterns
    let start_pattern = r#"<rust.*>"#;
    let end_pattern = r#"</rust>"#;
    let comment_pattern = r#"(?s)<!--.*?-->"#;

    // Create the regex patterns
    let start_regex = Regex::new(start_pattern)?;
    let end_regex = Regex::new(end_pattern)?;
    let comment_regex = Regex::new(comment_pattern)?;

    // Remove top-level comments
    let content_without_comments = comment_regex.replace_all(xml_content, "");

    // Find the start and end positions of the <rust> tag
    let start_position = match start_regex.find(&content_without_comments) {
        Some(pos) => pos.end(),
        None => return Ok(RsrModuleContent {
            source: None,
            markup: content_without_comments.to_string(),
        })
    };

    let end_position = match end_regex.find(&content_without_comments) {
        Some(pos) => pos.start(),
        None => return Ok(RsrModuleContent {
            source: None,
            markup: content_without_comments.to_string(),
        })
    };

    // Extract the content inside the <rust> tag
    let src_content = content_without_comments[start_position..end_position].trim().to_string()
        .lines()
        .map(|it| it.trim())
        .collect::<Vec<_>>()
        .join("\n");

    // Extract the non-captured content
    let markup_content = format!(
        "{}{}",
        &content_without_comments[..start_position - start_pattern.len()],
        &content_without_comments[end_position + end_pattern.len()..],
    );

    Ok(RsrModuleContent {
        source: Some(src_content),
        markup: markup_content,
    })
}

fn extract_services(source_code: &str) -> Vec<Service> {
    let attribute_pattern = r#"#\[(connect|delete|get|head|main|options|patch|post|put|route|routes|dist|trace)\(([^)]*)\)\]"#;
    let attribute_regex = Regex::new(attribute_pattern).expect("Invalid regex pattern");

    let function_pattern = r#"async\s*fn\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*"#;
    let function_regex = Regex::new(function_pattern).expect("Invalid regex pattern");

    let mut services = Vec::new();

    for attribute_capture in attribute_regex.captures_iter(source_code) {
        let method = attribute_capture[1].to_string().to_uppercase();
        let route = attribute_capture[2].to_string();
        let remaining_code = &source_code[attribute_capture.get(0).unwrap().end()..];

        if let Some(function_capture) = function_regex.captures(remaining_code) {
            let route = route.replace('"', "");
            let name = function_capture[1].to_string();
            services.push(Service { method, route, name });
        }
    }
    services
}


fn has_template_fn(source_code: &str) -> bool {
    let pattern = r#"pub\s+async\s+fn\s+template\s*\(\s*req\s*:\s*HttpRequest\s*\)\s*->\s*"#;
    let regex = Regex::new(pattern).expect("Invalid regex pattern");
    regex.is_match(source_code)
}

pub fn parse_rsr_module(module_desc: RsrModuleDesc) -> Option<RsrModule> {
    extract_src_and_markup(&module_desc.path).map(|module: RsrModuleContent| {
        let markup = module.markup;
        let markup_with_route = String::from(&markup).replace("$route", module_desc.route.as_str());
        let source_with_route = module.source
            .clone()
            .map(|source| String::from(&source).replace("$route", module_desc.route.as_str()));
        let services = module.source
            .clone()
            .map(|source| extract_services(&source)).unwrap_or(Vec::new());
        let has_template_fn = module.source
            .clone()
            .map(|source| has_template_fn(&source)).unwrap_or(false);
        let is_index = (&module_desc.name).eq("index");
        RsrModule {
            path: module_desc.path,
            name: module_desc.name,
            source: source_with_route,
            markup: markup_with_route,
            has_template_fn,
            is_index,
            services,
            route: module_desc.route,
        }
    }).ok()
}