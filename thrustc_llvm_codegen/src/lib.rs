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


use thrustc_ast::Ast;

use crate::{codegen::LLVMCodegen, context::LLVMCodeGenContext, metadata::LLVMMetadata};

mod abort;
mod anchor;
mod atomic;
mod attrbuilder;
mod block;
mod brancher;
mod builtins;
mod cast;
mod codegen;
pub mod context;
mod debug;
mod expressions;
mod globals;
mod impls;
pub mod jit;
mod memheap;
mod memory;
mod memstack;
mod memstatic;
mod metadata;
mod obfuscation;
pub mod optimizer;
mod predicates;
mod statements;
mod table;
mod targettriple;
mod traits;
mod typegeneration;
mod types;
mod utils;

pub struct LLVMCompiler;

impl<'a, 'ctx> LLVMCompiler {
    #[inline]
    pub fn compile(context: &'a mut LLVMCodeGenContext<'a, 'ctx>, ast: &'ctx [Ast<'ctx>]) {
        LLVMMetadata::setup_platform_independent(context);
        LLVMCodegen::generate(context, ast);
    }
}
