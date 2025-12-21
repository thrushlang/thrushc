use colored::Colorize;

use super::position::CompilationPosition;
use crate::core::diagnostic::span::Span;

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

#[derive(Debug, Clone, Copy)]
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
                format!("UNKNOWN REFERENCE - {}", "E0029".bright_red())
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
}
