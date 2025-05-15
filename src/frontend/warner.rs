use ahash::AHashMap as HashMap;

use crate::{
    middle::types::frontend::{
        parser::stmts::instruction::Instruction,
        warner::types::{
            WarnerConstantInfo, WarnerFunctionInfo, WarnerFunctionParameterInfo, WarnerLocalInfo,
            WarnerLocals, WarnersConstants, WarnersFunctionParameters, WarnersFunctions,
        },
    },
    standard::{
        constants::MINIMAL_WARNINGS_CAPACITY,
        diagnostic::Diagnostician,
        error::ThrushCompilerIssue,
        logging::{self, LoggingType},
        misc::CompilerFile,
    },
};

use super::lexer::Span;

const MINIMAL_WARNER_FUNCTIONS_CAPACITY: usize = 255;
const MINIMAL_WARNER_CONSTANTS_CAPACITY: usize = 255;
const MINIMAL_WARNER_LOCALS_CAPACITY: usize = 255;
const MINIMAL_WARNER_PARAMETERS_CAPACITY: usize = 255;

pub struct Warner<'warner> {
    instructions: &'warner [Instruction<'warner>],
    current: usize,
    warnings: Vec<ThrushCompilerIssue>,
    diagnostician: Diagnostician,

    functions: WarnersFunctions<'warner>,
    constants: WarnersConstants<'warner>,
    locals: WarnerLocals<'warner>,
    parameters: WarnersFunctionParameters<'warner>,
    parameters_to_analyze: WarnersFunctionParameters<'warner>,
    locals_to_analyze: WarnerLocals<'warner>,
    scope: usize,
}

impl<'warner> Warner<'warner> {
    pub fn new(instructions: &'warner [Instruction], file: &'warner CompilerFile) -> Self {
        Self {
            instructions,
            current: 0,
            warnings: Vec::with_capacity(MINIMAL_WARNINGS_CAPACITY),
            diagnostician: Diagnostician::new(file),
            functions: HashMap::with_capacity(MINIMAL_WARNER_FUNCTIONS_CAPACITY),
            constants: HashMap::with_capacity(MINIMAL_WARNER_CONSTANTS_CAPACITY),
            locals: Vec::with_capacity(MINIMAL_WARNER_LOCALS_CAPACITY),
            parameters: HashMap::with_capacity(MINIMAL_WARNER_LOCALS_CAPACITY),
            parameters_to_analyze: HashMap::with_capacity(MINIMAL_WARNER_PARAMETERS_CAPACITY),
            locals_to_analyze: Vec::with_capacity(MINIMAL_WARNER_PARAMETERS_CAPACITY),
            scope: 0,
        }
    }

    pub fn check(&mut self) {
        self.declare();

        while !self.is_eof() {
            let instruction: &Instruction = self.peek();

            self.analyze_instruction(instruction);

            self.advance();
        }

        self.generate_warnings();

        if !self.warnings.is_empty() {
            self.warnings.iter().for_each(|warn| {
                self.diagnostician
                    .build_diagnostic(warn, LoggingType::Warning);
            });
        }
    }

    pub fn analyze_instruction(&mut self, instruction: &'warner Instruction) {
        if let Instruction::EntryPoint { body, .. } = instruction {
            self.analyze_instruction(body);
        }

        if let Instruction::Function {
            parameters, body, ..
        } = instruction
        {
            if body.is_block() {
                self.start_parameters(parameters);

                self.analyze_instruction(body);

                self.end_parameters();
            }
        }

        if let Instruction::BinaryOp { left, right, .. } = instruction {
            self.analyze_instruction(left);
            self.analyze_instruction(right);
        }

        if let Instruction::UnaryOp { expression, .. } = instruction {
            self.analyze_instruction(expression);
        }

        if let Instruction::Block { stmts } = instruction {
            self.begin_scope();

            stmts.iter().for_each(|stmt| {
                self.analyze_instruction(stmt);
            });

            self.end_scope();
        }

        if let Instruction::Local {
            name,
            value,
            span,
            is_mutable,
            ..
        } = instruction
        {
            let scope: usize = self.get_scope();
            self.locals[scope].insert(name, (*span, false, !is_mutable));

            self.analyze_instruction(value);
        }

        if let Instruction::Call { name, .. } = instruction {
            let function: &mut WarnerFunctionInfo = self.get_mut_function(name);
            function.2 = true;
        }

        if let Instruction::ConstRef { name, .. } = instruction {
            let constant: &mut WarnerConstantInfo = self.get_mut_constant(name);
            constant.1 = true;
        }

        if let Instruction::LocalRef { name, .. } = instruction {
            if let Ok(local) = self.get_mut_local(name) {
                local.1 = true;
            }

            if let Ok(parameter) = self.get_mut_parameter(name) {
                parameter.1 = true;
            }
        }

        if let Instruction::LocalMut { source, .. } = instruction {
            if let Some(local_name) = source.0 {
                if let Ok(local) = self.get_mut_local(local_name) {
                    local.1 = true;
                }
            }
        }
    }

