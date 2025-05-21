use ahash::AHashMap as HashMap;

use crate::{
    frontend::lexer::span::Span,
    middle::types::frontend::{
        linter::types::{
            LinterConstantInfo, LinterConstants, LinterFunctionInfo, LinterFunctionParameterInfo,
            LinterFunctionParameters, LinterFunctions, LinterLocalInfo, LinterLocals,
        },
        parser::stmts::stmt::ThrushStatement,
    },
    standard::{
        constants::MINIMAL_WARNINGS_CAPACITY,
        diagnostic::Diagnostician,
        error::ThrushCompilerIssue,
        logging::{self, LoggingType},
        misc::CompilerFile,
    },
};

pub struct Linter<'linter> {
    stmts: &'linter [ThrushStatement<'linter>],
    current: usize,
    warnings: Vec<ThrushCompilerIssue>,
    diagnostician: Diagnostician,
    functions: LinterFunctions<'linter>,
    constants: LinterConstants<'linter>,
    locals: LinterLocals<'linter>,
    parameters: LinterFunctionParameters<'linter>,
    parameters_to_analyze: LinterFunctionParameters<'linter>,
    locals_to_analyze: LinterLocals<'linter>,
    scope: usize,
}

impl<'linter> Linter<'linter> {
    pub fn new(stmts: &'linter [ThrushStatement], file: &'linter CompilerFile) -> Self {
        Self {
            stmts,
            current: 0,
            warnings: Vec::with_capacity(MINIMAL_WARNINGS_CAPACITY),
            diagnostician: Diagnostician::new(file),
            functions: HashMap::with_capacity(255),
            constants: HashMap::with_capacity(255),
            locals: Vec::with_capacity(255),
            parameters: HashMap::with_capacity(255),
            parameters_to_analyze: HashMap::with_capacity(255),
            locals_to_analyze: Vec::with_capacity(255),
            scope: 0,
        }
    }

    pub fn check(&mut self) {
        self.declare();

        while !self.is_eof() {
            let instruction: &ThrushStatement = self.peek();

            self.analyze_stmt(instruction);

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

    pub fn analyze_stmt(&mut self, instruction: &'linter ThrushStatement) {
        if let ThrushStatement::EntryPoint { body, .. } = instruction {
            self.analyze_stmt(body);
        }

        if let ThrushStatement::Function {
            parameters, body, ..
        } = instruction
        {
            if body.is_block() {
                self.start_parameters(parameters);

                self.analyze_stmt(body);

                self.end_parameters();
            }
        }

        if let ThrushStatement::BinaryOp { left, right, .. } = instruction {
            self.analyze_stmt(left);
            self.analyze_stmt(right);
        }

        if let ThrushStatement::UnaryOp { expression, .. } = instruction {
            self.analyze_stmt(expression);
        }

        if let ThrushStatement::Block { stmts, .. } = instruction {
            self.begin_scope();

            stmts.iter().for_each(|stmt| {
                self.analyze_stmt(stmt);
            });

            self.end_scope();
        }

        if let ThrushStatement::Local {
            name,
            value,
            span,
            is_mutable,
            ..
        } = instruction
        {
            let scope: usize = self.get_scope();
            self.locals[scope].insert(name, (*span, false, !is_mutable));

            self.analyze_stmt(value);
        }

        if let ThrushStatement::Call { name, .. } = instruction {
            let function: &mut LinterFunctionInfo = self.get_mut_function(name);
            function.2 = true;
        }

        if let ThrushStatement::ConstRef { name, .. } = instruction {
            let constant: &mut LinterConstantInfo = self.get_mut_constant(name);
            constant.1 = true;
        }

        if let ThrushStatement::LocalRef { name, .. } = instruction {
            if let Ok(local) = self.get_mut_local(name) {
                local.1 = true;
            }

            if let Ok(parameter) = self.get_mut_parameter(name) {
                parameter.1 = true;
            }
        }

        if let ThrushStatement::Mut { source, .. } = instruction {
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
        self.stmts
            .iter()
            .filter(|instruction| instruction.is_function())
            .for_each(|instruction| {
                if let ThrushStatement::Function {
                    name, span, body, ..
                } = instruction
                {
                    self.functions.insert(name, (&**body, *span, false));
                }
            });

        self.stmts
            .iter()
            .filter(|instruction| instruction.is_constant())
            .for_each(|instruction| {
                if let ThrushStatement::Const { name, span, .. } = instruction {
                    self.constants.insert(name, (*span, false));
                }
            });
    }

    fn get_mut_local(&mut self, name: &str) -> Result<&mut LinterLocalInfo, ()> {
        for i in (0..=self.get_scope()).rev() {
            if self.locals[i].contains_key(name) {
                return Ok(self.locals[i].get_mut(name).unwrap());
            }
        }

        Err(())
    }

    fn get_mut_constant(&mut self, name: &str) -> &mut LinterConstantInfo {
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

    fn get_mut_function(&mut self, name: &str) -> &mut LinterFunctionInfo<'linter> {
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

    fn start_parameters(&mut self, parameters: &'linter [ThrushStatement<'linter>]) {
        parameters.iter().for_each(|instruction| {
            if let ThrushStatement::FunctionParameter {
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

    fn get_mut_parameter(&mut self, name: &str) -> Result<&mut LinterFunctionParameterInfo, ()> {
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
        self.locals.push(HashMap::with_capacity(255));
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

    fn peek(&self) -> &'linter ThrushStatement<'linter> {
        self.stmts.get(self.current).unwrap_or_else(|| {
            logging::log(
                LoggingType::Panic,
                "Attempting to get instruction in invalid current position.",
            );

            unreachable!()
        })
    }

    fn is_eof(&self) -> bool {
        self.current >= self.stmts.len()
    }
}
