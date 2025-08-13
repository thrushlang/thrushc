use colored::Colorize;

use crate::core::compiler::constants::{LLVM_TARGET_TRIPLES_X86_64, LLVM_X86_64_SUPPORTED_CPUS};

/* ######################################################################


    LLVM BACKEND | CLI UTILS - START


########################################################################*/

#[inline]
pub fn is_supported_llvm_cpu_target(cpu_target: &str) -> bool {
    LLVM_X86_64_SUPPORTED_CPUS.contains(&cpu_target)
}

#[inline]
pub fn is_supported_llvm_target_triple(target: &str) -> bool {
    LLVM_TARGET_TRIPLES_X86_64.contains(&target)
}

pub fn print_llvm_supported_cpus() {
    println!(
        "Supported LLVM CPUs count: {}\n",
        LLVM_X86_64_SUPPORTED_CPUS.len()
    );

    println!(
        "Supported {} CPUs:\n",
        "X86_64".custom_color((141, 141, 142)).bold().underline()
    );

    LLVM_X86_64_SUPPORTED_CPUS.iter().for_each(|cpu| {
        println!("- {}", cpu);
    });
}

#[inline]
pub fn print_llvm_supported_targets_triples() {
    println!(
        "Supported LLVM targets triple count: {}\n",
        LLVM_X86_64_SUPPORTED_CPUS.len()
    );

    println!(
        "Supported {} Targets Triples:\n",
        "X86_64".custom_color((141, 141, 142)).bold().underline()
    );

    LLVM_TARGET_TRIPLES_X86_64.iter().for_each(|tg_pg| {
        println!("- {}", tg_pg);
    });
}

/* ######################################################################


    LLVM BACKEND | CLI UTILS - END


########################################################################*/
