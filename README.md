<p align="center">
  <img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-v1.1.png" alt= "logo" style= "width: 2hv; height: 2hv;"> </img>
</p>

> [!WARNING]  
> **The compiler is still under development and is unfinished, please be peaceful if exists some bug.**

# The Thrush Compiler 

The Thrush Compiler is tasked with converting Thrush source code (`.th`) into native code for each architecture, using either Just In Time (**JIT**) or Ahead Of Time (**AOT**) compilation modes, leveraging the LLVM infrastructure (**LLVM-C API**) during the process.

# ¿How it works?

Currently, the only backend available for the thrush compiler to compile is the current LLVM, using the LLVM-C API. The process consists of three parts:

1. Compilation by thrushc to LLVM bitcode (*.bc).
2. Optimization by the LLVM optimization tool (opt & llvm-lto).
3. Final compilation by clang to the target.

In summary:

<p align="center">
  <img src= "https://github.com/thrushlang/thrushc/blob/master/assets/how%20it%20works%20(with%20llvm%20backend).png" style= "width: 1hv; height: 1hv;"> </img>
</p>

# Getting Started

## Build dependencies 

Among the dependencies required by the compiler is the LLVM-C API, which you can find pre-compiled for each operating system at **[Thrush Programming Language Toolchains](https://github.com/thrushlang/toolchains)**.

Automatically:

```console
python build.py <target-operating-system>
```

## Build the Compiler

Now you need to compile the compiler with Rust.

```console
cargo run -- --help
```

## Install a toolchain

To create binaries with the compiler, you need the pre-compiled toolchain for each operating system. Find your operating system and architecture.

- **[Thrush Programming Language Toolchains](https://github.com/thrushlang/toolchains)**.

> [!NOTE]  
> The language will contain a **pre-optimized** toolchain repository for each operating system. This process automates the installation of the language and its entire ecosystem through the **Thorium** package manager.

