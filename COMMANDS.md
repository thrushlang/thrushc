<img src= "https://github.com/thrushlang/.github/blob/main/assets/logos/thrushlang-logo.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

# The Thrush Compiler | Commands & Flags

A list of the commands supported by the Thrush Compiler command line.

> [!WARNING]  
> This might be a bit outdated, it could be information that's somewhat distant from the changes.

```console
The Thrush Compiler

Usage: thrushc [-flags|--flags] [files..]

General Commands:

• -h, --help optional[opt|emit|print|code-model|reloc-model] Show help message.
• -v, --version, version Show the version.

General flags:

• -build-dir Specifies the compiler artifacts directory.

Linkage flags:

• -clang-link [/usr/bin/clang] Specifies the path for use of an external Clang for linking purpose.
• -gcc-link [usr/bin/gcc] Specifies GNU Compiler Collection (GCC) for linking purpose.
• -start Marks the start of arguments to the active external or built-in linking compiler.
• -end Marks the end of arguments to the active external or built-in linker compiler.

Compiler flags:

• -target [x86_64] Set the target arquitecture.
• -target-triple [x86_64-pc-linux-gnu|x86_64-pc-windows-msvc] Set the target triple. For more information, see 'https://clang.llvm.org/docs/CrossCompilation.html'.
• -cpu [haswell|alderlake|ivybridge|pentium|pantherlake] Specify the CPU to optimize.
• -cpu-features [+sse2,+cx16,+sahf,-tbm] Specify the new features of the CPU to use.
• -emit [llvm-bc|llvm-ir|asm|unopt-llvm-ir|unopt-llvm-bc|unopt-asm|obj|ast|tokens] Compile the code into specified representation.
• -print [llvm-ir|unopt-llvm-ir|asm|unopt-asm|tokens] Displays the final compilation on standard output.
• -opt [O0|O1|O2|O3|Os|Oz] Optimization level.
• -dbg Enable generation of debug information (DWARF).
• -dbg-for-inlining Enable debug information specifically optimized for inlined functions.
• -dbg-for-profiling Emit extra debug info to support source-level profiling tools.

JIT Compiler flags:

• -jit Enable the use of the JIT Compiler for code execution.
• -jit-libc Specify the C runtime to link for code execution via the JIT Compiler.
• -jit-link Specify, add, and link an external dynamic library for code execution via the JIT Compiler.

Extra compiler flags:

• --opt-passes [-p{passname,passname}] Pass a list of custom optimization passes. For more information, see: 'https://releases.llvm.org/17.0.1/docs/CommandGuide/opt.html#cmdoption-opt-passname'.
• --modificator-passes [loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions;callgraphprofile;forgetallscevinloopunroll;licmmssaaccpromcap=0;licmmssaoptcap=0;] Pass a list of custom modificator optimization passes.
• --reloc-model [static|pic|dynamic] Indicate how references to memory addresses and linkage symbols are handled.
• --code-model [small|medium|large|kernel] Define how code is organized and accessed at machine code level.
• --target-triple-darwin-variant [arm64-apple-ios15.0-macabi] Specify the darwin target variant triple.
• --macos-version [15.0.0] Specify the MacOS SDK version.
• --ios-version [17.4.0] Specify the iOS SDK version.
• --enable-ansi-color It allows ANSI color formatting in compiler diagnostics.

Omission compiler flags:

• --omit-frame-pointer Regardless of the optimization level, it omits the emission of the frame pointer.
• --omit-uwtable It omits the unwind table required for exception handling and stack tracing.
• --omit-direct-access-external-data It omits direct access to external data references, forcing all external data loads to be performed indirectly via the Global Offset Table (GOT).
• --omit-rtlib-got It omits the runtime library dependency on the Global Offset Table (GOT), essential when generating non-Position Independent Code (PIC) with ARM.
• --omit-default-opt It omits default optimization that occurs even without specified optimization.

Debug compiler flags:

• --debug-clang-command Displays the generated command for Clang in the phase of linking.
• --debug-gcc-commands Displays the generated command for GCC in the phase of linking.

Useful flags:

• --export-compiler-errors Export compiler error diagnostics to files.
• --export-compiler-warnings Export compiler warning diagnostics to files.
• --export-diagnostics-path [diagnostics/] Specify the path where diagnostic files will be exported.
• --clean-exported-diagnostics Clean the exported diagnostics directory.
• --clean-build Clean the compiler build folder that holds everything.
• --clean-tokens Clean the compiler folder that holds the lexical analysis tokens.
• --clean-assembler Clean the compiler folder containing emitted assembler.
• --clean-llvm-ir Clean the compiler folder containing the emitted LLVM IR.
• --clean-llvm-bitcode Clean the compiler folder containing emitted LLVM Bitcode.
• --clean-objects Clean the compiler folder containing emitted object files.

• --no-obfuscate-archive-names Stop generating name obfuscation for each file; this does not apply to the final build.
• --no-obfuscate-ir Stop generating name obfuscation in the emitted IR code.

• --print-targets Show the current target supported.
• --print-supported-cpus Show the current supported CPUs for the current target.
• --print-host-target-triple Show the host target triple.
• --print-opt-passes Show all available optimization passes through '--opt-passes=p{passname, passname}'.
```