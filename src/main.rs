mod modules;
mod config;
mod dependencies;

use std::fs;
use std::ops::Add;
use std::path::Path;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use modules::find_modules;
use crate::config::DirtConfig;
use crate::modules::{Module};

macro_rules! run {
    ($program:expr, $($arg:expr),*) => {
        {
            std::process::Command::new($program)
                $(
                    .arg($arg)
                )*
            .spawn()
            .and_then(|mut child| child.wait())
        }
    };
}

macro_rules! replace_file {
    ($file_path:expr, $content:expr) => {{
        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open($file_path)
            .unwrap();
        file.write_all($content.as_bytes())
    }};
}

fn compile(config: &DirtConfig, modules: &Vec<Module>, main_source: String) {
    let app_dir_path = Path::new(config.app_name.as_str());
    let dir_builder = fs::DirBuilder::new();// tempfile::tempdir().unwrap();


    if let Ok(()) = dir_builder.create(app_dir_path) {
        run!("cargo", "init", "--vcs", "none", app_dir_path).expect(
            "Could not run cargo init!\
            Do you have the Rust dev tools installed? If not, go to https://rustup.rs/ and get started!"
        );
    }

    // Make sure that cargo has the minimal dependencies
    let cargo_pb = app_dir_path.join("Cargo.toml");
    let cargo_path = cargo_pb.as_path();
    let expect_msg = format!("Could not add deps to {:?}/Cargo.toml", app_dir_path);
    let expect_msg_str = expect_msg.as_str();
    dependencies::write_deps(config, cargo_path).expect(expect_msg_str);

    // Now run through each of the modules and move their Rust implementation and markup into files.
    for module in modules {
        let markup_path = app_dir_path.join("src").join(format!("{}.html.hbs", module.name));
        replace_file!(markup_path, module.markup).expect("Could not replace markup file!");
        let src_path = app_dir_path.join("src").join(format!("{}.rs", module.name));
        replace_file!(src_path, module.source).expect("Could nto replace source file!");
    }

    // Then construct the main.rs file.
    let main_source = modules
        .iter()
        .map(|module| format!("mod {};\n", module.name))
        .reduce(|a, b| format!("{}\n{}", a, b))
        .unwrap_or(String::new())
        .add(main_source.as_str());

    replace_file!(app_dir_path.join("src").join("main.rs"), main_source).expect("Could not replace main file!");

    // Now compile the executable:
    let old_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(app_dir_path).expect("Could not switch working directory!");
    run!("cargo", "fmt").expect("Could not format app sources!");
    run!("cargo", "build", "--release").expect("Could not build app!");
    std::env::set_current_dir(old_dir).expect("Could not switch working directory back!");
    let release_dir = app_dir_path.join("target").join("release");
    let app = if cfg!(windows) {
        format!("{}.exe", config.app_name.as_str())
    } else {
        config.app_name.to_string()
    };
    let app = Path::new(app.as_str());
    // If deploy, copy the application to a local file.
    let release_app = release_dir.join(app);
    if let Ok(_) = fs::metadata(app) {
        if let Err(err) = fs::remove_file(app) {
            eprintln!("Could not remove previously compiled artifact '{:?}''.\n{}", app, err);
        }
    }
    if let Err(err) = fs::copy(release_app.clone(), app) {
        eprintln!("Could not move compiled artifact from '{:?}' to '{:?}'.\n{}", release_app.clone(), app, err);
    }
    if config.cleanup.unwrap_or(false) {
        fs::remove_dir_all(app_dir_path).expect("Could not cleanup app directory!")
    } else {
        let serve_path = Path::new(&config.serve_dir);
        if serve_path.is_relative() {
            let dest = app_dir_path.join(&config.serve_dir);
            println!("linking: '{:?}' -> '{:?}'", &config.serve_dir, dest);
            let result = if cfg!(windows) {
                std::os::windows::fs::symlink_dir(&config.serve_dir, app_dir_path.join(&config.serve_dir))
            } else if cfg!(unix) {
                fs::soft_link(&config.serve_dir, app_dir_path.join(&config.serve_dir))
            } else {
                panic!("Unsupported operating system type...")
            };
            if let Err(err) = result {
                eprintln!("Could not automatically link serve_dir.\n{}", err);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MainData {
    pub config: DirtConfig,
    pub modules: Vec<Module>,
}

fn main() {
    let config = config::load();
    let modules: Vec<Module> = find_modules(&config)
        .into_iter()
        .filter_map(modules::parse_module)
        .collect();
    let main_template = include_str!("main.trs");
    let handlebars = Handlebars::new();
    let main_data = MainData {
        config: config.clone(),
        modules: modules.clone(),
    };
    let rendered = handlebars
        .render_template(main_template, &main_data)
        .expect("Could not render config unto main template!");
    compile(&config, &modules, rendered);
}