    pub fn generate_warnings(&mut self) {
        self.constants.iter().for_each(|(name, info)| {
            let span: Span = info.0;
            let used: bool = info.1;

            if !used {
                self.warnings.push(ThrushCompilerIssue::Warning(
                    String::from("Constant not used"),
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        });

        self.functions.iter().for_each(|(name, info)| {
            let span: Span = info.1;
            let used: bool = info.2;

            if !used {
                self.warnings.push(ThrushCompilerIssue::Warning(
                    String::from("Function not used"),
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        });

        self.parameters_to_analyze.iter().for_each(|parameter| {
            let name: &str = parameter.0;
            let span: Span = parameter.1.0;
            let used: bool = parameter.1.1;
            let is_mutable_used: bool = parameter.1.2;

            if !used {
                self.warnings.push(ThrushCompilerIssue::Warning(
                    String::from("Parameter not used"),
                    format!("'{}' not used.", name),
                    span,
                ));
            }

            if !is_mutable_used {
                self.warnings.push(ThrushCompilerIssue::Warning(
                    String::from("Mutable parameter not used"),
                    format!("'{}' not used.", name),
                    span,
                ));
            }
        });

        self.locals_to_analyze.iter().for_each(|scope| {
            scope.iter().for_each(|(name, info)| {
                let span: Span = info.0;
                let used: bool = info.1;
                let is_mutable_used: bool = info.2;

                if !used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Local not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }

                if !is_mutable_used {
                    self.warnings.push(ThrushCompilerIssue::Warning(
                        String::from("Mutable local not used"),
                        format!("'{}' not used.", name),
                        span,
                    ));
                }
            });
        });
    }

    pub fn declare(&mut self) {
        self.instructions
            .iter()
            .filter(|instruction| instruction.is_function())
            .for_each(|instruction| {
                if let Instruction::Function {
                    name, span, body, ..
                } = instruction
                {
                    self.functions.insert(name, (&**body, *span, false));
                }
            });

        self.instructions
            .iter()
            .filter(|instruction| instruction.is_constant())
            .for_each(|instruction| {
                if let Instruction::Const { name, span, .. } = instruction {
                    self.constants.insert(name, (*span, false));
                }
            });
    }

    fn get_mut_local(&mut self, name: &str) -> Result<&mut WarnerLocalInfo, ()> {
        for i in (0..=self.get_scope()).rev() {
            if self.locals[i].contains_key(name) {
                return Ok(self.locals[i].get_mut(name).unwrap());
            }
        }

        Err(())
    }

    fn get_mut_constant(&mut self, name: &str) -> &mut WarnerConstantInfo {
        self.constants.get_mut(name).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Attempting to get warning info of the constant with name '{}'.",
                    name
                ),
            );

            unreachable!()
        })
    }

    fn get_mut_function(&mut self, name: &str) -> &mut WarnerFunctionInfo<'warner> {
        self.functions.get_mut(name).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Attempting to get warning info of the function with name '{}'.",
                    name
                ),
            );

            unreachable!()
        })
    }

    fn start_parameters(&mut self, parameters: &'warner [Instruction<'warner>]) {
        parameters.iter().for_each(|instruction| {
            if let Instruction::FunctionParameter {
                name,
                span,
                is_mutable,
                ..
            } = instruction
            {
                self.parameters.insert(name, (*span, false, !is_mutable));
            }
        });
    }

    fn get_mut_parameter(&mut self, name: &str) -> Result<&mut WarnerFunctionParameterInfo, ()> {
        if let Some(parameter) = self.parameters.get_mut(name) {
            return Ok(parameter);
        }

        Err(())
    }

    fn end_parameters(&mut self) {
        self.parameters_to_analyze.extend(self.parameters.iter());
        self.parameters.clear();
    }

    fn begin_scope(&mut self) {
        self.locals
            .push(HashMap::with_capacity(MINIMAL_WARNER_LOCALS_CAPACITY));

        self.scope += 1;
    }

    fn end_scope(&mut self) {
        self.locals_to_analyze.push(self.locals.pop().unwrap());
        self.scope -= 1;
    }

    fn get_scope(&self) -> usize {
        self.scope - 1
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            self.current += 1;
        }
    }

    fn peek(&self) -> &'warner Instruction<'warner> {
        self.instructions.get(self.current).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                "Attempting to get instruction in invalid current position.",
            );

            unreachable!()
        })
    }

    fn is_eof(&self) -> bool {
        self.current >= self.instructions.len()
    }
}
