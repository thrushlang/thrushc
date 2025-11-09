<img src= "https://github.com/thrushlang/.github/blob/main/assets/logos/thrushlang-logo.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

## Thrush Programming Language | TCSCT (Thrush Compiler Source Code Tree)

This folder contains everything that the Thrush compiler represents, from its code generators, bounds checking processes, lexical analysis, abstract tree representation construction, optimizers, optimization pass handler, command line, diagnostics, and much more.

### Compiler Organization

- ``linkage/`` It generally contains a compiler to be used as a bridge to the nearest linker, or it can be used to directly transform the compiler into a linker.

- ``back_end/`` It contains code generators for the compiler, facilitating the use of the GCC or LLVM infrastructure. It also mentions the inclusion of C compilers in the compiler, which serve as a bridge to the nearest linker.

- ``middle_end/`` It contains standard high-level optimizations in compilers, corresponding to each type of compiler frontend type.
 
- ``core/`` Contains everything related to compiler control, command line, abstraction for code generators, and structures that represent the Thrush compiler from a high-level view.

- ``front_end/`` It generally contain information related to lexical analysis, syntactic analysis, and semantic analysis of the language for that specific model. It is worth noting that this language has its own implementation of its parser and lexer; commercial programs like LarPOP or ANTLR are not used.

