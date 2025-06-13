use crate::frontend::{
    semantic::typeresolver::{context::TypeResolverContext, table::TypeResolverSymbols},
    types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
};

mod context;
mod table;

#[derive(Debug)]
pub struct TypeResolver;

impl<'typer> TypeResolver {
    pub fn resolve(
        stmts: &'typer mut [ThrushStatement<'typer>],
    ) -> &'typer [ThrushStatement<'typer>] {
        let mut typer_ctx: TypeResolverContext = TypeResolverContext::new();
        let mut symbols: TypeResolverSymbols = TypeResolverSymbols::new();

        stmts.iter_mut().for_each(|stmt| {
            Self::swap_types(stmt, &mut typer_ctx, &mut symbols);
        });

        stmts
    }

    fn swap_types(
        stmt: &'typer mut ThrushStatement,
        typer_ctx: &mut TypeResolverContext,
        symbols: &mut TypeResolverSymbols<'typer>,
    ) {
        match stmt {
            ThrushStatement::EntryPoint { body, .. } => {
                if let ThrushStatement::Block { stmts, .. } = &mut **body {
                    stmts.iter_mut().for_each(|stmt| {
                        Self::swap_types(stmt, typer_ctx, symbols);
                    });
                }
            }

            ThrushStatement::Block { stmts, .. } => {
                stmts.iter_mut().for_each(|stmt| {
                    Self::swap_types(stmt, typer_ctx, symbols);
                });
            }

            ThrushStatement::Function {
                name,
                parameter_types,
                body,
                ..
            } => {
                symbols.new_function(name, parameter_types);

                if let ThrushStatement::Block { stmts, .. } = &mut **body {
                    stmts.iter_mut().for_each(|stmt| {
                        Self::swap_types(stmt, typer_ctx, symbols);
                    });
                }
            }

            ThrushStatement::Local { kind, value, .. } => {
                let target_type: ThrushType = kind.get_base_type();

                typer_ctx.set_numeric_target_type(target_type);
                Self::swap_types(value, typer_ctx, symbols);
                typer_ctx.reset_numeric_target_type();
            }

            ThrushStatement::LLI { kind, value, .. } => {
                let target_type: ThrushType = kind.get_base_type();

                typer_ctx.set_numeric_target_type(target_type);
                Self::swap_types(value, typer_ctx, symbols);
                typer_ctx.reset_numeric_target_type();
            }

            ThrushStatement::For {
                local,
                cond,
                actions,
                block,
                ..
            } => {
                Self::swap_types(local, typer_ctx, symbols);
                Self::swap_types(cond, typer_ctx, symbols);
                Self::swap_types(actions, typer_ctx, symbols);
                Self::swap_types(block, typer_ctx, symbols);
            }

            ThrushStatement::BinaryOp {
                left, right, kind, ..
            } => {
                if let Some(mut_target_type) = typer_ctx.get_mut_target_type() {
                    if Self::is_compatible_numeric_cast(mut_target_type, kind) {
                        *kind = mut_target_type.clone();
                    }
                } else if let Some(array_items_target_type) =
                    typer_ctx.get_array_items_target_type()
                {
                    if Self::is_compatible_numeric_cast(array_items_target_type, kind) {
                        *kind = array_items_target_type.clone();
                    }
                } else if let Some(call_arg_target_type) = typer_ctx.get_call_arg_target_type() {
                    if Self::is_compatible_numeric_cast(call_arg_target_type, kind) {
                        *kind = call_arg_target_type.clone();
                    }
                } else if let Some(numeric_target_type) = typer_ctx.get_numeric_target_type() {
                    if Self::is_compatible_numeric_cast(numeric_target_type, kind) {
                        *kind = numeric_target_type.clone();
                    }
                }

                Self::swap_types(left, typer_ctx, symbols);
                Self::swap_types(right, typer_ctx, symbols);
            }

            ThrushStatement::Call {
                name, args, kind, ..
            } => {
                if let Some(mut_target_type) = typer_ctx.get_mut_target_type() {
                    if Self::is_compatible_numeric_cast(mut_target_type, kind) {
                        *kind = mut_target_type.clone();
                    }
                } else if let Some(array_items_target_type) =
                    typer_ctx.get_array_items_target_type()
                {
                    if Self::is_compatible_numeric_cast(array_items_target_type, kind) {
                        *kind = array_items_target_type.clone();
                    }
                } else if let Some(call_arg_target_type) = typer_ctx.get_call_arg_target_type() {
                    if Self::is_compatible_numeric_cast(call_arg_target_type, kind) {
                        *kind = call_arg_target_type.clone();
                    }
                } else if let Some(numeric_target_type) = typer_ctx.get_numeric_target_type() {
                    if Self::is_compatible_numeric_cast(numeric_target_type, kind) {
                        *kind = numeric_target_type.clone();
                    }
                }

                if let Some(types) = symbols.get_function(name) {
                    for (target_type, arg) in types.iter().zip(args) {
                        typer_ctx.set_call_arg_target_type(target_type.clone());

                        Self::swap_types(arg, typer_ctx, symbols);

                        typer_ctx.reset_call_arg_target_type();
                    }
                }
            }

            ThrushStatement::Integer { kind, .. } => {
                if let Some(mut_target_type) = typer_ctx.get_mut_target_type() {
                    if Self::is_compatible_numeric_cast(mut_target_type, kind) {
                        *kind = mut_target_type.clone();
                    }
                } else if let Some(array_items_target_type) =
                    typer_ctx.get_array_items_target_type()
                {
                    if Self::is_compatible_numeric_cast(array_items_target_type, kind) {
                        *kind = array_items_target_type.clone();
                    }
                } else if let Some(call_arg_target_type) = typer_ctx.get_call_arg_target_type() {
                    if Self::is_compatible_numeric_cast(call_arg_target_type, kind) {
                        *kind = call_arg_target_type.clone();
                    }
                } else if let Some(numeric_target_type) = typer_ctx.get_numeric_target_type() {
                    if Self::is_compatible_numeric_cast(numeric_target_type, kind) {
                        *kind = numeric_target_type.clone();
                    }
                }
            }

            ThrushStatement::Float { kind, .. } => {
                if let Some(mut_target_type) = typer_ctx.get_mut_target_type() {
                    if Self::is_compatible_numeric_cast(mut_target_type, kind) {
                        *kind = mut_target_type.clone();
                    }
                } else if let Some(array_items_target_type) =
                    typer_ctx.get_array_items_target_type()
                {
                    if Self::is_compatible_numeric_cast(array_items_target_type, kind) {
                        *kind = array_items_target_type.clone();
                    }
                } else if let Some(call_arg_target_type) = typer_ctx.get_call_arg_target_type() {
                    if Self::is_compatible_numeric_cast(call_arg_target_type, kind) {
                        *kind = call_arg_target_type.clone();
                    }
                } else if let Some(numeric_target_type) = typer_ctx.get_numeric_target_type() {
                    if Self::is_compatible_numeric_cast(numeric_target_type, kind) {
                        *kind = numeric_target_type.clone();
                    }
                }
            }

            ThrushStatement::Array { items, kind, .. } => {
                if let Some(array_items_target_type) = typer_ctx.get_array_items_target_type() {
                    if Self::is_compatible_numeric_cast(array_items_target_type, kind) {
                        *kind = array_items_target_type.clone();
                    }
                } else if let Some(mut_target_type) = typer_ctx.get_mut_target_type() {
                    if Self::is_compatible_numeric_cast(mut_target_type, kind) {
                        *kind = mut_target_type.clone();
                    }
                } else if let Some(call_arg_target_type) = typer_ctx.get_call_arg_target_type() {
                    if Self::is_compatible_numeric_cast(call_arg_target_type, kind) {
                        *kind = call_arg_target_type.clone();
                    }
                } else if let Some(numeric_target_type) = typer_ctx.get_numeric_target_type() {
                    if Self::is_compatible_numeric_cast(numeric_target_type, kind) {
                        *kind = numeric_target_type.clone();
                    }
                }

                let array_type: &ThrushType = kind.get_array_base_type();

                items.iter_mut().for_each(|item| {
                    typer_ctx.set_array_items_target_type(array_type.clone());

                    Self::swap_types(item, typer_ctx, symbols);

                    typer_ctx.reset_array_items_target_type();
                });
            }

            ThrushStatement::Mut { source, value, .. } => {
                if let Some(any_reference) = &mut source.0 {
                    let reference: &mut Box<ThrushStatement> = &mut any_reference.1;

                    if let Ok(raw_target_type) = reference.get_value_type() {
                        let target_type: ThrushType = raw_target_type.get_base_type();

                        typer_ctx.set_mut_target_type(target_type);

                        Self::swap_types(value, typer_ctx, symbols);

                        typer_ctx.reset_mut_target_type()
                    }
                } else if let Some(expr) = &mut source.1 {
                    if let Ok(raw_target_type) = expr.get_value_type() {
                        let target_type: ThrushType = raw_target_type.get_base_type();

                        typer_ctx.set_mut_target_type(target_type);

                        Self::swap_types(value, typer_ctx, symbols);

                        typer_ctx.reset_mut_target_type();
                    }
                }
            }

            _ => (),
        }
    }

    pub fn is_compatible_numeric_cast(target_type: &ThrushType, from_type: &ThrushType) -> bool {
        match (target_type, from_type) {
            (ThrushType::S16, ThrushType::S8) => true,
            (ThrushType::S32, ThrushType::S16 | ThrushType::S8) => true,
            (ThrushType::S64, ThrushType::S32 | ThrushType::S16 | ThrushType::S8) => true,
            (ThrushType::U16, ThrushType::U8) => true,
            (ThrushType::U32, ThrushType::U16 | ThrushType::U8) => true,
            (ThrushType::U64, ThrushType::U32 | ThrushType::U16 | ThrushType::U8) => true,

            (ThrushType::F64, ThrushType::F32) => true,

            (ThrushType::FixedArray(target_type, ..), ThrushType::FixedArray(from_type, ..)) => {
                Self::is_compatible_numeric_cast(target_type, from_type)
            }

            _ => false,
        }
    }
}
