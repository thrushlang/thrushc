mod backend;
mod frontend;
mod middle;
mod standard;

use {
    backend::llvm::builder::Thrushc,
    colored::{Colorize, control},
    frontend::{
        lexer::{Lexer, Token},
        parser::Parser,
    },
    inkwell::targets::{InitializationConfig, Target},
    lazy_static::lazy_static,
    standard::{cli::Cli, logging},
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
    static ref EXECUTABLE_EXTENSION: &'static str = {
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
            "windows" => ".exe",
            "linux" => "",
            _ => {
                unsupported_os();
                unreachable!();
            }
        }
    };
    static ref LLVM_BACKEND: Option<PathBuf> = {
        let llvm_linker: PathBuf = if cfg!(target_os = "linux") {
            HOME.join("thrushlang/backends/llvm/lld")
        } else {
            HOME.join("thrushlang/backends/llvm/lld.exe")
        };

        let llvm_backend_required_paths: [PathBuf; 9] = [
            HOME.join("thrushlang"),
            HOME.join("thrushlang/backends"),
            HOME.join("thrushlang/backends/llvm"),
            HOME.join("thrushlang/backends/llvm/build"),
            HOME.join("thrushlang/backends/llvm/tools"),
            llvm_linker,
            HOME.join(format!(
                "thrushlang/backends/llvm/llc{}",
                *EXECUTABLE_EXTENSION
            )),
            HOME.join(format!(
                "thrushlang/backends/llvm/tools/opt{}",
                *EXECUTABLE_EXTENSION
            )),
            HOME.join(format!(
                "thrushlang/backends/llvm/tools/llvm-dis{}",
                *EXECUTABLE_EXTENSION
            )),
        ];

        for path in llvm_backend_required_paths.iter() {
            if !path.exists() {
                return None;
            }
        }

        return Some(HOME.join("thrushlang/backends/llvm"));
    };
}

fn main() {
    if cfg!(target_os = "windows") {
        control::set_override(true);
    }

    let cli: Cli = Cli::parse(env::args().collect());

    Target::initialize_all(&InitializationConfig::default());

    if !cli.get_options().use_llvm() {
        logging::log(
            logging::LoggingType::Error,
            "Select a backend infrastructure with flags: '--llvm'.",
        );
    }

    if cli.get_options().use_llvm() && LLVM_BACKEND.is_none() {
        logging::log(
            logging::LoggingType::Error,
            "Unable to find a valid LLVM Toolchain, you should use 'thorium setup toolchain'.",
        );
    }

    let start_time: Instant = Instant::now();

    let compile_time: (u128, u128) =
        Thrushc::new(cli.get_options().get_files(), cli.get_options()).compile();

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
