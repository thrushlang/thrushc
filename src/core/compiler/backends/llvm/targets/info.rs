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

#[inline]
pub fn print_all_target_triples() {
    logging::write(
        logging::OutputIn::Stdout,
        "aarch64-apple-darwin
aarch64-apple-ios
aarch64-apple-ios-macabi
aarch64-apple-ios-sim
aarch64-apple-tvos
aarch64-apple-tvos-sim
aarch64-apple-visionos
aarch64-apple-visionos-sim
aarch64-apple-watchos
aarch64-apple-watchos-sim
aarch64-kmc-solid_asp3
aarch64-linux-android
aarch64-nintendo-switch-freestanding
aarch64-pc-windows-gnullvm
aarch64-pc-windows-msvc
aarch64-unknown-freebsd
aarch64-unknown-fuchsia
aarch64-unknown-hermit
aarch64-unknown-illumos
aarch64-unknown-linux-gnu
aarch64-unknown-linux-gnu_ilp32
aarch64-unknown-linux-musl
aarch64-unknown-linux-ohos
aarch64-unknown-netbsd
aarch64-unknown-none
aarch64-unknown-none-softfloat
aarch64-unknown-nto-qnx700
aarch64-unknown-nto-qnx710
aarch64-unknown-nto-qnx710_iosock
aarch64-unknown-nto-qnx800
aarch64-unknown-nuttx
aarch64-unknown-openbsd
aarch64-unknown-redox
aarch64-unknown-teeos
aarch64-unknown-trusty
aarch64-unknown-uefi
aarch64-uwp-windows-msvc
aarch64-wrs-vxworks
aarch64_be-unknown-linux-gnu
aarch64_be-unknown-linux-gnu_ilp32
aarch64_be-unknown-netbsd
amdgcn-amd-amdhsa
arm-linux-androideabi
arm-unknown-linux-gnueabi
arm-unknown-linux-gnueabihf
arm-unknown-linux-musleabi
arm-unknown-linux-musleabihf
arm64_32-apple-watchos
arm64e-apple-darwin
arm64e-apple-ios
arm64e-apple-tvos
arm64ec-pc-windows-msvc
armeb-unknown-linux-gnueabi
armebv7r-none-eabi
armebv7r-none-eabihf
armv4t-none-eabi
armv4t-unknown-linux-gnueabi
armv5te-none-eabi
armv5te-unknown-linux-gnueabi
armv5te-unknown-linux-musleabi
armv5te-unknown-linux-uclibceabi
armv6-unknown-freebsd
armv6-unknown-netbsd-eabihf
armv6k-nintendo-3ds
armv7-linux-androideabi
armv7-rtems-eabihf
armv7-sony-vita-newlibeabihf
armv7-unknown-freebsd
armv7-unknown-linux-gnueabi
armv7-unknown-linux-gnueabihf
armv7-unknown-linux-musleabi
armv7-unknown-linux-musleabihf
armv7-unknown-linux-ohos
armv7-unknown-linux-uclibceabi
armv7-unknown-linux-uclibceabihf
armv7-unknown-netbsd-eabihf
armv7-unknown-trusty
armv7-wrs-vxworks-eabihf
armv7a-kmc-solid_asp3-eabi
armv7a-kmc-solid_asp3-eabihf
armv7a-none-eabi
armv7a-none-eabihf
armv7a-nuttx-eabi
armv7a-nuttx-eabihf
armv7k-apple-watchos
armv7r-none-eabi
armv7r-none-eabihf
armv7s-apple-ios
armv8r-none-eabihf
bpfeb-unknown-none
bpfel-unknown-none
hexagon-unknown-linux-musl
hexagon-unknown-none-elf
loongarch32-unknown-none
loongarch32-unknown-none-softfloat
loongarch64-unknown-linux-gnu
loongarch64-unknown-linux-musl
loongarch64-unknown-linux-ohos
loongarch64-unknown-none
loongarch64-unknown-none-softfloat
m68k-unknown-linux-gnu
m68k-unknown-none-elf
mips-mti-none-elf
mips-unknown-linux-gnu
mips-unknown-linux-musl
mips-unknown-linux-uclibc
mips64-openwrt-linux-musl
mips64-unknown-linux-gnuabi64
mips64-unknown-linux-muslabi64
mips64el-unknown-linux-gnuabi64
mips64el-unknown-linux-muslabi64
mipsel-mti-none-elf
mipsel-sony-psp
mipsel-sony-psx
mipsel-unknown-linux-gnu
mipsel-unknown-linux-musl
mipsel-unknown-linux-uclibc
mipsel-unknown-netbsd
mipsel-unknown-none
mipsisa32r6-unknown-linux-gnu
mipsisa32r6el-unknown-linux-gnu
mipsisa64r6-unknown-linux-gnuabi64
mipsisa64r6el-unknown-linux-gnuabi64
msp430-none-elf
nvptx64-nvidia-cuda
powerpc-unknown-freebsd
powerpc-unknown-linux-gnu
powerpc-unknown-linux-gnuspe
powerpc-unknown-linux-musl
powerpc-unknown-linux-muslspe
powerpc-unknown-netbsd
powerpc-unknown-openbsd
powerpc-wrs-vxworks
powerpc-wrs-vxworks-spe
powerpc64-ibm-aix
powerpc64-unknown-freebsd
powerpc64-unknown-linux-gnu
powerpc64-unknown-linux-musl
powerpc64-unknown-openbsd
powerpc64-wrs-vxworks
powerpc64le-unknown-freebsd
powerpc64le-unknown-linux-gnu
powerpc64le-unknown-linux-musl
riscv32-wrs-vxworks
riscv32e-unknown-none-elf
riscv32em-unknown-none-elf
riscv32emc-unknown-none-elf
riscv32gc-unknown-linux-gnu
riscv32gc-unknown-linux-musl
riscv32i-unknown-none-elf
riscv32im-risc0-zkvm-elf
riscv32im-unknown-none-elf
riscv32ima-unknown-none-elf
riscv32imac-esp-espidf
riscv32imac-unknown-none-elf
riscv32imac-unknown-nuttx-elf
riscv32imac-unknown-xous-elf
riscv32imafc-esp-espidf
riscv32imafc-unknown-none-elf
riscv32imafc-unknown-nuttx-elf
riscv32imc-esp-espidf
riscv32imc-unknown-none-elf
riscv32imc-unknown-nuttx-elf
riscv64-linux-android
riscv64-wrs-vxworks
riscv64gc-unknown-freebsd
riscv64gc-unknown-fuchsia
riscv64gc-unknown-hermit
riscv64gc-unknown-linux-gnu
riscv64gc-unknown-linux-musl
riscv64gc-unknown-netbsd
riscv64gc-unknown-none-elf
riscv64gc-unknown-nuttx-elf
riscv64gc-unknown-openbsd
riscv64imac-unknown-none-elf
riscv64imac-unknown-nuttx-elf
s390x-unknown-linux-gnu
s390x-unknown-linux-musl
sparc-unknown-linux-gnu
sparc-unknown-none-elf
sparc64-unknown-linux-gnu
sparc64-unknown-netbsd
sparc64-unknown-openbsd
sparcv9-sun-solaris
wasm32-unknown-emscripten
wasm32-unknown-unknown
wasm32-wali-linux-musl
wasm32-wasip1
wasm32-wasip1-threads
wasm32-wasip2
wasm32v1-none
wasm64-unknown-unknown
x86_64-apple-darwin
x86_64-apple-ios
x86_64-apple-ios-macabi
x86_64-apple-tvos
x86_64-apple-watchos-sim
x86_64-fortanix-unknown-sgx
x86_64-linux-android
x86_64-lynx-lynxos178
x86_64-pc-cygwin
x86_64-pc-nto-qnx710
x86_64-pc-nto-qnx710_iosock
x86_64-pc-nto-qnx800
x86_64-pc-solaris
x86_64-pc-windows-gnu
x86_64-pc-windows-gnullvm
x86_64-pc-windows-msvc
x86_64-unikraft-linux-musl
x86_64-unknown-dragonfly
x86_64-unknown-freebsd
x86_64-unknown-fuchsia
x86_64-unknown-haiku
x86_64-unknown-hermit
x86_64-unknown-hurd-gnu
x86_64-unknown-illumos
x86_64-unknown-l4re-uclibc
x86_64-unknown-linux-gnu
x86_64-unknown-linux-gnux32
x86_64-unknown-linux-musl
x86_64-unknown-linux-none
x86_64-unknown-linux-ohos
x86_64-unknown-netbsd
x86_64-unknown-none
x86_64-unknown-openbsd
x86_64-unknown-redox
x86_64-unknown-trusty
x86_64-unknown-uefi
x86_64-uwp-windows-gnu
x86_64-uwp-windows-msvc
x86_64-win7-windows-gnu
x86_64-win7-windows-msvc
x86_64-wrs-vxworks
x86_64h-apple-darwin",
    );
}
