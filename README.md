<img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-logo-v1.5.png" alt= "logo" style= "width: 80%; height: 80%;"> </img>

> [!WARNING]
> The development of The **Thrush Compiler** (thrushc) is incomplete; build the program at your own risk, as it may contain errors.

# The Thrush Compiler 

The **Thrush Compiler** efficiently transfers source code from Thrush files directly to the intended target. Beyond this, it serves as a flexible bridge, integrating with diverse code generators for both research and development purposes.

### Target Architectures

> [!WARNING]
> This doesn't mean it has the ability to compile 100% for every architecture, since compilation depends on whether the host system has certain tools required to output compiled code for that architecture. However, the assembler should theoretically output without problems.

This represents all possible combinations of triple targets or targets, which the compiler can compile with the backend it has available.

### LLVM Targets

The compiler supports all triple targets, in addition to the architectures supported by the entire LLVM-C API.

- ``x86_64``
- ``AArch64``
- ``RISC-V``
- ``ARM``
- ``MIPS``
- ``PowerPC``
- ``SystemZ``
- ``AMDGPU``
- ``Hexagon``
- ``Lanai``
- ``LoongArch``
- ``MSP430``
- ``NVPTX``
- ``SPARC``
- ``XCore``
- ``BPF``
- ``SPIR-V``
- ``WebAssembly``

## Syntax 

You can see the syntax of the language properly in the repository: __https://github.com/thrushlang/syntax__

# ¿How it works?

Currently, the only backend available for the thrush compiler to compile is the current LLVM, using the LLVM-C API. 

#### Embedded Clang

The compiler has Clang compiled for Linux & Windows inside the executable in case the programmer does not have access to it; however, you can specify a custom Clang & GCC.

#### Code Generations Phases

The code generation is in 3 phases. 

- Intermediate Code Generation (``LLVM IR``).
- Emit object files (``.o``). 
- Linking with some linker through the ``Clang`` or ``GCC`` C compilers. ~ *Rust 2015 be like*

In summary:

<p align="center">
  <img src= "https://github.com/thrushlang/thrushc/blob/master/assets/how%20it%20works%20(thrushc)%20v1.3.png" style= "width: 1hv; height: 1hv;"> </img>
</p>

# Getting Started

## Start

You must first clone the repository and access it locally. 

```console
git clone --depth 1 https://github.com/thrushlang/thrushc && cd thrushc
```

## Build dependencies 

Among the dependencies required by the compiler is the LLVM-C API, which you can find pre-compiled for each operating system at **[Thrush Programming Language Toolchains](https://github.com/thrushlang/toolchains)**.

Automatically:

```console
cd builder
cargo run
```

## Build the Compiler

Now you need to have Rust installed with a recent version.

- \>= **[Rust](https://www.rust-lang.org/)** (v1.18.5) 
- **[Rust](https://www.rust-lang.org/)** 2024 Edition

Now you need to compile the compiler with Rust. 

```console
cargo run -- --help
```
