/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

#[derive(Debug)]
pub struct LLVMTargetTriple {
    arch: String,
    vendor: String,
    os: String,
    abi: String,
}

impl LLVMTargetTriple {
    #[inline]
    pub fn new(target_triple: String) -> Self {
        let triple_dissasembled: Vec<&str> = target_triple.split('-').collect();

        let arch: String = triple_dissasembled
            .first()
            .unwrap_or(&"unknown")
            .to_string();

        let vendor: String = triple_dissasembled.get(1).unwrap_or(&"unknown").to_string();
        let os: String = triple_dissasembled.get(2).unwrap_or(&"unknown").to_string();
        let abi: String = triple_dissasembled.get(3).unwrap_or(&"unknown").to_string();

        Self {
            arch,
            vendor,
            os,
            abi,
        }
    }
}

impl LLVMTargetTriple {
    #[inline]
    pub fn get_abi(&self) -> &str {
        &self.abi
    }

    #[inline]
    pub fn get_arch(&self) -> &str {
        &self.arch
    }

    #[inline]
    pub fn get_os(&self) -> &str {
        &self.os
    }

    #[inline]
    pub fn get_vendor(&self) -> &str {
        &self.vendor
    }
}

impl LLVMTargetTriple {
    #[inline]
    pub fn has_posix_thread_model(&self) -> bool {
        matches!(
            self.get_os(),
            "linux" | "android" | "freebsd" | "netbsd" | "openbsd"
        ) || matches!(self.get_abi(), "gnu")
    }
}

impl LLVMTargetTriple {
    #[inline]
    pub fn is_object_format_mach_o(&self) -> bool {
        // https://llvm.org/doxygen/Triple_8cpp_source.html
        // https://github.com/llvm/llvm-project/blob/648193e1619f7af68230f6eddc526af542446cd8/llvm/include/llvm/TargetParser/Triple.h#L804

        let arch_ok: bool = matches!(
            self.arch.as_str(),
            "aarch64"
                | "aarch64_32"
                | "aarch64_be"
                | "arm"
                | "thumb"
                | "armeb"
                | "thumbeb"
                | "x86"
                | "x86_64"
                | "ppc"
                | "ppc64"
        );

        if !arch_ok {
            return false;
        }

        let darwin_os: bool = self.is_os_darwin();

        let macho_abi: bool = self.abi.eq_ignore_ascii_case("macho")
            || self.abi.ends_with("macho")
            || self.abi.contains("macho");

        darwin_os || macho_abi
    }

    #[inline]
    pub fn is_xcoff_object_format(&self) -> bool {
        // https://llvm.org/doxygen/Triple_8cpp_source.html
        // https://github.com/llvm/llvm-project/blob/648193e1619f7af68230f6eddc526af542446cd8/llvm/include/llvm/TargetParser/Triple.h#L804

        let arch_ok: bool = matches!(
            self.arch.as_str(),
            "powerpc" | "ppc" | "powerpc64" | "ppc64" | "powerpcle" | "ppcle"
        );

        if !arch_ok {
            return false;
        }

        let is_aix: bool = self.os.eq_ignore_ascii_case("aix")
            || self.os.starts_with("aix")
            || self.os.ends_with("aix");

        let xcoff_abi: bool = self.abi.eq_ignore_ascii_case("xcoff")
            || self.abi.ends_with("xcoff")
            || self.abi.contains("xcoff");

        is_aix || xcoff_abi
    }

    #[inline]
    pub fn is_object_format_elf(&self) -> bool {
        // https://llvm.org/doxygen/Triple_8cpp_source.html

        let arch_in_elf_list: bool = matches!(
            self.arch.as_str(),
            "aarch64_be"
                | "amdgcn"
                | "amdil64"
                | "amdil"
                | "arc"
                | "armeb"
                | "avr"
                | "bpfeb"
                | "bpfel"
                | "csky"
                | "hexagon"
                | "hsail64"
                | "hsail"
                | "kalimba"
                | "lanai"
                | "loongarch32"
                | "loongarch64"
                | "m68k"
                | "mips64"
                | "mips64el"
                | "mips"
                | "msp430"
                | "nvptx64"
                | "nvptx"
                | "ppc64le"
                | "ppcle"
                | "r600"
                | "renderscript32"
                | "renderscript64"
                | "riscv32"
                | "riscv64"
                | "riscv32be"
                | "riscv64be"
                | "shave"
                | "sparc"
                | "sparcel"
                | "sparcv9"
                | "spir64"
                | "spir"
                | "tce"
                | "tcele"
                | "thumbeb"
                | "ve"
                | "xcore"
                | "xtensa"
        );

        if !arch_in_elf_list {
            return false;
        }

        if self.is_object_format_mach_o() {
            return false;
        }

        if self.is_xcoff_object_format() {
            return false;
        }

        let coff_os: bool = matches!(self.os.as_str(), "win32" | "windows" | "uefi")
            || self.os.eq_ignore_ascii_case("windows");

        if coff_os {
            return false;
        }

        let is_systemz: bool = self.arch == "systemz" || self.arch == "s390x";
        let is_zos: bool = self.os.eq_ignore_ascii_case("zos") || self.os.starts_with("zos");

        if is_systemz && is_zos {
            return false;
        }

        if self.arch.contains("wasm32") || self.arch.contains("wasm64") {
            return false;
        }

        if self.arch.contains("spirv") {
            return false;
        }

        if self.arch.contains("dxil") {
            return false;
        }

        let explicit_elf_abi: bool = self.abi.eq_ignore_ascii_case("elf")
            || self.abi.ends_with("elf")
            || self.abi.contains("elf");

        if explicit_elf_abi {
            return true;
        }

        true
    }

