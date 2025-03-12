<p align="center">
  <img src= "https://github.com/thrushlang/thrushc/blob/master/assets/thrushlang-v1.0.png" alt= "logo" style= "width: 2hv; height: 2hv;"> </img>
</p>

> [!WARNING]  
> **The compiler is still under development and is unfinished, please be peaceful if exists some bug.**

# The Thrush Compiler 

The compiler is responsible for translating each code effectively into LLVM intermediate code so that LLVM effectively compiles each scenario to the indicated architecture. LLVM is the main code generator for thrushc; it is speculated that in the future, GCC will be used.

## Build dependencies for the compiler 

**Important Rust crates:**

- **llvm-sys** (v170)
- **inkwell** (v0.50)
  
## Requirements for create executables with the compiler

### Linux

- [LLVM & Clang Toolchain (x64)](https://github.com/thrushlang/toolchains/releases/download/Toolchains/thrushlang-toolchain-linux-x64-v1.0.0.tar.gz)

> [!NOTE]  
> The theoretically toolchain of thrush (under development), is ready contain the toolchain for each operating system in than language is available. The default Installed location gived by the package manager `thorium` in `%HOMEUSER%/thrushlang/backend/llvm/`. This process is going to be automatized by `thorium`.

