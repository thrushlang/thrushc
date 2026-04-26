<img src="https://github.com/thrustlang/.github/blob/main/assets/logos/thrustlang-logo-name.png" alt="logo" style="width: 80%; height: 80%;">

# Thrust Compiler - Compiler Architecture

<img src="https://github.com/thrustlang/.github/blob/main/assets/standard-text-separator.png" alt="standard-separator" style="width: 1hv;">

`thrustc` is a modular compiler for the **Thrust** Programming Language — a general-purpose, statically-typed systems programming language focused on writing verbose, accurate, and fast code.

The frontend uses a **handwritten recursive descent parser**. The backend performs code generation using the **LLVM C API** (via `llvm-sys` and `inkwell`) with custom abstractions and low-level tricks for access to LLVM C++ API indirectly.

## Crate Structure (`thrustc_*`)

### Core Infrastructure

- **`thrustc`**  
  Main binary entry point. Handles command-line arguments and orchestrates the entire compilation process.

- **`thrustc_core`**  
  Central driver of the compiler. Manages the compilation pipeline, emitters, printers (AST, LLVM IR, tokens, assembler), and lifecycle stages (`starter`, `cleaner`, `finisher`, `validate`).

- **`thrustc_cli`**  
  Command-line interface helpers and argument parsing utilities.

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

- **`thrustc_parser_context`**  
  Context state maintained by the parser during recursive descent.

- **`thrustc_parser_table`** & **`thrustc_parser_external_table`**  
  Symbol and declaration tables for fast lookups during parsing and external access.

- **`thrustc_ast`**  
  Abstract Syntax Tree definitions, node types, visitor traits, metadata, and language builtins.

- **`thrustc_ast_external`**  
  Thin re-export layer that exposes selected AST types to other crates without circular dependencies.

- **`thrustc_ast_verifier`**  
  Structural and consistency verification of the AST.

- **`thrustc_span`**  
  Source span and location tracking used throughout the compiler.

- **`thrustc_token`** & **`thrustc_token_type`**  
  Token definitions and supporting traits.

- **`thrustc_reader`**  
  Source file reading and input management.

### Semantic Analysis & Middle-end

- **`thrustc_scoper`**  
  Scope analysis and resolution.

- **`thrustc_analyzer`**  
  General static analysis.

- **`thrustc_typechecker`**  
  Main type checker with type inference for expressions, function calls, operations, and globals.

- **`thrustc_typesystem`**  
  Complete type system: arrays, fixed arrays, pointers, structures, function references, casting, inference, layout, and modifiers.

- **`thrustc_semantic`**  
  General semantic analysis layer.

- **`thrustc_entities`**  
  Shared entities used by analyzer, parser, typechecker, and linter.

- **`thrustc_linter`**  
  Static linter for style and best-practice warnings.

- **`thrustc_attributes`** & **`thrustc_attribute_checker`**  
  Handling and validation of language and LLVM attributes.

- **`thrustc_modificators`**  
  Handling of language modifiers (visibility, mutability, etc.).

- **`thrustc_mir`**  
  Mid-level Intermediate Representation (atomic operations, thread mode, etc.).

### LLVM Backend

- **`thrustc_llvm_codegen`**  
  **Primary code generation backend**. Uses the **LLVM C API** directly with custom wrappers.  
  Supports expressions, statements, globals, functions, intrinsics, inline assembly, heap/stack, JIT, and optimizations.

- **`thrustc_llvm_target_triple`**  
  Intelligent wrapper around LLVM target triples with architecture queries (`supports_f80`, `supports_ppc128`, `is_64_bit`, etc.).

- **`thrustc_llvm_attributes`**  
  Mapping and emission of LLVM-specific attributes.

- **`thrustc_llvm_callconventions`** & **`thrustc_llvm_callconventions_checker`**  
  Support and validation of calling conventions.

- **`thrustc_llvm_intrinsic_checker`**  
  Validation of LLVM intrinsic usage.

- **`thrustc_llvm_abi`** & **`thrustc_llvm_abi_x86`**  
  ABI-specific handling (especially x86-64).

- **`thrustc_backends`**  
  Backend abstraction layer (currently focused on LLVM).

### Compiler Control & Support

- **`thrustc_frontend_abort`**  
  Controlled handling of unrecoverable frontend errors.

- **`thrustc_heap_allocator`**  
  Custom heap allocation logic used by the compiler itself.

---

## Compiler Pipeline

1. **Lexer**
2. **Preprocessor** (and associated crates)
3. **Parser** (recursive descent + tables)
4. **Semantic Analysis**:
   - Scoper
   - AST Verifier
   - Type Checker + Type System
   - Analyzer & Linter
   - Attribute Checker
5. **MIR** (optional)
6. **LLVM Codegen** (LLVM C API + custom abstractions)
7. **Optimization** → Object file / LLVM IR / Assembler / JIT emission

---