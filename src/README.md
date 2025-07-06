<p align="center">
  <img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-v1.6.png" alt= "logo" style= "width: 2hv; height: 2hv;"> </img>
</p>

## Thrush Programming Language | Compiler Source Code Tree

This folder contains everything that the Thrush compiler represents, from its code generators, bounds checking processes, lexical analysis, abstract tree representation construction, optimizers, optimization pass handler, command line, diagnostics, and much more.

### Compiler Organization

- ``backend/`` It contains code generators for the compiler that facilitate the use of GCC or LLVM infrastructure via official wrapper APIs. It also mentions the inclusion of C compilers within the compiler, which serve as a bridge to the nearest linker. A direct linker driver is expected to be developed in the future.

- ``core/`` Contains everything related to compiler control, command line, abstraction for code generators, and structures that represent the Thrush compiler from a high-level view.

- ``frontend/`` It contains everything related to lexical analysis, parsing, and semantic analysis of the language. It should be noted that this language has its own implementation of its parser and lexer from scratch; commercials such as Larpop or ANTLR are not used.
