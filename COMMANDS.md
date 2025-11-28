<img src= "https://github.com/thrushlang/.github/blob/main/assets/logos/thrushlang-logo.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

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

• llvm-print-targets Show the current LLVM target supported.
• llvm-print-supported-cpus Show the current LLVM supported CPUs for the current LLVM target.
• llvm-print-host-target-triple Show the host LLVM target triple.
• llvm-print-opt-passes Show all available optimization passes through '--opt-passes=p{passname, passname}' in the compiler for the LLVM backend.

General flags:

• -build-dir Configure the build directory for the AOT compiler.

Linkage flags:

• -clang-link [/usr/bin/clang] Specifies the path for use of an external Clang for linking purpose.
• -gcc-link [usr/bin/gcc] Specifies GNU Compiler Collection (GCC) for linking purpose.
• -start Marks the start of arguments to the active external or built-in linking compiler.
• -end Marks the end of arguments to the active external or built-in linker compiler.

Compiler flags:

• -llvm-backend Enable the usage of the LLVM backend infrastructure.
• -target [x86_64] Set the LLVM target arquitecture.
• -target-triple [x86_64-pc-linux-gnu] Set the LLVM backend target triple. For more information, see 'https://clang.llvm.org/docs/CrossCompilation.html'.
• -cpu [haswell] Specify in LLVM the CPU to optimize.
• -cpu-features [+sse2,+cx16,+sahf,-tbm] Specify in LLVM the new features of the CPU to use.
• -emit [llvm-bc|llvm-ir|asm|raw-llvm-ir|raw-llvm-bc|raw-asm|obj|ast|tokens] Compile the code into specified representation.
• -print [llvm-ir|raw-llvm-ir|tokens] Displays the final compilation on standard output.
• -opt [O0|O1|O2|mcqueen] Optimization level.

JIT Compiler flags:

• -jit Enable the use of the JIT Compiler for code execution.
• -jit-libc Specify the C runtime to link for code execution via the JIT Compiler.
• -jit-link Specify, add, and link an external dynamic library for code execution via the JIT Compiler.

Extra compiler flags:

• --opt-passes [-p{passname,passname}] Pass a list of custom optimization passes to the LLVM backend. For more information, see: 'https://releases.llvm.org/17.0.1/docs/CommandGuide/opt.html#cmdoption-opt-passname'.
• --modificator-passes [loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions;callgraphprofile;forgetallscevinloopunroll;licmmssaaccpromcap=0;licmmssaoptcap=0;] Pass a list of custom modificator optimization passes to the LLVM backend.
• --reloc [static|pic|dynamic] Indicate how references to memory addresses and linkage symbols are handled.
• --codemodel [small|medium|large|kernel] Define how code is organized and accessed at machine code level.
• --omit-frame-pointer Regardless of the optimization level, it omits the emission of the frame pointer.
• --omit-uwtable It omits the unwind table required for exception handling and stack tracing.
• --disable-default-opt It disable default optimization that occurs even without specified optimization.
• --enable-ansi-color It allows ANSI color formatting in compiler diagnostics.

Useful flags:

• --debug-clang-command Displays the generated command for Clang in the phase of linking.
• --debug-gcc-commands Displays the generated command for GCC in the phase of linking.

• --clean-build Clean the compiler build folder that holds everything.
• --clean-tokens Clean the compiler folder that holds the lexical analysis tokens.
• --clean-assembler Clean the compiler folder containing emitted assembler.
• --clean-llvm-ir Clean the compiler folder containing the emitted LLVM IR.
• --clean-llvm-bitcode Clean the compiler folder containing emitted LLVM Bitcode.
• --clean-objects Clean the compiler folder containing emitted object files.

• --no-obfuscate-archive-names Stop generating name obfuscation for each file; this does not apply to the final build.
• --no-obfuscate-ir Stop generating name obfuscation in the emitted IR code.
```