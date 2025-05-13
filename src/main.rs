mod backend;
mod frontend;
mod middle;
mod standard;

use {
    backend::thrushc::Thrushc,
    colored::{Colorize, control},
    inkwell::targets::{InitializationConfig, Target},
    lazy_static::lazy_static,
    standard::{cli::CommandLine, logging},
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
                    "Incompatible host operating system '{}' for compilation.",
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
                    "Incompatible host operating system '{}' for compilation.",
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
    static ref LLVM_BACKEND: PathBuf = {
        let llvm_x86_64_linker: PathBuf = if cfg!(target_os = "linux") {
            HOME.join("thrushlang/backends/llvm/compilers/ld.lld")
        } else {
            HOME.join("thrushlang/backends/llvm/compilers/lld.exe")
        };

        let llvm_wasmer_linker: PathBuf = if cfg!(target_os = "linux") {
            HOME.join("thrushlang/backends/llvm/compilers/wasm-ld")
        } else {
            HOME.join("thrushlang/backends/llvm/compilers/wasm-ld.exe")
        };

        let llvm_backend_required_paths: [PathBuf; 3] = [
            HOME.join(format!(
                "thrushlang/backends/llvm/compilers/clang{}",
                *EXECUTABLE_EXTENSION
            )),
            llvm_x86_64_linker,
            llvm_wasmer_linker,
        ];

        for path in llvm_backend_required_paths.iter() {
            if !path.exists() {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!(
                        "Missinng LLVM Toolchain component: '{}'; It's time to use 'thorium toolchain llvm repair'.",
                        path.display()
                    ),
                );
            }
        }

        return HOME.join("thrushlang/backends/llvm");
    };
}

fn main() {
    if cfg!(target_os = "windows") {
        control::set_override(true);
    }

    let cli: CommandLine = CommandLine::parse(env::args().collect());

    Target::initialize_all(&InitializationConfig::default());

    if !cli.get_options().use_llvm() {
        logging::log(
            logging::LoggingType::Panic,
            "Select a backend infrastructure for example: '-llvm'.",
        );
    }

    let start_time: Instant = Instant::now();

    let compile_time: (u128, u128) =
        Thrushc::new(cli.get_options().get_files(), cli.get_options()).compile();

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
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
        ),
    );

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "\r{} {}",
            "Finished".custom_color((141, 141, 142)).bold(),
            format!(
                "{}.{}s",
                start_time.elapsed().as_secs(),
                start_time.elapsed().as_millis()
            )
            .custom_color((141, 141, 142))
            .bold(),
        ),
    );

    process::exit(0);
}
