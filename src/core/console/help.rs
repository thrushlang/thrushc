use colored::Colorize;

use crate::core::console::logging;

pub fn show_help() -> ! {
    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{}",
            "The Thrush Compiler".custom_color((141, 141, 142)).bold()
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "\n\n{} {} {}\n\n",
            "Usage:".bold(),
            "thrushc".custom_color((141, 141, 142)).bold(),
            "[-flags|--flags] [files..]"
        ),
    );

    logging::write(logging::OutputIn::Stderr, "General Commands:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {}, {} {} {}\n",
            "•".bold(),
            "-h".custom_color((141, 141, 142)).bold(),
            "--help".custom_color((141, 141, 142)).bold(),
            "optional[opt|emit|print|code-model|reloc-model]"
                .custom_color((141, 141, 142))
                .bold(),
            "Show help message.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {}, {} {}\n\n",
            "•".bold(),
            "-v".custom_color((141, 141, 142)).bold(),
            "--version".custom_color((141, 141, 142)).bold(),
            "Show the version.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "General flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-build-dir".custom_color((141, 141, 142)).bold(),
            "Specifies the compiler artifacts directory.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nLinkage flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-clang-link".custom_color((141, 141, 142)).bold(),
            "path/to/clang",
            "Specifies the path for use of an external Clang for linking purpose.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-gcc-link".custom_color((141, 141, 142)).bold(),
            "path/to/gcc",
            "Specifies GNU Compiler Collection (GCC) for linking purpose.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-start".custom_color((141, 141, 142)).bold(),
            "Marks the start of arguments to the active external or built-in linking compiler.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-end".custom_color((141, 141, 142)).bold(),
            "Marks the end of arguments to the active external or built-in linker compiler.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nCompiler flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-target".custom_color((141, 141, 142)).bold(),
            "x86_64",
            "Set the target arquitecture.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-target-triple".custom_color((141, 141, 142)).bold(),
            "x86_64-pc-linux-gnu|x86_64-pc-windows-msvc",
            "Set the target triple. For more information, see 'https://clang.llvm.org/docs/CrossCompilation.html'.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-cpu".custom_color((141, 141, 142)).bold(),
            "haswell|alderlake|ivybridge|pentium|pantherlake",
            "Specify the CPU to optimize.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-cpu-features".custom_color((141, 141, 142)).bold(),
            "+sse2,+cx16,+sahf,-tbm",
            "Specify the new features of the CPU to use.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-emit".custom_color((141, 141, 142)).bold(),
            "llvm-bc|llvm-ir|asm|unopt-llvm-ir|unopt-llvm-bc|unopt-asm|obj|ast|tokens",
            "Compile the code into specified representation.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-print".custom_color((141, 141, 142)).bold(),
            "llvm-ir|unopt-llvm-ir|asm|unopt-asm|tokens",
            "Displays the final compilation on standard output.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-opt".custom_color((141, 141, 142)).bold(),
            "O0|O1|O2|O3|Os|Oz",
            "Optimization level.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-dbg".custom_color((141, 141, 142)).bold(),
            "Enable generation of debug information (DWARF).",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-dbg-for-inlining".custom_color((141, 141, 142)).bold(),
            "Enable debug information specifically optimized for inlined functions.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-dbg-for-profiling".custom_color((141, 141, 142)).bold(),
            "Emit extra debug info to support source-level profiling tools.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-dbg-dwarf-version".custom_color((141, 141, 142)).bold(),
            "v4|v5",
            "Configure the Dwarf version for debugging purposes.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nJIT compiler flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-jit".custom_color((141, 141, 142)).bold(),
            "Enable the use of the JIT compiler for code execution.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-jit-libc".custom_color((141, 141, 142)).bold(),
            "path/to/libc.so",
            "Specify the C runtime to link for code execution via the JIT compiler.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-jit-link".custom_color((141, 141, 142)).bold(),
            "path/to/raylib.so",
            "Specify, add, and link an external dynamic library for code execution via the JIT compiler.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-jit-entry".custom_color((141, 141, 142)).bold(),
            "main",
            "Specify the entry point name for the JIT compiler.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nOthers compiler flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--sanitizer".custom_color((141, 141, 142)).bold(),
            "[address|hwaddress|memory|thread|memtag]",
            "Enable the specified sanitizer. Adds runtime checks for bugs like memory errors, data races and others, with potential performance overhead.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--no-sanitize".custom_color((141, 141, 142)).bold(),
            "[bounds;coverage]",
            "Modifies certain code emissions for the selected sanitizer.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--opt-passes".custom_color((141, 141, 142)).bold(),
            "[-p{passname,passname}]",
            "Pass a list of custom optimization passes. For more information, see: 'https://releases.llvm.org/17.0.1/docs/CommandGuide/opt.html#cmdoption-opt-passname'.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--modificator-passes".custom_color((141, 141, 142)).bold(),
            "[loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions;callgraphprofile;forgetallscevinloopunroll;licmmssaaccpromcap=0;licmmssaoptcap=0;]",
            "Pass a list of custom modificator optimization passes.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "--reloc-model".custom_color((141, 141, 142)).bold(),
            "static|pic|dynamic",
            "Indicate how references to memory addresses and linkage symbols are handled."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--code-model".custom_color((141, 141, 142)).bold(),
            "[small|medium|large|kernel]",
            "Define how code is organized and accessed at machine code level."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--target-triple-darwin-variant"
                .custom_color((141, 141, 142))
                .bold(),
            "[arm64-apple-ios15.0-macabi]",
            "Specify the darwin target variant triple."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--macos-version".custom_color((141, 141, 142)).bold(),
            "[15.0.0]",
            "Specify the MacOS SDK version."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--ios-version".custom_color((141, 141, 142)).bold(),
            "[17.4.0]",
            "Specify the iOS SDK version."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}",
            "•".bold(),
            "--enable-ansi-color".custom_color((141, 141, 142)).bold(),
            "It allows ANSI color formatting in compiler diagnostics.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        "\n\nOmission compiler flags:\n\n",
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--omit-frame-pointer".custom_color((141, 141, 142)).bold(),
            "Regardless of the optimization level, it omits the emission of the frame pointer.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--omit-uwtable".custom_color((141, 141, 142)).bold(),
            "It omits the unwind table required for exception handling and stack tracing.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--omit-direct-access-external-data"
                .custom_color((141, 141, 142))
                .bold(),
            "It omits direct access to external data references, forcing all external data loads to be performed indirectly via the Global Offset Table (GOT).",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--omit-rtlib-got".custom_color((141, 141, 142)).bold(),
            "It omits the runtime library dependency on the Global Offset Table (GOT), essential when generating non-Position Independent Code (PIC) with ARM.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--omit-default-opt".custom_color((141, 141, 142)).bold(),
            "It omits default optimization that occurs even without specified optimization.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nDebug compiler flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--debug-clang-command".custom_color((141, 141, 142)).bold(),
            "Displays the generated command for Clang in the phase of linking."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--debug-gcc-commands".custom_color((141, 141, 142)).bold(),
            "Displays the generated command for GCC in the phase of linking."
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nUseful flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--export-compiler-errors"
                .custom_color((141, 141, 142))
                .bold(),
            "Export compiler error diagnostics to files."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--export-compiler-warnings"
                .custom_color((141, 141, 142))
                .bold(),
            "Export compiler warning diagnostics to files."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "--export-diagnostics-path"
                .custom_color((141, 141, 142))
                .bold(),
            "diagnostics/",
            "Specify the path where diagnostic files will be exported."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-exported-diagnostics"
                .custom_color((141, 141, 142))
                .bold(),
            "Clean the exported diagnostics directory."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-build".custom_color((141, 141, 142)).bold(),
            "Clean the compiler build folder that holds everything."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-tokens".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder that holds the lexical analysis tokens."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-assembler".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing emitted assembler."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-llvm-ir".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing the emitted LLVM IR."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-llvm-bitcode".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing emitted LLVM Bitcode."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n\n",
            "•".bold(),
            "--clean-objects".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing emitted object files."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--no-obfuscate-archive-names"
                .custom_color((141, 141, 142))
                .bold(),
            "Stop generating name obfuscation for each file; this does not apply to the final build."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n\n",
            "•".bold(),
            "--no-obfuscate-ir".custom_color((141, 141, 142)).bold(),
            "Stop generating name obfuscation in the emitted IR code."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--print-targets".custom_color((141, 141, 142)).bold(),
            "Show the current target supported."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--print-supported-cpus"
                .custom_color((141, 141, 142))
                .bold(),
            "Show the current supported CPUs for the current target.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--print-host-target-triple"
                .custom_color((141, 141, 142))
                .bold(),
            "Show the host target triple.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--print-opt-passes".custom_color((141, 141, 142)).bold(),
            "Show all available optimization passes through '--opt-passes=p{passname, passname}'.",
        ),
    );

    std::process::exit(1);
}

