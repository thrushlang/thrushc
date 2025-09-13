<img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-logo-v1.5.png" alt= "logo" style= "width: 80%; height: 80%;"> </img>

# The Thrush Compiler 

The **Thrush Compiler** efficiently transfers source code from Thrush files directly to the intended target. Beyond this, it serves as a flexible bridge, integrating with diverse code generators for both research and development purposes.

> [!WARNING]  
> The compiler is in an early development phase. It may contain bugs when testing certain syntax. Continue on your own.

# Getting Started

## Start

You must first clone the repository and access it locally. 

```console
git clone --depth 1 https://github.com/thrushlang/thrushc && cd thrushc
```

## Build dependencies 

Among the dependencies required by the compiler is the LLVM-C API, which you can find pre-compiled for each operating system at **[Thrush Programming Language - Toolchains](https://github.com/thrushlang/toolchains)**.

Automatically:

```console
cd builder && cargo run && cd ..
```

## Build the Compiler

Now you need to have Rust installed with a recent version.

- \>= **[Rust](https://www.rust-lang.org/)** (v1.18.5) 
- Rust 2024 Edition

Now you need to compile the compiler with Rust. 

```console
cargo run -- --help
```

If you just need to quickly see the commands, you can look at **[Thrush Compiler - Commands & Flags](https://github.com/thrushlang/thrushc/blob/master/COMMANDS.md)**.

# Code Generators 

Code generators are generally backend compilers that accept the generation of intermediate code through an interface, which can then be used to transfer it to machine-specific assembler or directly to machine code for execution.

## LLVM

The LLVM backend infrastructure is the default code generator for the **[Thrush Programming Language](https://github.com/thrushlang/)**. It offers full scope and portability across many architectures or targets.

### LLVM Version

- ``17.0.6``
 
#### Why this specific version of LLVM for the compiler?

Between version 16-17, the introduction to the change of typed pointers was made, which are now almost a standard in the backend. 

Some programming languages like Swift tend to use versions lower than 16 of LLVM, for reasons of compatibility with code generation that differs between higher and lower versions of LLVM, and version 16 offers legacy support for languages that need it.

We only need support for C and nothing else. We are not interested in FFI with C++ for the moment, nor in mangling with it either.
17 is enough and from there on.

#### LLVM Targets

Beyond the standard triple targets, the compiler also supports all architectures available through the **[LLVM](https://llvm.org)**-C API. These include:

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

## GCC

The GCC compiler backend is still under construction.

In the future, you will be able to use it with the ``-gcc`` flag to use the GCC backend code generator instead of the default LLVM one.

However, it is only available on **Linux**.

You must also have ``libgccjit.so`` dynamically installed in your distribution so that the compiler doesn't get scared at runtime when using GCC.

### GCC backend installation

The GCC backend, which is completely embeddable, of the JIT compiler type, can practically only be built dynamically and not statically. For this reason, it has been distributed in many package managers of Linux distributions.

If you need help finding a way to install libgccjit on your system, you can check: [GCC JIT - Documentation](https://gcc.gnu.org/onlinedocs/jit/internals/index.html#working-on-the-jit-library)

### Fedora 

```console
sudo dnf install libgccjit-devel
```

### Arch

```console
sudo pacman -S libgccjit
```

### Debian

```console
sudo apt install libgccjit-0-dev
```

### Notes

Currently, the very same Rust is using ``libgccjit`` as a library for an AOT backend prototype for Rust. Called ``rustc_codegen_gcc``. Thrush will integrate it in his own way for use in the language.

For more information: [Rust - GCC AOT Code Generation](https://github.com/rust-lang/rustc_codegen_gcc)

## QIR

Unlike GCC, this target is only planned and not currently under construction.

QIR is not a backend compiler as such or a code generator. Rather, it is a runtime prototyped in a version of LLVM that allows the execution of code on quantum executors available on Azure, Rigetti, Quantinuum, and much more.

The very same [Q#](https://github.com/microsoft/qsharp/tree/main/source/compiler/qsc_codegen/src) uses it as a backend to execute and emulate code behavior in a "quantum" environment (real quantum computers). This infrastructure has executors and emulators of this behavior, written entirely in Rust, which will allow Thrush to generate code similar to [Q#](https://github.com/microsoft/qsharp/tree/main/source/compiler/qsc_codegen/src) during its execution.

Projects like the **[bytecode-runner](https://github.com/qir-alliance/qir-runner)** from **[QIR Alliance](https://github.com/qir-alliance)**.

It's a titanic job because it requires rewriting a good part of Thrush frontend, practically creating a new one and integrating it with that frontend with new rules.

## Syntax 

The language syntax is under construction at the same time as the compiler. It may be outdated compared to the compiler, as the latter progresses more rapidly. This will be normalized once a valid and sufficiently stable beta is released.

**[Thrush Programming Language - General Syntax](https://github.com/thrushlang/syntax)**

## ¿How it works?

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

# Frequent Questions

#### > Why isn't the compiler designed to use it as a bootstrap compiler?

Regarding the concept of bootstrapping in compilers (For more information: https://www.bootstrappable.org/).

The decision was made to fully implement all the programming language functions in the compiler written in Rust, because it proposes a development approach similar to what Gleam Team did for Gleam Programming Language, and also to lighten the workload, given that we are already using LLVM.
