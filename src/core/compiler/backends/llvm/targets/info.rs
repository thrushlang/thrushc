use std::process;

use crate::core::{compiler::backends::llvm::targets, console::logging};

#[inline]
pub fn print_all_targets() {
    logging::write(
        logging::OutputIn::Stdout,
        "aarch64     - AArch64 (little endian)
aarch64_32  - AArch64 (little endian ILP32)
aarch64_be  - AArch64 (big endian)
amdgcn      - AMD GCN GPUs
arm         - ARM
arm64       - ARM64 (little endian)
arm64_32    - ARM64 (little endian ILP32)
armeb       - ARM (big endian)
bpf         - BPF (host endian)
bpfeb       - BPF (big endian)
bpfel       - BPF (little endian)
hexagon     - Hexagon
lanai       - Lanai
loongarch32 - 32-bit LoongArch
loongarch64 - 64-bit LoongArch
mips        - MIPS (32-bit big endian)
mips64      - MIPS (64-bit big endian)
mips64el    - MIPS (64-bit little endian)
mipsel      - MIPS (32-bit little endian)
msp430      - MSP430 [experimental]
nvptx       - NVIDIA PTX 32-bit
nvptx64     - NVIDIA PTX 64-bit
ppc32       - PowerPC 32
ppc32le     - PowerPC 32 LE
ppc64       - PowerPC 64
ppc64le     - PowerPC 64 LE
r600        - AMD GPUs HD2XXX-HD6XXX
riscv32     - 32-bit RISC-V
riscv64     - 64-bit RISC-V
sparc       - Sparc
sparcel     - Sparc LE
sparcv9     - Sparc V9
spirv       - SPIR-V Logical
spirv32     - SPIR-V 32-bit
spirv64     - SPIR-V 64-bit
systemz     - SystemZ
wasm32      - WebAssembly 32-bit
wasm64      - WebAssembly 64-bit
x86         - 32-bit X86: Pentium-Pro and above
x86-64      - 64-bit X86: EM64T and AMD64
xcore       - XCore",
    );
}

#[inline]
pub fn print_specific_cpu_support(arch: &str) {
    match arch {
        "aarch64" | "aarch64_32" | "aarch64_be" => {
            targets::aarch64::print_all_supported_cpus();
        }

        "arm" | "arm64" | "arm64_32" | "armeb" => {
            targets::arm::print_all_supported_cpus();
        }

        "amdgcn" | "r600" => {
            targets::amdgpu::print_all_supported_cpus();
        }

        "bpf" | "bpfeb" | "bpfel" => {
            targets::bpf::print_all_supported_cpus();
        }

        "loongarch32" | "loongarch64" => {
            targets::loongarch::print_all_supported_cpus();
        }

        "mips" | "mips64" | "mips64el" | "mipsel" => {
            targets::mips::print_all_supported_cpus();
        }

        "ppc32" | "ppc32le" | "ppc64" | "ppc64le" => {
            targets::powerpc::print_all_supported_cpus();
        }

        "riscv32" | "riscv64" => {
            targets::riscv::print_all_supported_cpus();
        }

        "sparc" | "sparcel" | "sparcv9" => {
            targets::sparc::print_all_supported_cpus();
        }

        "wasm32" | "wasm64" => {
            targets::wasm::print_all_supported_cpus();
        }

        "hexagon" => {
            targets::hexagon::print_all_supported_cpus();
        }

        "msp430" => {
            targets::msp430::print_all_supported_cpus();
        }

        "nvptx" | "nvptx64" => {
            targets::nvptx::print_all_supported_cpus();
        }

        "systemz" => {
            targets::systemz::print_all_supported_cpus();
        }

        "x86" | "x86_64" => {
            targets::x86_64::print_all_supported_cpus();
        }

        what => {
            if what.is_empty() {
                logging::print_error(
                    logging::LoggingType::Error,
                    "Try to set the target architecture using the '-target' command line flag.",
                );

                process::exit(1)
            }

            logging::print_error(
                logging::LoggingType::Error,
                &format!("CPU support isn't available for '{}' architecture.", what),
            );

            process::exit(1)
        }
    }

    process::exit(0)
}
