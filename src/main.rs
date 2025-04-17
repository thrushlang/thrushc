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
    std::{env, path::PathBuf, process, time::Instant},
};

lazy_static! {
    static ref HOME: PathBuf = {
        let error = |_| {
            logging::log(logging::LoggingType::Panic, "Unable to get user %HOME%.");
            unreachable!()
        };

        let unsupported_os = || {
            logging::log(
                logging::LoggingType::Panic,
                &format!(
                    "No compatible operating system '{}' for host compilation.",
                    env::consts::OS
                ),
            );
            unreachable!()
        };

        match env::consts::OS {
            "windows" => PathBuf::from(env::var("APPDATA").unwrap_or_else(error)),
            "linux" => PathBuf::from(env::var("HOME").unwrap_or_else(error)),
            _ => {
                unsupported_os();
                unreachable!();
            }
        }
    };
    static ref LLVM_BACKEND: PathBuf = {
        let error = || {
            logging::log(
                logging::LoggingType::Panic,
                "The LLVM Backend was corrupted; reinstall the entire toolchain across ~ `thorium install`.",
            );
        };

        let llvm_backend_required_paths: [PathBuf; 9] = [
            HOME.join("thrushlang"),
            HOME.join("thrushlang/backends"),
            HOME.join("thrushlang/backends/llvm"),
            HOME.join("thrushlang/backends/llvm/build"),
            HOME.join("thrushlang/backends/llvm/tools"),
            HOME.join("thrushlang/backends/llvm/ld.lld"),
            HOME.join("thrushlang/backends/llvm/clang-17"),
            HOME.join("thrushlang/backends/llvm/tools/opt"),
            HOME.join("thrushlang/backends/llvm/tools/llvm-dis"),
        ];

        llvm_backend_required_paths.iter().for_each(|path| {
            if !path.exists() {
                error()
            }
        });

        return HOME.join("thrushlang/backends/llvm/");
    };
}

fn main() {
    if cfg!(target_os = "windows") {
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

    process::exit(0);
}
