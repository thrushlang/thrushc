use crate::frontend::semantic::linter::Linter;

pub fn mark_as_used<'linter>(linter: &mut Linter<'linter>, name: &'linter str) {
    if let Some(local) = linter.symbols.get_local_info(name) {
        local.1 = true;
        return;
    }

    if let Some(parameter) = linter.symbols.get_parameter_info(name) {
        parameter.1 = true;
        return;
    }

    if let Some(lli) = linter.symbols.get_lli_info(name) {
        lli.1 = true;
    }

    if let Some(constant) = linter.symbols.get_constant_info(name) {
        constant.1 = true;
    }
}
