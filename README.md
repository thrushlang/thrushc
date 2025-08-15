<img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-logo-v1.5.png" alt= "logo" style= "width: 80%; height: 80%;"> </img>

> [!NOTE]
> # TEMPORARY ONE-MONTH HIATUS
>
> We have reached a point where we are faced with higher obligations that somewhat set this project aside.
> Therefore, from ``8/5/25`` to ``9/6/25``, there will be a HIATUS in the development of the compiler and its plans moving forward.
> ## What does it mean?
>
> Essentially, not much, just that `thrushc` won't be reflected with commits regularly or habitually.
> The other projects or repositories of the organization will continue with the planned development pace, but with a little less frequency.
>
> If you'd like to be part of the team, please contact us through the organization's social media channels. They will continue as usual.
> 
> Without anything else to say, *Kevin Benavides*, see you next month.

> [!WARNING]
> The development of The **Thrush Compiler** (thrushc) is incomplete; build the program at your own risk, as it may contain errors.

# The Thrush Compiler 

The **Thrush Compiler** efficiently transfers source code from Thrush files directly to the intended target. Beyond this, it serves as a flexible bridge, integrating with diverse code generators for both research and development purposes.

### Target Architectures

> [!WARNING]
> While the compiler itself can theoretically generate code for these architectures, successful compilation into executable files (like .o files or final binaries) ultimately depends on your host system having the necessary toolchain components (e.g., assemblers, linkers, system libraries) for that specific target. The assembler should, however, produce output without issues.

### LLVM

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
cd builder && cargo run
```

## Build the Compiler

Now you need to have Rust installed with a recent version.

- \>= **[Rust](https://www.rust-lang.org/)** (v1.18.5) 
- Rust 2024 Edition

Now you need to compile the compiler with Rust. 

```console
cargo run -- --help
```

# Frequent Questions

#### > Why isn't the compiler designed to use it as a bootstrap compiler?

Regarding the concept of bootstrapping in compilers (For more information: https://www.bootstrappable.org/).

The decision was made to fully implement all the programming language functions in the compiler written in Rust, because it proposes a development approach similar to what Gleam Team did for Gleam Programming Language, and also to lighten the workload, given that we are already using LLVM.
