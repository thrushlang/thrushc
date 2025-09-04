<p align="center">
  <img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-v1.6.png" alt= "logo" style= "width: 2hv; height: 2hv;"> </img>
</p>

## Thrush Programming Language | Compiler Source Code Tree

This folder contains everything that the Thrush compiler represents, from its code generators, bounds checking processes, lexical analysis, abstract tree representation construction, optimizers, optimization pass handler, command line, diagnostics, and much more.

### Compiler Organization

- ``backends/`` It contains code generators for each branch of quantum and classical computing for the compiler, facilitating the use of the GCC or LLVM infrastructure. It also mentions the inclusion of C compilers in the compiler, which serve as a bridge to the nearest linker.

- ``core/`` Contains everything related to compiler control, command line, abstraction for code generators, and structures that represent the Thrush compiler from a high-level view.

- ``frontends/`` It contains all frontends of the programming language with its branches to quantum and classical computing. They generally contain information related to lexical analysis, syntactic analysis, and semantic analysis of the language for that specific model. It is worth noting that this language has its own implementation of its parser and lexer; commercial programs like LarPOP or ANTLR are not used.
