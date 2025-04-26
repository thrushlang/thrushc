use crate::{common::misc::CompilerFile, middle::instruction::Instruction};

use super::super::{
    common::{
        constants::MINIMAL_ERROR_CAPACITY, diagnostic::Diagnostician, error::ThrushCompilerError,
    },
    logging::LoggingType,
};

use std::process;

const MINIMAL_SCOPE_CAPACITY: usize = 256;

#[derive(Debug)]
pub struct ThrushScoper<'ctx> {
    blocks: Vec<ThrushBlock<'ctx>>,
    errors: Vec<ThrushCompilerError>,
    diagnostician: Diagnostician,
}

#[derive(Debug)]
struct ThrushBlock<'ctx> {
    stmts: Vec<Instruction<'ctx>>,
}

impl<'ctx> ThrushScoper<'ctx> {
    pub fn new(file: &'ctx CompilerFile) -> Self {
        Self {
            blocks: Vec::with_capacity(MINIMAL_SCOPE_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            diagnostician: Diagnostician::new(file),
        }
    }

    pub fn check(&mut self) {
        if self.blocks.is_empty() {
            return;
        }

        for depth in (0..=self.blocks.len() - 1).rev() {
            for instruction in self.blocks[depth].stmts.iter().rev() {
                if let Err(error) = self.analyze(instruction, depth) {
                    self.errors.push(error);
                }
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .build_diagnostic(error, LoggingType::Error);
            });

            process::exit(1);
        }
    }

    fn analyze(&self, instr: &Instruction<'ctx>, depth: usize) -> Result<(), ThrushCompilerError> {
        if let Instruction::Block { stmts, .. } = instr {
            stmts
                .iter()
                .try_for_each(|instr| match self.analyze(instr, depth) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                })?;
        }

        if let Instruction::Function {
            body: Some(body), ..
        } = instr
        {
            self.analyze(body.as_ref(), depth)?;
        }

        if let Instruction::EntryPoint { body } = instr {
            self.analyze(body, depth)?;
        }

        if let Instruction::LocalRef { name, span, .. } = instr {
            if !self.is_at_current_scope(name, None, depth) {
                return Err(ThrushCompilerError::Error(
                    String::from("Undefined variable"),
                    format!("'{}' local not exist at this scope.", name),
                    *span,
                ));
            }

            let line: usize = span.get_line();

            if self.is_at_current_scope(name, None, depth)
                && !self.is_reacheable_at_current_scope(name, line, None, depth)
            {
                return Err(ThrushCompilerError::Error(
                    String::from("Unreacheable variable"),
                    format!("'{}' local is unreacheable at this point.", name),
                    *span,
                ));
            }

            return Ok(());
        }

        Ok(())
    }

    fn is_reacheable_at_current_scope(
        &self,
        name: &str,
        refvar_line: usize,
        block: Option<&Instruction<'ctx>>,
        mut depth: usize,
    ) -> bool {
        if depth > self.blocks.len() {
            return false;
        }

        if block.is_some() {
            if let Instruction::Block { stmts, .. } = block.as_ref().unwrap() {
                return stmts.iter().rev().any(|instr| match instr {
                    Instruction::Local {
                        name: other_name,
                        span,
                        ..
                    }
                    | Instruction::FunctionParameter {
                        name: other_name,
                        span,
                        ..
                    } if *other_name == name => {
                        if span.get_line() > refvar_line {
                            return false;
                        }

                        true
                    }
                    Instruction::Block { .. } => {
                        self.is_reacheable_at_current_scope(name, refvar_line, Some(instr), depth)
                    }
                    _ => {
                        depth += 1;
                        self.is_reacheable_at_current_scope(name, refvar_line, None, depth)
                    }
                });
            }
        }

        if self.blocks.len() == 1 || depth == 0 {
            self.blocks[0].stmts.iter().rev().any(|instr| match instr {
                Instruction::FunctionParameter {
                    name: instr_name,
                    span,
                    ..
                }
                | Instruction::Local {
                    name: instr_name,
                    span,
                    ..
                } if *instr_name == name => {
                    if span.get_line() > refvar_line {
                        return false;
                    }

                    true
                }
                Instruction::Block { .. } => {
                    self.is_reacheable_at_current_scope(name, refvar_line, Some(instr), depth)
                }
                _ => {
                    depth += 1;
                    self.is_reacheable_at_current_scope(name, refvar_line, None, depth)
                }
            })
        } else {
            self.blocks[depth - 1]
                .stmts
                .iter()
                .rev()
                .any(|instr| match instr {
                    Instruction::FunctionParameter {
                        name: instr_name,
                        span,
                        ..
                    }
                    | Instruction::Local {
                        name: instr_name,
                        span,
                        ..
                    } if *instr_name == name => {
                        if span.get_line() > refvar_line {
                            return false;
                        }

                        true
                    }
                    Instruction::Block { .. } => {
                        self.is_reacheable_at_current_scope(name, refvar_line, Some(instr), depth)
                    }
                    _ => {
                        depth += 1;
                        self.is_reacheable_at_current_scope(name, refvar_line, None, depth)
                    }
                })
        }
    }

    fn is_at_current_scope(
        &self,
        name: &str,
        block: Option<&Instruction<'ctx>>,
        mut depth: usize,
    ) -> bool {
        if depth > self.blocks.len() {
            return false;
        }

        if block.is_some() {
            if let Instruction::Block { stmts, .. } = block.as_ref().unwrap() {
                return stmts.iter().rev().any(|instr| match instr {
                    Instruction::Local {
                        name: instr_name, ..
                    }
                    | Instruction::FunctionParameter {
                        name: instr_name, ..
                    } if *instr_name == name => true,
                    Instruction::Block { .. } => self.is_at_current_scope(name, Some(instr), depth),
                    _ => {
                        depth += 1;
                        self.is_at_current_scope(name, None, depth)
                    }
                });
            }
        }

        if self.blocks.len() == 1 || depth == 0 {
            self.blocks[0].stmts.iter().rev().any(|instr| match &instr {
                Instruction::Local {
                    name: instr_name, ..
                }
                | Instruction::FunctionParameter {
                    name: instr_name, ..
                } if *instr_name == name => true,

                Instruction::Block { .. } => self.is_at_current_scope(name, Some(instr), depth),
                _ => {
                    depth += 1;
                    self.is_at_current_scope(name, None, depth)
                }
            })
        } else {
            self.blocks[depth - 1]
                .stmts
                .iter()
                .rev()
                .any(|instr| match &instr {
                    Instruction::Local {
                        name: instr_name, ..
                    }
                    | Instruction::FunctionParameter {
                        name: instr_name, ..
                    } if *instr_name == name => true,
                    Instruction::Block { .. } => self.is_at_current_scope(name, Some(instr), depth),
                    _ => {
                        depth += 1;
                        self.is_at_current_scope(name, None, depth)
                    }
                })
        }
    }

    pub fn add_scope(&mut self, stmts: Vec<Instruction<'ctx>>) {
        self.blocks.push(ThrushBlock { stmts });
    }
}
