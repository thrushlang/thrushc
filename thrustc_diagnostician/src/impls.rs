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

use colored::Colorize;

use crate::Notificator;

impl std::fmt::Display for Notificator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Notificator::Error => write!(f, "{}: ", "ERROR".bright_red().bold().underline()),
            Notificator::Warning => write!(f, "{}: ", "WARNING".bright_yellow().bold().underline()),
            Notificator::CompilerFrontendBug | Self::CompilerBackendBug => {
                write!(f, "{}: ", "INFO".bright_green().bold().underline())
            }
        }
    }
}
