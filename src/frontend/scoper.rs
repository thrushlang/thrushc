use super::super::{
    backend::compiler::{instruction::Instruction, misc::CompilerFile},
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

    pub fn add_scope(&mut self, stmts: Vec<Instruction<'ctx>>) {
        self.blocks.push(ThrushBlock { stmts });
    }

    pub fn analyze(&mut self) {
        if self.blocks.is_empty() {
            return;
        }

        for depth in (0..=self.blocks.len() - 1).rev() {
            for instruction in self.blocks[depth].stmts.iter().rev() {
                if let Err(error) = self.analyze_instruction(instruction, depth) {
                    self.errors.push(error);
                }
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician.report_error(error, LoggingType::Error);
            });

            process::exit(1);
        }
    }

    fn analyze_instruction(
        &self,
        instr: &Instruction<'ctx>,
        depth: usize,
    ) -> Result<(), ThrushCompilerError> {
        if let Instruction::Block { stmts, .. } = instr {
            stmts
                .iter()
                .try_for_each(|instr| match self.analyze_instruction(instr, depth) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(e),
                })?;
        }

        if let Instruction::Function {
            body: Some(body), ..
        } = instr
        {
            self.analyze_instruction(body.as_ref(), depth)?;
        }

        if let Instruction::EntryPoint { body } = instr {
            self.analyze_instruction(body, depth)?;
        }

        if let Instruction::LocalRef { name, line, .. } = instr {
            if !self.is_at_current_scope(name, None, depth) {
                return Err(ThrushCompilerError::Error(
                    String::from("Undefined variable"),
                    format!("Local variable '{}' not found at current scope.", name),
                    *line,
                    None,
                ));
            }

            if self.is_at_current_scope(name, None, depth)
                && !self.is_reacheable_at_current_scope(name, *line, None, depth)
            {
                return Err(ThrushCompilerError::Error(
                    String::from("Unreacheable variable"),
                    format!(
                        "Local variable '{}' is unreacheable at current scope.",
                        name
                    ),
                    *line,
                    None,
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
                        name: var_name,
                        line,
                        ..
                    } if *var_name == name => {
                        if *line > refvar_line {
                            return false;
                        }

                        true
                    }
                    Instruction::FunctionParameter {
                        name: param_name,
                        line,
                        ..
                    } if *param_name == name => {
                        if *line > refvar_line {
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
                    line,
                    ..
                }
                | Instruction::Local {
                    name: instr_name,
                    line,
                    ..
                } if *instr_name == name => {
                    if *line > refvar_line {
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
                        line,
                        ..
                    }
                    | Instruction::Local {
                        name: instr_name,
                        line,
                        ..
                    } if *instr_name == name => {
                        if *line > refvar_line {
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
}
