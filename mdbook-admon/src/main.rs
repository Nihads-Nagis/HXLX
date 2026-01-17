// main.rs
use clap::{crate_version, Arg, ArgMatches, Command};
use mdbook_admonish::Admonish;
use mdbook_preprocessor::errors::Error;
use mdbook_preprocessor::Preprocessor;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::process;

// 1. EMBED THE LOCAL, VENDORED CSS ASSET (No CDN)
const ADMONISH_CSS: &[u8] = include_bytes!("../assets/hxbook-admonish.css");

// 2. LIST OF STATIC ASSETS FOR THE INSTALLER
static ASSETS: &[(&str, &[u8])] = &[("hxbook-admonish.css", ADMONISH_CSS)];

// 3. CONFIGURATION FOR YOUR PREPROCESSOR
const PREPROCESSOR_NAME: &str = "admonish"; // Key in `book.toml`
const BINARY_NAME: &str = "mycompany-mdbook-admonish"; // Your binary name

pub fn make_app() -> Command {
    Command::new(BINARY_NAME)
        .version(crate_version!())
        .about("Vendored mdBook preprocessor for Material Design admonishments. Uses local assets.")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
        .subcommand(
            Command::new("install")
                .arg(
                    Arg::new("dir")
                        .default_value(".")
                        .help("Root directory of the mdBook project (contains book.toml)"),
                )
                .about("Install local assets and update the book configuration"),
        )
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let matches = make_app().get_matches();

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(sub_args);
    } else if let Some(sub_args) = matches.subcommand_matches("install") {
        handle_install(sub_args);
    } else if let Err(e) = handle_preprocessing() {
        // This branch runs for the standard `mdbook build` invocation
        eprintln!("Preprocessor Error: {}", e);
        process::exit(1);
    }
}

/// Handles the main preprocessing work when invoked by `mdbook build`.
fn handle_preprocessing() -> Result<(), Error> {
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    // Optional but recommended: warn on version mismatch
    if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
        eprintln!(
            "Note: Preprocessor built for mdbook {}, called with {}.",
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processor = Admonish; // Assumes `Admonish` implements `Preprocessor`
    let processed_book = processor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;
    Ok(())
}

/// Handles the `supports <renderer>` CLI command.
fn handle_supports(sub_args: &ArgMatches) -> ! {
    let renderer = sub_args.get_one::<String>("renderer").unwrap();
    let processor = Admonish;
    // Exit code 0 for supported, 1 for unsupported
    let supported = processor.supports_renderer(renderer);
    process::exit(if supported.unwrap_or(false) { 0 } else { 1 });
}

/// --- INSTALLER LOGIC (Idempotent and Safe) ---
fn handle_install(sub_args: &ArgMatches) -> ! {
    let proj_dir = sub_args.get_one::<String>("dir").unwrap();
    let proj_dir = PathBuf::from(proj_dir);
    let config_path = proj_dir.join("book.toml");

    if !config_path.exists() {
        log::error!("Configuration file '{}' not found.", config_path.display());
        process::exit(1);
    }

    log::info!("Reading configuration from {}", config_path.display());
    let toml_content = fs::read_to_string(&config_path).unwrap_or_else(|_| {
        log::error!("Could not read '{}'.", config_path.display());
        process::exit(1);
    });

    let mut doc = toml_content
        .parse::<toml_edit::Document>()
        .unwrap_or_else(|_| {
            log::error!("Invalid TOML syntax in '{}'.", config_path.display());
            process::exit(1);
        });

    // 1. Add preprocessor configuration if missing
    if !has_preprocessor(&doc, PREPROCESSOR_NAME) {
        log::info!("Adding '{}' preprocessor configuration.", PREPROCESSOR_NAME);
        add_preprocessor(&mut doc, PREPROCESSOR_NAME, BINARY_NAME);
    }

    // 2. Add CSS to `additional-css` if missing
    let config_updated = add_assets_to_config(&mut doc, ASSETS);

    // 3. Write updated config back to disk
    if config_updated {
        log::info!("Updating configuration file.");
        let updated_toml = doc.to_string();
        if let Err(e) = fs::write(&config_path, updated_toml) {
            log::error!("Failed to write configuration: {}", e);
            process::exit(1);
        }
    }

    // 4. Write embedded asset files to the book directory
    deploy_assets(&proj_dir, ASSETS);

    log::info!("✅ Installation complete for '{}'.", PREPROCESSOR_NAME);
    log::info!("  -> Assets are now local.");
    log::info!("  -> Run `mdbook build` to use the preprocessor.");
    process::exit(0);
}

/// Deploys (writes) the embedded assets to the target directory.
fn deploy_assets(proj_dir: &PathBuf, assets: &[(&str, &[u8])]) {
    let mut wrote_any = false;
    for (filename, content) in assets {
        let file_path = proj_dir.join(filename);
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent); // Ignore error if exists
        }

        if file_path.exists() {
            log::debug!("Asset '{}' already exists, skipping.", filename);
        } else {
            if !wrote_any {
                wrote_any = true;
                log::info!("Writing local assets to: {}", proj_dir.display());
            }
            log::debug!("Writing '{}'", filename);
            if let Err(e) = fs::write(&file_path, content) {
                log::error!("Failed to write '{}': {}", filename, e);
                process::exit(1);
            }
        }
    }
}

/// --- HELPER FUNCTIONS (Boilerplate) ---

fn has_preprocessor(doc: &toml_edit::Document, name: &str) -> bool {
    doc.get("preprocessor")
        .and_then(|p| p.get(name))
        .map(|entry| entry.is_table())
        .unwrap_or(false)
}

fn add_preprocessor(doc: &mut toml_edit::Document, name: &str, command: &str) {
    use toml_edit::{value, Item, Table};
    let preprocessors = doc
        .as_table_mut()
        .entry("preprocessor")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .unwrap();

    let my_preprocessor = preprocessors
        .entry(name)
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .unwrap();

    my_preprocessor.insert("command", value(command));
}

fn add_assets_to_config(doc: &mut toml_edit::Document, assets: &[(&str, &[u8])]) -> bool {
    use toml_edit::{Array, Item, Value};
    let mut config_changed = false;

    let output_section = doc
        .as_table_mut()
        .entry("output")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .unwrap();

    let html_section = output_section
        .entry("html")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .unwrap();

    // Add .css files to `additional-css`
    for (filename, _) in assets.iter().filter(|(f, _)| f.ends_with(".css")) {
        let css_array = html_section
            .entry("additional-css")
            .or_insert(Item::Value(Value::Array(Array::new())))
            .as_array_mut()
            .unwrap();

        if !css_array.iter().any(|item| item.as_str() == Some(filename)) {
            css_array.push(*filename);
            config_changed = true;
        }
    }

    // Add .js files to `additional-js` (if you have any in the future)
    for (filename, _) in assets.iter().filter(|(f, _)| f.ends_with(".js")) {
        let js_array = html_section
            .entry("additional-js")
            .or_insert(Item::Value(Value::Array(Array::new())))
            .as_array_mut()
            .unwrap();

        if !js_array.iter().any(|item| item.as_str() == Some(filename)) {
            js_array.push(*filename);
            config_changed = true;
        }
    }
    config_changed
}
