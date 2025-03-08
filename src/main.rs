mod backend;
mod cli;
mod constants;
mod diagnostic;
mod error;
mod frontend;
mod logging;

use {
    ahash::AHashMap as HashMap,
    backend::{
        apis::{debug, vector},
        builder::Thrushc,
    },
    cli::Cli,
    frontend::{
        lexer::{Lexer, Token},
        parser::Parser,
    },
    inkwell::targets::{InitializationConfig, Target},
    lazy_static::lazy_static,
    std::{env, path::PathBuf, process, time::Instant},
    stylic::{style, Color, Stylize},
};

lazy_static! {
    static ref HOME: Option<PathBuf> = {
        let error = |_| {
            logging::log(
                logging::LogType::PANIC,
                "Unable to get %HOME% of the system user.",
            );

            unreachable!()
        };

        match env::consts::OS {
            "windows" => Some(PathBuf::from(env::var("APPDATA").unwrap_or_else(error))),
            "linux" => Some(PathBuf::from(env::var("HOME").unwrap_or_else(error))),
            _ => None,
        }
    };
    static ref CORE_LIBRARY_PATH: HashMap<&'static str, (String, String)> = {
        if HOME.is_none() {
            logging::log(
                logging::LogType::ERROR,
                &format!("The Thrush toolchain is unreachable across the system; reinstall the entire toolchain via thorium install \"{}\".", env::consts::OS),
            );

            process::exit(1);
        }

        let mut imports: HashMap<&'static str, (String, String)> = HashMap::with_capacity(1);

        imports.insert(
            "core.fmt",
            (
                String::from("fmt.th"),
                HOME.as_ref()
                    .unwrap()
                    .join("thrushlang/core/fmt.th")
                    .to_string_lossy()
                    .to_string(),
            ),
        );

        imports
    };
    static ref LLVM_BACKEND_COMPILER: PathBuf = {
        if HOME.is_none() {
            logging::log(
                logging::LogType::ERROR,
                &format!("The LLVM toolchain was corrupted from the thrush toolchain; reinstall the entire toolchain across \"thorium install {}\".", env::consts::OS),
            );

            process::exit(1);
        }

        if !HOME.as_ref().unwrap().join("thrushlang").exists()
            || !HOME.as_ref().unwrap().join("thrushlang/backends/").exists()
            || !HOME
                .as_ref()
                .unwrap()
                .join("thrushlang/backends/llvm")
                .exists()
            || !HOME
                .as_ref()
                .unwrap()
                .join("thrushlang/backends/llvm/llvm")
                .exists()
            || !HOME
                .as_ref()
                .unwrap()
                .join("thrushlang/backends/llvm/clang/")
                .exists()
        {
            logging::log(
                logging::LogType::ERROR,
                &format!("The LLVM toolchain was corrupted from the thrush toolchain; reinstall the entire toolchain across \"thorium install {}\".", env::consts::OS),
            );

            process::exit(1);
        }

        if !HOME
            .as_ref()
            .unwrap()
            .join("thrushlang/backends/llvm/clang/bin/clang-17")
            .exists()
        {
            logging::log(
                logging::LogType::ERROR,
                &format!("Clang-17 don't exists in thrush toolchain; reinstall the entire toolchain across \"thorium install {}\".", env::consts::OS),
            );

            process::exit(1);
        } else if !HOME
            .as_ref()
            .unwrap()
            .join("thrushlang/backends/llvm/llvm/opt")
            .exists()
        {
            logging::log(
                logging::LogType::ERROR,
                &format!("Opt don't exists in thrush toolchain; reinstall the entire toolchain across \"thorium install {}\".", env::consts::OS),
            );

            process::exit(1);
        } else if !HOME
            .as_ref()
            .unwrap()
            .join("thrushlang/backends/llvm/llvm/llc")
            .exists()
        {
            logging::log(
                logging::LogType::ERROR,
                &format!("The LLVM static compiler don't exists in thrush toolchain; reinstall the entire toolchain across \"thorium install {}\".", env::consts::OS),
            );

            process::exit(1);
        } else if !HOME
            .as_ref()
            .unwrap()
            .join("thrushlang/backends/llvm/llvm/llvm-dis")
            .exists()
        {
            logging::log(
            logging::LogType::ERROR,
            &format!("The LLVM dissamsembler don't exists in thrush toolchain; reinstall the entire toolchain across \"thorium install {}\".", env::consts::OS),
        );

            process::exit(1);
        }

        HOME.as_ref().unwrap().join("thrushlang/backends/llvm/")
    };
}

fn main() {
    if !["linux", "windows"].contains(&env::consts::OS) {
        logging::log(
            logging::LogType::ERROR,
            "Compilation from unsopported operating system. Only Linux and Windows are supported.",
        );

        process::exit(1);
    }

    Target::initialize_all(&InitializationConfig::default());

    let mut cli: Cli = Cli::parse(env::args().collect());

    if cli.options.files.is_empty() {
        logging::log(logging::LogType::PANIC, "No files to compile!");
    }

    if !cli.options.include_vector_api {
        vector::compile_vector_api(&mut cli.options);
    }

    if !cli.options.include_debug_api {
        debug::compile_debug_api(&mut cli.options);
    }

    cli.options.files.sort_by_key(|file| file.name != "main.th");

    if cli.options.executable || cli.options.library || cli.options.static_library {
        cli.options
            .args
            .extend(["output/vector.o".to_string(), "output/debug.o".to_string()]);
    }

    let start_time: Instant = Instant::now();

    let compile_time: (u128, u128) = Thrushc::new(&cli.options.files, &cli.options).compile();

    println!(
        "\n{}\n{}\n\n{}\n{}\n{}",
        style("─────────────────────────────────────────")
            .bold()
            .fg(Color::Rgb(141, 141, 142)),
        style("Compile time report")
            .bold()
            .fg(Color::Rgb(141, 141, 142)),
        format_args!("Thrush Compiler: {}ms", style(compile_time.0).bold()),
        format_args!("LLVM & Clang: {}ms", style(compile_time.1).bold()),
        style("─────────────────────────────────────────")
            .bold()
            .fg(Color::Rgb(141, 141, 142))
    );

    println!(
        "\r{} {}",
        style("Finished").bold().fg(Color::Rgb(141, 141, 142)),
        style(&format!(
            "{}.{}s",
            start_time.elapsed().as_secs(),
            start_time.elapsed().as_millis()
        ))
        .bold()
        .fg(Color::Rgb(141, 141, 142))
    );
}
