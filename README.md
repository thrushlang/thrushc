<p align="center">
  <img src= "https://github.com/thrushlang/.github/blob/main/assets/thrush.png" alt= "logo" style= "width: 2hv; height: 2hv;"> </img>
</p>

> [!WARNING]  
> **The Compiler is still under development and is unfinished, please be peaceful if exists some bug.**

# The Thrush Compiler 

This compiler compile target to **LLVM IR** and coming soon to anothers backend infraestructures; This compile for native **Optimized Machine Code** with object files or executables.

## Build Dependencies for the Compiler 

**Important Crates:**

- **llvm-sys** (v170)
- **inkwell** (v0.50)
  
## External Requirements for create executables with the Compiler

- **Clang** 17.0.6 (Linux Binaries)

~ **NOTE:** In the theoretically toolchain of thrush (under development), is ready contain the toolchain for each operating system in than language is available. The default Installed location gived by the package manager `throium` in `%HOMEUSER%/thrushlang/backend/llvm/`. This process is going to be automatized by `throium`.
