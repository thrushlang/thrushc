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
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;

use crate::{ParserContext, control::ParserSyncPosition, statements::block};

pub const SYNC_STATEMENTS: [TokenType; 16] = [
    TokenType::Return,
    TokenType::Static,
    TokenType::Const,
    TokenType::Struct,
    TokenType::Type,
    TokenType::Enum,
    TokenType::Local,
    TokenType::If,
    TokenType::For,
    TokenType::While,
    TokenType::Loop,
    TokenType::Continue,
    TokenType::ContinueAll,
    TokenType::Break,
    TokenType::BreakAll,
    TokenType::Defer,
];

pub const SYNC_DECLARATIONS: [TokenType; 11] = [
    TokenType::Type,
    TokenType::Struct,
    TokenType::Const,
    TokenType::Static,
    TokenType::Enum,
    TokenType::Fn,
    TokenType::AsmFn,
    TokenType::Intrinsic,
    TokenType::GlobalAsm,
    TokenType::Import,
    TokenType::Embedded,
];

impl<'parser> ParserContext<'parser> {
    pub fn sync(&mut self) {
        if let Some(position) = self.get_control_ctx().get_sync_position() {
            match position {
                ParserSyncPosition::Declaration => self::sync_with_declaration(self),
                ParserSyncPosition::Statement => self::sync_with_statement(self),
                ParserSyncPosition::Expression => self::sync_with_expression(self),
                ParserSyncPosition::NoRelevant => (),
            }
        }
    }
}

fn sync_with_declaration(ctx: &mut ParserContext) {
    loop {
        if ctx.is_eof() {
            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_DECLARATIONS.contains(&peeked.kind) {
            ctx.get_mut_symbols().finish_parameters();
            ctx.get_mut_symbols().finish_scopes();
            ctx.reset_scope();

            break;
        } else {
            ctx.get_mut_symbols().end_scope();
            ctx.end_scope();
        }

        let _ = ctx.only_advance();
    }
}

fn sync_with_statement<'parser>(ctx: &mut ParserContext<'parser>) {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.is_main_scope() {
            let peeked: &Token = ctx.peek();

            while !SYNC_DECLARATIONS.contains(&peeked.kind) {
                let _ = ctx.only_advance();
            }

            ctx.get_mut_symbols().finish_parameters();
            ctx.get_mut_symbols().finish_scopes();
            ctx.reset_scope();

            break;
        }

        if ctx.check(TokenType::RBrace) {
            let _ = ctx.only_advance();

            if !ctx.is_main_scope() {
                ctx.get_mut_symbols().end_scope();
                ctx.end_scope();
            }

            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.get_type()) {
            let first_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                block::build_block_without_start(ctx);

            if first_.is_ok() {
                let mut continue_first_loop: bool = false;

                while ctx.check_ahead(TokenType::RBrace, &SYNC_DECLARATIONS) {
                    let second_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                        block::build_block_without_start(ctx);

                    if second_.is_err() {
                        continue_first_loop = true;
                        break;
                    }
                }

                if continue_first_loop {
                    let _ = ctx.only_advance();
                    continue;
                }

                break;
            } else {
                let _ = ctx.only_advance();
                continue;
            }
        }

        let _ = ctx.only_advance();
    }
}

fn sync_with_expression<'parser>(ctx: &mut ParserContext<'parser>) {
    loop {
        if ctx.is_eof() {
            break;
        }

        if ctx.is_main_scope() {
            let peeked: &Token = ctx.peek();

            while !SYNC_DECLARATIONS.contains(&peeked.kind) {
                let _ = ctx.only_advance();
            }

            ctx.get_mut_symbols().finish_parameters();
            ctx.get_mut_symbols().finish_scopes();
            ctx.reset_scope();

            break;
        }

        if ctx.check(TokenType::RBrace) {
            let _ = ctx.only_advance();

            if !ctx.is_main_scope() {
                ctx.get_mut_symbols().end_scope();
                ctx.end_scope();
            }

            break;
        }

        let peeked: &Token = ctx.peek();

        if SYNC_STATEMENTS.contains(&peeked.get_type()) {
            let first_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                block::build_block_without_start(ctx);

            if first_.is_ok() {
                let mut continue_first_loop: bool = false;

                while ctx.check_ahead(TokenType::RBrace, &SYNC_DECLARATIONS) {
                    let second_: Result<Ast<'_>, thrustc_errors::CompilationIssue> =
                        block::build_block_without_start(ctx);

                    if second_.is_err() {
                        continue_first_loop = true;
                        break;
                    }
                }

                if continue_first_loop {
                    let _ = ctx.only_advance();
                    continue;
                }

                break;
            } else {
                let _ = ctx.only_advance();
                continue;
            }
        }

        let _ = ctx.only_advance();
    }
}
