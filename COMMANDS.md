<img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-logo-v1.5.png" alt= "logo" style= "width: 80%; height: 80%;"> </img>

# The Thrush Compiler | Commands & Flags

A list of the commands supported by the Thrush Compiler command line.

> [!WARNING]  
> This might be a bit outdated, it could be information that's somewhat distant from the changes.

```console
The Thrush Compiler

Usage: thrushc [-flags | --flags] [files..]

General Commands:

• -h, --help, help Show help message.
• -v, --version, version Show the version.

LLVM Commands:

• llvm-print-target-triples Show the current LLVM target triples supported.
• llvm-print-targets Show the current LLVM target supported.
• llvm-print-supported-cpus Show the current LLVM supported CPUs for the current LLVM target.
• llvm-print-host-target-triple Show the host LLVM target triple.

General flags:

• -build-dir Set the build directory.
• -clang Enable embedded Clang to link.
• -gcc [usr/bin/gcc] Speciefies GNU Compiler Collection (GCC) to link.
• -custom-clang [/usr/bin/clang] Specifies the path for use of an external Clang to link.
• -start Marks the start of arguments to the active external or built-in linking compiler.
• -end Marks the end of arguments to the active external or built-in linker compiler.

Compiler flags:

• -llvm-backend Enable the usage of the LLVM backend infrastructure.
• -target [x86_64] Set the LLVM target.
• -target-triple [x86_64-pc-linux-gnu] Set the LLVM target triple.
• -cpu [haswell] Specify in LLVM the CPU to optimize.
• -cpu-features [+sse2,+cx16,+sahf,-tbm] Specify in LLVM the new features of the CPU to use.
• -emit [llvm-bc|llvm-ir|asm|raw-llvm-ir|raw-llvm-bc|raw-asm|obj|ast|tokens] Compile the code into specified representation.
• -print [llvm-ir|raw-llvm-ir|tokens] Displays the final compilation on stdout.
• -opt [O0|O1|O2|mcqueen] Optimization level.

Extra compiler flags:

• --opt-passes [-p{passname}] Pass a list of custom optimization passes to the LLVM optimizator.
• --modificator-passes [loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions] Pass a list of custom modificator passes to the LLVM optimizator.
• --reloc [static|pic|dynamic] Indicate how references to memory addresses and linkage symbols are handled.
• --codemodel [small|medium|large|kernel] Define how code is organized and accessed at machine code level.

Special flags:

• -llinker Transform the compiler into the LLVM linker.
• -llinker-flavor Specify the build flavor for the LLVM linker.

Useful flags:

• --debug-clang-command Displays the generated command for Clang in the phase of linking.
• --debug-gcc-commands Displays the generated command for GCC in the phase of linking.

• --clean-tokens Clean the compiler folder that holds the lexical analysis tokens.
• --clean-assembler Clean the compiler folder containing emitted assembler.
• --clean-llvm-ir Clean the compiler folder containing the emitted LLVM IR.
• --clean-llvm-bitcode Clean the compiler folder containing emitted LLVM Bitcode.
• --clean-objects Clean the compiler folder containing emitted object files.

• --no-obfuscate-archive-names Stop generating name obfuscation for each file; this does not apply to the final build.
```