<img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-logo-v1.5.png" alt= "logo" style= "width: 80%; height: 80%;"> </img>

> [!WARNING]
> The development of **The Thush Compiler** (thrushc) is incomplete and constantly changing; build the program at your own risk, as it may contain errors.

# The Thrush Compiler 

The Thrush Compiler is tasked with converting Thrush source code (`.th`) into machine code for each architecture, using Ahead Of Time (**AOT**) compilation mode, leveraging the LLVM infrastructure (**LLVM-C API**) during the process.

## Syntax 

You can see the syntax of the language properly in the repository: __https://github.com/thrushlang/syntax__

## Minimal dependencies of executables dispatched by ``thrushc``

- ``libc`` C Standard Library Interface.
- ``crt`` C Runtime.

# ¿How it works?

Currently, the only backend available for the thrush compiler to compile is the current LLVM, using the LLVM-C API. 

By default the compiler has the LLVM-C API fully embedded, in addition to multiple linkers; Avoiding any direct external dependencies.

The code generation is in 3 phases. 

- Intermediate Code Generation (``LLVM IR``).
- Emit object files (``.o``). 
- Linking with LLVM Linker (``LLD``).

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

- \>= Rust (v1.18.5) 
- Rust 2024 Edition

Now you need to compile the compiler with Rust. 

```console
cargo run -- --help
```