pub fn show_optimization_help() -> ! {
    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} [{}]\n\n",
            "Usage:".bold(),
            "thrushc".custom_color((141, 141, 142)).bold(),
            "-opt value; -opt=value; -opt:value;"
                .custom_color((141, 141, 142))
                .bold(),
            "O0|O1|O2|O3|Os|Oz",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "O0".custom_color((141, 141, 142)).bold(),
            "No optimization. Minimal compile time; produces the most predictable code for debugging.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "O1".custom_color((141, 141, 142)).bold(),
            "Basic optimization. Reduces code size and execution time without significantly increasing compile time.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "O2".custom_color((141, 141, 142)).bold(),
            "Standard optimization. Enables most stable optimizations.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "O3".custom_color((141, 141, 142)).bold(),
            "Enables SIMD vectorization and heavy inlining to maximize execution speed.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "Os".custom_color((141, 141, 142)).bold(),
            "Optimize for size. Enables all 'O2' optimizations that do not increase the size of the generated binary.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "Oz".custom_color((141, 141, 142)).bold(),
            "Aggressive size optimization. Further reduces binary size by disabling certain 'O2' optimization passes.",
        ),
    );

    std::process::exit(1);
}

pub fn show_emission_help() -> ! {
    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} [{}]\n\n",
            "Usage:".bold(),
            "thrushc".custom_color((141, 141, 142)).bold(),
            "-emit value; -emit=value; -emit:value;"
                .custom_color((141, 141, 142))
                .bold(),
            "llvm-bc|llvm-ir|asm|unopt-llvm-ir|unopt-llvm-bc|unopt-asm|obj|ast|tokens",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "llvm-bc".custom_color((141, 141, 142)).bold(),
            "Emit optimized LLVM bitcode.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "llvm-ir".custom_color((141, 141, 142)).bold(),
            "Emit optimized LLVM Intermediate Representation.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "asm".custom_color((141, 141, 142)).bold(),
            "Emit target-specific optimized assembly code.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "unopt-llvm-ir".custom_color((141, 141, 142)).bold(),
            "Emit unoptimized LLVM IR before any optimization passes.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "unopt-llvm-bc".custom_color((141, 141, 142)).bold(),
            "Emit unoptimized LLVM bitcode before any optimization passes.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "unopt-asm".custom_color((141, 141, 142)).bold(),
            "Emit unoptimized target-specific assembly before any optimizations passes.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "obj".custom_color((141, 141, 142)).bold(),
            "Emit machine-specific object file.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "ast".custom_color((141, 141, 142)).bold(),
            "Emit the compiler abstract syntax tree.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "tokens".custom_color((141, 141, 142)).bold(),
            "Emit the compiler lexical tokens.",
        ),
    );

    std::process::exit(1)
}

