<p align="center">
  <img src= "https://github.com/thrushlang/.github/blob/main/assets/Thrush.png" alt= "logo" style= "width: 2hv; height: 2hv;"> </img>
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

- **Clang** && **LLVM** 17.0.2 (Linux Binaries with included statically linked libraries)

~ **NOTE:** In the theoretically toolchain of thrush (under development), is ready contain the toolchain for each Operating System in than language is available. The Default Installed location gived by the package manager `throium` in `%HOMEUSER%/thrushlang/backend/llvm/`. This process is going to be automatized by `throium`.