    #[inline]
    pub fn is_os_darwin(&self) -> bool {
        // https://llvm.org/doxygen/Triple_8cpp_source.html
        // https://github.com/llvm/llvm-project/blob/648193e1619f7af68230f6eddc526af542446cd8/llvm/include/llvm/TargetParser/Triple.h#L804

        matches!(
            self.os.as_str(),
            "darwin"
                | "macosx"
                | "macos"
                | "ios"
                | "tvos"
                | "watchos"
                | "xros"
                | "bridgeos"
                | "driverkit"
        ) || self.os.contains("darwin")
            || self.os.contains("macos")
            || self.os.contains("ios")
            || self.os.contains("tvos")
            || self.os.contains("watchos")
            || self.os.contains("xros")
    }

    #[inline]
    pub fn is_x86_64(&self) -> bool {
        matches!(self.arch.as_str(), "x86_64" | "amd64")
    }

    #[inline]
    pub fn is_aarch64(&self) -> bool {
        matches!(
            self.arch.as_str(),
            "aarch64" | "arm64" | "aarch64_32" | "aarch64_be"
        )
    }

    #[inline]
    pub fn is_riscv64(&self) -> bool {
        matches!(self.arch.as_str(), "riscv64" | "riscv64be")
    }

    #[inline]
    pub fn is_ppc64(&self) -> bool {
        matches!(self.arch.as_str(), "ppc64" | "ppc64le" | "powerpc64le")
    }

    #[inline]
    pub fn is_mips64(&self) -> bool {
        matches!(self.arch.as_str(), "mips64" | "mips64el")
    }

    #[inline]
    pub fn is_systemz(&self) -> bool {
        matches!(self.arch.as_str(), "systemz" | "s390x")
    }

    #[inline]
    pub fn is_loongarch64(&self) -> bool {
        matches!(self.arch.as_str(), "loongarch64")
    }

    #[inline]
    pub fn is_wasm64(&self) -> bool {
        matches!(self.arch.as_str(), "wasm64")
    }

    #[inline]
    pub fn is_avr(&self) -> bool {
        self.arch.contains("avr")
    }

    #[inline]
    pub fn is_arc(&self) -> bool {
        self.arch.contains("arc")
    }

    #[inline]
    pub fn is_csky(&self) -> bool {
        self.arch.contains("csky")
    }

    #[inline]
    pub fn is_arm_family(&self) -> bool {
        self.arch.contains("arm") || self.arch.contains("aarch64") || self.arch.contains("thumb")
    }

    #[inline]
    pub fn is_hexagon(&self) -> bool {
        self.arch.contains("hexagon")
    }

    #[inline]
    pub fn is_msp430(&self) -> bool {
        self.arch.contains("msp430")
    }

    #[inline]
    pub fn is_ppc(&self) -> bool {
        self.arch.contains("powerpc") || self.arch.contains("ppc")
    }

    #[inline]
    pub fn is_sparc(&self) -> bool {
        self.arch.contains("sparc")
    }

    #[inline]
    pub fn is_xcore(&self) -> bool {
        self.arch.contains("xcore")
    }

    #[inline]
    pub fn is_os_aix(&self) -> bool {
        self.os.contains("aix")
    }

    #[inline]
    pub fn is_64_bit(&self) -> bool {
        self.is_x86_64()
            || self.is_aarch64()
            || self.is_wasm64()
            || self.is_riscv64()
            || self.is_ppc64()
            || self.is_mips64()
            || self.is_systemz()
            || self.is_loongarch64()
            || self.arch.contains("64")
    }

    #[inline]
    pub fn get_normalized(&self) -> String {
        format!("{}-{}-{}-{}", self.arch, self.vendor, self.os, self.abi)
    }
}
