mod backend;
mod common;
mod frontend;

use {
    backend::builder::Thrushc,
    colored::{Colorize, control},
    common::{cli::Cli, logging},
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
    static ref LLVM_BACKEND: PathBuf = {
        let error = || {
            logging::log(
                logging::LogType::Panic,
                "The LLVM Toolchain was corrupted from the thrush toolchain; reinstall the entire toolchain across ~ `thorium install`.",
            );
        };

        if let Some(os_home) = HOME.as_ref() {
            let llvm_backend_required_paths: [PathBuf; 8] = [
                os_home.join("thrushlang"),
                os_home.join("thrushlang/backends"),
                os_home.join("thrushlang/backends/llvm"),
                os_home.join("thrushlang/backends/llvm/tools"),
                os_home.join("thrushlang/backends/llvm/ld.lld"),
                os_home.join("thrushlang/backends/llvm/clang-17"),
                os_home.join("thrushlang/backends/llvm/tools/opt"),
                os_home.join("thrushlang/backends/llvm/tools/llvm-dis"),
            ];

            llvm_backend_required_paths.iter().for_each(|path| {
                if !path.exists() {
                    error()
                }
            });

            return os_home.join("thrushlang/backends/llvm/");
        }

        error();
        unreachable!()
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
