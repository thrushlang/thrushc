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


use colored::Colorize;

use inkwell::module::Module;
use thrustc_options::CompilationUnit;

pub fn llvm_codegen(llvm_module: &Module, file: &CompilationUnit) -> Result<(), ()> {
    if let Err(codegen_error) = llvm_module.verify() {
        thrustc_logging::print_backend_panic_not_exit(
            thrustc_logging::LoggingType::BackendPanic,
            codegen_error.to_string().trim_end(),
        );

        thrustc_logging::write(
            thrustc_logging::OutputIn::Stderr,
            &format!(
                "\r{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "FAILED".bright_red().bold(),
                file.get_path().display()
            ),
        );

        return Err(());
    }

    Ok(())
}
