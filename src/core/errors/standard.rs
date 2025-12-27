use colored::Colorize;

use super::position::CompilationPosition;
use crate::core::{console::logging, diagnostic::span::Span};

#[derive(Debug, Clone)]
pub enum CompilationIssue {
    Error(CompilationIssueCode, String, Option<String>, Span),
    Warning(CompilationIssueCode, String, Span),

    FrontEndBug(
        String,
        String,
        Span,
        CompilationPosition,
        std::path::PathBuf,
        u32,
    ),
    BackenEndBug(
        String,
        String,
        Span,
        CompilationPosition,
        std::path::PathBuf,
        u32,
    ),
}

impl CompilationIssue {
    #[inline]
    pub fn is_bug(&self) -> bool {
        matches!(
            self,
            CompilationIssue::FrontEndBug(..) | CompilationIssue::BackenEndBug(..)
        )
    }
}

lazy_static::lazy_static! {
    pub static ref COMPILATION_ISSUE_CODE_EXPLANATIONS: ahash::AHashMap<CompilationIssueCode, &'static str> = {
        let mut explanations: ahash::AHashMap<CompilationIssueCode, &'static str> = ahash::AHashMap::with_capacity(100);

        explanations.insert(CompilationIssueCode::E0001, r#""#);
        explanations.insert(CompilationIssueCode::E0002, r#""#);
        explanations.insert(CompilationIssueCode::E0003, r#""#);
        explanations.insert(CompilationIssueCode::E0004, r#""#);
        explanations.insert(CompilationIssueCode::E0005, r#""#);
        explanations.insert(CompilationIssueCode::E0006, r#""#);
        explanations.insert(CompilationIssueCode::E0007, r#""#);
        explanations.insert(CompilationIssueCode::E0008, r#""#);
        explanations.insert(CompilationIssueCode::E0010, r#""#);
        explanations.insert(CompilationIssueCode::E0011, r#""#);
        explanations.insert(CompilationIssueCode::E0012, r#""#);
        explanations.insert(CompilationIssueCode::E0013, r#""#);
        explanations.insert(CompilationIssueCode::E0014, r#""#);
        explanations.insert(CompilationIssueCode::E0015, r#""#);
        explanations.insert(CompilationIssueCode::E0016, r#""#);
        explanations.insert(CompilationIssueCode::E0017, r#""#);
        explanations.insert(CompilationIssueCode::E0018, r#""#);
        explanations.insert(CompilationIssueCode::E0019, r#""#);
        explanations.insert(CompilationIssueCode::E0020, r#""#);
        explanations.insert(CompilationIssueCode::E0021, r#""#);
        explanations.insert(CompilationIssueCode::E0022, r#""#);
        explanations.insert(CompilationIssueCode::E0023, r#""#);
        explanations.insert(CompilationIssueCode::E0024, r#""#);
        explanations.insert(CompilationIssueCode::E0025, r#""#);
        explanations.insert(CompilationIssueCode::E0026, r#""#);
        explanations.insert(CompilationIssueCode::E0027, r#""#);
        explanations.insert(CompilationIssueCode::E0028, r#""#);
        explanations.insert(CompilationIssueCode::E0029, r#""#);
        explanations.insert(CompilationIssueCode::E0030, r#""#);
        explanations.insert(CompilationIssueCode::E0031, r#""#);
        explanations.insert(CompilationIssueCode::E0032, r#""#);
        explanations.insert(CompilationIssueCode::E0033, r#""#);

        explanations.insert(CompilationIssueCode::W0001, r#""#);
        explanations.insert(CompilationIssueCode::W0002, r#""#);
        explanations.insert(CompilationIssueCode::W0003, r#""#);
        explanations.insert(CompilationIssueCode::W0004, r#""#);
        explanations.insert(CompilationIssueCode::W0005, r#""#);
        explanations.insert(CompilationIssueCode::W0007, r#""#);
        explanations.insert(CompilationIssueCode::W0008, r#""#);
        explanations.insert(CompilationIssueCode::W0009, r#""#);
        explanations.insert(CompilationIssueCode::W0010, r#""#);
        explanations.insert(CompilationIssueCode::W0011, r#""#);
        explanations.insert(CompilationIssueCode::W0012, r#""#);
        explanations.insert(CompilationIssueCode::W0013, r#""#);
        explanations.insert(CompilationIssueCode::W0014, r#""#);
        explanations.insert(CompilationIssueCode::W0015, r#""#);
        explanations.insert(CompilationIssueCode::W0016, r#""#);
        explanations.insert(CompilationIssueCode::W0017, r#""#);

        explanations
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompilationIssueCode {
    E0001, // Syntax Error.
    E0002, // EOF.
    E0003, // Unknown compiler built-in.
    E0004, // Already declared.
    E0005, // Duplicated global assembler.
    E0006, // Non-constant value
    E0007, // Reference without an address
    E0008, // Value without an address
    E0010, // Possible undefined behavior.
    E0011, // Missing Attribute Error.
    E0012, // Attribute Syntax Error.
    E0013, // Attribute Situation Error.
    E0014, // Unreaceable instruction.
    E0015, // Terminator declared before.
    E0016, // Invalid Scope Position.
    E0017, // Loop Control Flow outside of a loop
    E0018, // Function terminator outside of function.
    E0019, // Type Error
    E0020, // Mismatched Types
    E0021, // Mismatched attributes
    E0022, // Missing call arguments,
    E0023, // Mismatched call arguments
    E0024, // Unsopported call convention.
    E0025, // Unknown Compiler intrinsic.
    E0026, // Too many fields.
    E0027, // Missing fields.
    E0028, // Unknown reference.
    E0029, // Import Error.
    E0030, // Incompatible Operation
    E0031, // Unknown Operation
    E0032, // Incompatible Type Cast
    E0033, // Attribute Conflict

    W0001, // Irrelevant Attribute
    W0002, // Unknown Call Convention
    W0003, // Unknown Linkage
    W0004, // Attribute Conflict,
    W0005, // Local not used
    W0007, // LLI not used
    W0008, // Parameter not used,
    W0009, // Static not used,
    W0010, // Constant no used,
    W0011, // Assembler Function not used
    W0012, // Enum not used,
    W0013, // Enum field not used,
    W0014, // Intrinsic not Used
    W0015, // Strucuture not Used,
    W0016, // Structure Field not Used,
    W0017, // Function not used
}

impl CompilationIssueCode {
    pub fn to_title(self) -> String {
        match self {
            CompilationIssueCode::E0001 => format!("SYNTAX ERROR - {}", "E0001".bright_red()),
            CompilationIssueCode::E0002 => {
                format!("UNEXPECTED END OF FILE - {}", "E0002".bright_red())
            }
            CompilationIssueCode::E0003 => {
                format!("UNKNOWN COMPILER BUILT-IN - {}", "E0003".bright_red())
            }
            CompilationIssueCode::E0004 => format!("ALREADY DEFINED - {}", "E0004".bright_red()),
            CompilationIssueCode::E0005 => {
                format!("DUPLICATED GLOBAL ASSEMBLER - {}", "E0005".bright_red())
            }
            CompilationIssueCode::E0006 => format!("NON-CONSTANT VALUE - {}", "E0006".bright_red()),
            CompilationIssueCode::E0007 => {
                format!("REFERENCE WITHOUT ADDRESS - {}", "E0007".bright_red())
            }
            CompilationIssueCode::E0008 => {
                format!("VALUE WITHOUT ADDRESS - {}", "E0008".bright_red())
            }
            CompilationIssueCode::E0010 => {
                format!("POSSIBLE UNDEFINED BEHAVIOR - {}", "E0010".bright_red())
            }
            CompilationIssueCode::E0011 => format!("MISSING ATTRIBUTE - {}", "E0011".bright_red()),
            CompilationIssueCode::E0012 => {
                format!("ATTRIBUTE SYNTAX ERROR - {}", "E0012".bright_red())
            }
            CompilationIssueCode::E0013 => {
                format!("ATTRIBUTE SITUATION ERROR - {}", "E0013".bright_red())
            }
            CompilationIssueCode::E0014 => {
                format!("UNREACHABLE INSTRUCTION - {}", "E0014".bright_red())
            }
            CompilationIssueCode::E0015 => {
                format!("TERMINATOR ALREADY DECLARED - {}", "E0015".bright_red())
            }
            CompilationIssueCode::E0016 => {
                format!("INVALID SCOPE POSITION - {}", "E0016".bright_red())
            }
            CompilationIssueCode::E0017 => {
                format!("LOOP CONTROL FLOW OUTSIDE LOOP - {}", "E0017".bright_red())
            }
            CompilationIssueCode::E0018 => format!(
                "FUNCTION TERMINATOR OUTSIDE FUNCTION - {}",
                "E0018".bright_red()
            ),
            CompilationIssueCode::E0019 => format!("TYPE ERROR - {}", "E0019".bright_red()),
            CompilationIssueCode::E0020 => format!("MISMATCHED TYPES - {}", "E0020".bright_red()),
            CompilationIssueCode::E0021 => {
                format!("MISMATCHED ATTRIBUTES - {}", "E0021".bright_red())
            }
            CompilationIssueCode::E0022 => {
                format!("MISSING CALL ARGUMENTS - {}", "E0022".bright_red())
            }
            CompilationIssueCode::E0023 => {
                format!("MISMATCHED CALL ARGUMENTS - {}", "E0023".bright_red())
            }
            CompilationIssueCode::E0024 => {
                format!("UNSUPPORTED CALL CONVENTION - {}", "E0024".bright_red())
            }
            CompilationIssueCode::E0025 => {
                format!("UNKNOWN COMPILER INTRINSIC - {}", "E0025".bright_red())
            }
            CompilationIssueCode::E0026 => {
                format!("TOO MANY FIELDS - {}", "E0026".bright_red())
            }
            CompilationIssueCode::E0027 => {
                format!("MISSING FIELDS - {}", "E0027".bright_red())
            }
            CompilationIssueCode::E0028 => {
                format!("UNKNOWN REFERENCE - {}", "E0028".bright_red())
            }
            CompilationIssueCode::E0029 => {
                format!("IMPORT ERROR - {}", "E0029".bright_red())
            }
            CompilationIssueCode::E0030 => {
                format!("INCOMPATIBLE OPERATION - {}", "E0030".bright_red())
            }
            CompilationIssueCode::E0031 => {
                format!("UNKNOWN OPERATION - {}", "E0031".bright_red())
            }
            CompilationIssueCode::E0032 => {
                format!("INCOMPATIBLE TYPE CAST - {}", "E0032".bright_red())
            }
            CompilationIssueCode::E0033 => {
                format!("ATTRIBUTE CONFLICT - {}", "E0033".bright_red())
            }
            CompilationIssueCode::W0001 => {
                format!("IRRELEVANT ATTRIBUTE - {}", "W0001".bright_yellow())
            }
            CompilationIssueCode::W0002 => {
                format!("UNKNOWN CALL CONVENTION - {}", "W0002".bright_yellow())
            }
            CompilationIssueCode::W0003 => format!("UNKNOWN LINKAGE - {}", "W0003".bright_yellow()),
            CompilationIssueCode::W0004 => {
                format!("ATTRIBUTE CONFLICT - {}", "W0004".bright_yellow())
            }
            CompilationIssueCode::W0005 => {
                format!("UNUSED LOCAL VARIABLE - {}", "W0005".bright_yellow())
            }
            CompilationIssueCode::W0007 => format!("UNUSED LLI - {}", "W0007".bright_yellow()),
            CompilationIssueCode::W0008 => {
                format!("UNUSED PARAMETER - {}", "W0008".bright_yellow())
            }
            CompilationIssueCode::W0009 => format!("UNUSED STATIC - {}", "W0009".bright_yellow()),
            CompilationIssueCode::W0010 => format!("UNUSED CONSTANT - {}", "W0010".bright_yellow()),
            CompilationIssueCode::W0011 => {
                format!("UNUSED ASSEMBLER FUNCTION - {}", "W0011".bright_yellow())
            }
            CompilationIssueCode::W0012 => format!("UNUSED ENUM - {}", "W0012".bright_yellow()),
            CompilationIssueCode::W0013 => {
                format!("UNUSED ENUM FIELD - {}", "W0013".bright_yellow())
            }
            CompilationIssueCode::W0014 => {
                format!("UNUSED INTRINSIC - {}", "W0014".bright_yellow())
            }
            CompilationIssueCode::W0015 => {
                format!("UNUSED STRUCTURE - {}", "W0015".bright_yellow())
            }
            CompilationIssueCode::W0016 => {
                format!("UNUSED STRUCTURE FIELD - {}", "W0016".bright_yellow())
            }
            CompilationIssueCode::W0017 => {
                format!("UNUSED FUNCTION - {}", "W0017".bright_yellow())
            }
        }
    }

    pub fn get_explanation(&self) -> &str {
        COMPILATION_ISSUE_CODE_EXPLANATIONS
            .get(self)
            .unwrap_or_else(|| {
                logging::print_warn(
                    logging::LoggingType::Warning,
                    "Unable to get the properly issue explanation.",
                );

                &""
            })
    }
}

impl From<&str> for CompilationIssueCode {
    fn from(n: &str) -> Self {
        match n {
            "E0001" => CompilationIssueCode::E0001,
            "E0002" => CompilationIssueCode::E0002,
            "E0003" => CompilationIssueCode::E0003,
            "E0004" => CompilationIssueCode::E0004,
            "E0005" => CompilationIssueCode::E0005,
            "E0006" => CompilationIssueCode::E0006,
            "E0007" => CompilationIssueCode::E0007,
            "E0008" => CompilationIssueCode::E0008,
            "E0010" => CompilationIssueCode::E0010,
            "E0011" => CompilationIssueCode::E0011,
            "E0012" => CompilationIssueCode::E0012,
            "E0013" => CompilationIssueCode::E0013,
            "E0014" => CompilationIssueCode::E0014,
            "E0015" => CompilationIssueCode::E0015,
            "E0016" => CompilationIssueCode::E0016,
            "E0017" => CompilationIssueCode::E0017,
            "E0018" => CompilationIssueCode::E0018,
            "E0019" => CompilationIssueCode::E0019,
            "E0020" => CompilationIssueCode::E0020,
            "E0021" => CompilationIssueCode::E0021,
            "E0022" => CompilationIssueCode::E0022,
            "E0023" => CompilationIssueCode::E0023,
            "E0024" => CompilationIssueCode::E0024,
            "E0025" => CompilationIssueCode::E0025,
            "E0026" => CompilationIssueCode::E0026,
            "E0027" => CompilationIssueCode::E0027,
            "E0028" => CompilationIssueCode::E0028,
            "E0029" => CompilationIssueCode::E0029,
            "E0030" => CompilationIssueCode::E0030,
            "E0031" => CompilationIssueCode::E0031,
            "E0032" => CompilationIssueCode::E0032,
            "E0033" => CompilationIssueCode::E0033,

            "W0001" => CompilationIssueCode::W0001,
            "W0002" => CompilationIssueCode::W0002,
            "W0003" => CompilationIssueCode::W0003,
            "W0004" => CompilationIssueCode::W0004,
            "W0005" => CompilationIssueCode::W0005,
            "W0007" => CompilationIssueCode::W0007,
            "W0008" => CompilationIssueCode::W0008,
            "W0009" => CompilationIssueCode::W0009,
            "W0010" => CompilationIssueCode::W0010,
            "W0011" => CompilationIssueCode::W0011,
            "W0012" => CompilationIssueCode::W0012,
            "W0013" => CompilationIssueCode::W0013,
            "W0014" => CompilationIssueCode::W0014,
            "W0015" => CompilationIssueCode::W0015,
            "W0016" => CompilationIssueCode::W0016,
            "W0017" => CompilationIssueCode::W0017,

            unknown => logging::print_critical_error(
                logging::LoggingType::Error,
                &format!(
                    "Unknown '{}' as valid issue code. Try again with another.",
                    unknown
                ),
            ),
        }
    }
}
