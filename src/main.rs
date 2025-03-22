mod backend;
mod cli;
mod constants;
mod diagnostic;
mod error;
mod frontend;
mod logging;

use {
    ahash::AHashMap as HashMap,
    backend::builder::Thrushc,
    cli::Cli,
    colored::{Colorize, control},
    frontend::{
        lexer::{Lexer, Token},
        parser::Parser,
    },
    inkwell::targets::{InitializationConfig, Target},
    lazy_static::lazy_static,
    std::{env, path::PathBuf, time::Instant},
};

lazy_static! {
    static ref HOME: Option<PathBuf> = {
        let error = |_| {
            logging::log(
                logging::LogType::Panic,
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
                logging::LogType::Panic,
                "The Thrush Toolchain is unreachable across the system; reinstall the entire toolchain across ~ `thorium install`.",
            );
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
        let error = || {
            logging::log(
                logging::LogType::Panic,
                "The LLVM Toolchain was corrupted from the thrush toolchain; reinstall the entire toolchain across ~ `thorium install`.",
            );
        };

        if HOME.is_none() {
            error()
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
                .join("thrushlang/backends/llvm/tools")
                .exists()
        {
            error()
        }

        if !HOME
            .as_ref()
            .unwrap()
            .join("thrushlang/backends/llvm/ld.lld")
            .exists()
            || !HOME
                .as_ref()
                .unwrap()
                .join("thrushlang/backends/llvm/clang-17")
                .exists()
            || !HOME
                .as_ref()
                .unwrap()
                .join("thrushlang/backends/llvm/tools/opt")
                .exists()
            || !HOME
                .as_ref()
                .unwrap()
                .join("thrushlang/backends/llvm/tools/llvm-dis")
                .exists()
        {
            error()
        }

        HOME.as_ref().unwrap().join("thrushlang/backends/llvm/")
    };
}

fn main() {
    if cfg!(windows) {
        control::set_override(true);
    }

    Target::initialize_all(&InitializationConfig::default());

    let cli: Cli = Cli::parse(env::args().collect());

    let start_time: Instant = Instant::now();

    let compile_time: (u128, u128) = Thrushc::new(&cli.options.files, &cli.options).compile();

    logging::write(
        logging::OutputIn::Stdout,
        format!(
            "\n{}\n{}\n\n{}\n{}\n{}\n",
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
            "Compile time report".custom_color((141, 141, 142)).bold(),
            format_args!("Thrush Compiler: {}ms", compile_time.0.to_string().bold()),
            format_args!("LLVM & Clang: {}ms", compile_time.1.to_string().bold()),
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
        )
        .as_bytes(),
    );

    logging::write(
        logging::OutputIn::Stdout,
        format!(
            "\r{} {}",
            "Finished".custom_color((141, 141, 142)).bold(),
            format!(
                "{}.{}s",
                start_time.elapsed().as_secs(),
                start_time.elapsed().as_millis()
            )
            .custom_color((141, 141, 142))
            .bold(),
        )
        .as_bytes(),
    );
}