pub fn show_printing_help() -> ! {
    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} [{}]\n\n",
            "Usage:".bold(),
            "thrushc".custom_color((141, 141, 142)).bold(),
            "-print value; -print=value; -print:value;"
                .custom_color((141, 141, 142))
                .bold(),
            "llvm-ir|unopt-llvm-ir|asm|unopt-asm|tokens",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "llvm-ir".custom_color((141, 141, 142)).bold(),
            "Print optimized LLVM Intermediate Representation.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "unopt-llvm-ir".custom_color((141, 141, 142)).bold(),
            "Print unoptimized LLVM IR before any optimization passes.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "asm".custom_color((141, 141, 142)).bold(),
            "Print optimized target-specific assembly code.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "unopt-asm".custom_color((141, 141, 142)).bold(),
            "Print unoptimized assembly code before any optimizations.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "tokens".custom_color((141, 141, 142)).bold(),
            "Print the compiler lexical tokens.",
        ),
    );

    std::process::exit(1)
}

pub fn show_code_model_help() -> ! {
    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} [{}]\n\n",
            "Usage:".bold(),
            "thrushc".custom_color((141, 141, 142)).bold(),
            "--code-model value; --code-model=value; --code-model:value;"
                .custom_color((141, 141, 142))
                .bold(),
            "small|medium|large|kernel",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "small".custom_color((141, 141, 142)).bold(),
            "Default model. Assumes the code and data fit within a 2GB address space.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "medium".custom_color((141, 141, 142)).bold(),
            "Allows code to be in the 2GB range, but data sections can be larger or located further away.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "large".custom_color((141, 141, 142)).bold(),
            "No assumptions about addresses. Code and data can be anywhere in the 64-bit address space.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "kernel".custom_color((141, 141, 142)).bold(),
            "Maps code to the high end of the address spaces.",
        ),
    );

    std::process::exit(1)
}

pub fn show_reloc_model() -> ! {
    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} [{}]\n\n",
            "Usage:".bold(),
            "thrushc".custom_color((141, 141, 142)).bold(),
            "--reloc-model value; --reloc-model=value; --reloc-model:value;"
                .custom_color((141, 141, 142))
                .bold(),
            "static|pic|dynamic",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "static".custom_color((141, 141, 142)).bold(),
            "Non-relocatable code. Addresses are fixed at link time. Fastest, but not suitable for shared libraries.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "pic".custom_color((141, 141, 142)).bold(),
            "Position Independent Code. Required for shared libraries.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "dynamic".custom_color((141, 141, 142)).bold(),
            "Generates code that relies on a dynamic linker to resolve addresses at runtime.",
        ),
    );

    std::process::exit(1)
}
