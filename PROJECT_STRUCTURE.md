<img src= "https://github.com/thrustlang/.github/blob/main/assets/logos/thrustlang-logo-name.png" alt= "logo" style= "width: 80%; height: 80%;"></img>

# Thrush Compiler - Compiler Architecture

<img src= "https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt= "standard-separator" style= "width: 1hv;"> </img>

`thrustc` is a modular compiler for the **Thrust** Programming Language — a general-purpose, statically-typed systems programming language focused on writing verbose, accurate, and fast code.

The frontend uses a **handwritten recursive descent parser**. The backend performs code generation using the **LLVM C API** (via `llvm-sys` and `inkwell`) with custom abstractions and low-level tricks for access to LLVM C++ API indirectly.

## Crate Structure (`thrustc_*`)

### Core Infrastructure

- **`thrustc`**  
  Main binary entry point. Handles command-line arguments and orchestrates the entire compilation process.
- **`thrustc_core`**  
  Central driver of the compiler. Manages the compilation pipeline, emitters, printers (AST, LLVM IR, tokens, assembler), and lifecycle stages (starter, cleaner, finisher).
- **`thrustc_cli`**  
  Command-line interface helpers and argument parsing utilities shared by the `thrustc` binary. Provides structured access to flags, subcommands, and help text generation.
- **`thrustc_options`**  
  Compiler configuration and command-line options (backends, optimization levels, debug information, linkage, target settings, etc.).
- **`thrustc_diagnostician`**  
  Rich diagnostic and error reporting system with source positions and pretty-printed messages.
- **`thrustc_errors`**  
  Internal error types and utilities.
- **`thrustc_logging`**  
  Structured logging for compiler internals.
- **`thrustc_utils`**  
  General shared utilities used across crates.

### Frontend

- **`thrustc_lexer`**  
  Handwritten lexer supporting identifiers, numbers, strings, characters, and language-specific rules.
- **`thrustc_preprocessor`**  
  Preprocessor for modules, imports, and early processing of source code.
- **`thrustc_preprocessor_type_resolver`**  
  Early type resolution during the preprocessing phase.
- **`thrustc_parser`**  
  **Handwritten recursive descent parser** with layered precedence climbing. Parses declarations, expressions, statements, control flow, attributes (`@...`), imports, and more.
- **`thrustc_ast`**  
  Abstract Syntax Tree definitions, node types, visitor traits, metadata, and language builtins.
- **`thrustc_ast_verifier`**  
  Structural and consistency verification of the AST.
- **`thrustc_span`**  
  Source span and location tracking used throughout the compiler.
- **`thrustc_token`** & **`thrustc_token_type`**  
  Token definitions and supporting traits.
- **`thrustc_reader`**  
  Source file reading and input management.
- **`thrustc_parser_context`**  
  Context state maintained by the parser during recursive descent, including current scope, pending declarations, and parse-time flags.
- **`thrustc_parser_table`**  
  Symbol and declaration tables consumed by the parser, providing fast lookups for identifiers, types, and built-in constructs during parsing.
- **`thrustc_parser_external_table`**  
  External-facing table interface exposed to other crates that need read access to parser-built symbol data without depending on the full parser.
- **`thrustc_ast_external`**  
  Thin re-export layer that exposes selected AST types to external consumers (preprocessor, codegen, tools) without creating circular dependencies on `thrustc_ast`.

### Semantic Analysis

- **`thrustc_scoper`**  
  Scope analysis.
- **`thrustc_analyzer`**  
  General Analysis.
- **`thrustc_typechecker`**  
  Main type checker with type inference for expressions, function calls, operations, and globals.
- **`thrustc_typesystem`**  
  Complete type system: arrays, fixed arrays, pointers, structures, function references, casting, inference, and modifiers.
- **`thrustc_semantic`**  
  General semantic analysis layer.
- **`thrustc_entities`**  
  Shared entities used by the analyzer, parser, typechecker, and linter.
- **`thrustc_linter`**  
  Static linter for style and best-practice warnings.
- **`thrustc_attributes`**  
  They serve as direct wrappers to LLVM attributes that modify code generation behavior.
- **`thrustc_attribute_checker`**  
  Validation that attributes are correctly applied to functions, types, variables, and other items.
- **`thrustc_modificators`**  
  Handling of language modifiers (visibility, mutability, etc.).

### LLVM Backend & IR

- **`thrustc_llvm_codegen`**  
  **Primary code generation backend**. Uses the **LLVM C API** directly with custom wrappers and tricks.  
  Supports expressions, statements, globals, functions, intrinsics, inline assembly, heap/stack management, JIT, and optimizations.
- **`thrustc_llvm_attributes`**  
  Mapping and emission of LLVM-specific attributes.
- **`thrustc_llvm_builtins`**  
  Builtins and intrinsics integration with LLVM.
- **`thrustc_llvm_callconventions`** & **`thrustc_llvm_callconventions_checker`**  
  Support and validation of calling conventions.
- **`thrustc_llvm_intrinsic_checker`**  
  Validation of LLVM intrinsic usage.
- **`thrustc_mir`**  
  Mid-level Intermediate Representation (optional, after type checking).
- **`thrustc_constants`**  
  Compile-time constant evaluation.

### Compiler Control

- **`thrustc_frontend_abort`**  
  Controlled handling of unrecoverable frontend errors.
- **`thrustc_heap_allocator`**  
  Heap allocation logic for support the compiler.

---

## Compiler Pipeline

1. **Lexer** 
2. **Preprocessor** (and associated crates) 
3. **Parser** (recursive descent)
4. **Semantic Analysis**:
   - Scoper
   - AST Verifier
   - Type Checker
   - Analyzer
   - Attribute Checker
   - Linter (only if no errors from previous stages)
5. **MIR** (optional)
6. **LLVM Codegen** (LLVM C API + custom abstractions)
7. **Optimization** → Object file / LLVM IR / Assembler / JIT emission

---