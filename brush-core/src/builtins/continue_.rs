use clap::Parser;

use crate::builtin::{BuiltinCommand, BuiltinExitCode};

/// Continue to the next iteration of a control-flow loop.
#[derive(Parser)]
pub(crate) struct ContinueCommand {
    /// If specified, indicates which nested loop to continue to the next iteration of.
    #[clap(default_value = "1")]
    which_loop: i8,
}

#[async_trait::async_trait]
impl BuiltinCommand for ContinueCommand {
    async fn execute(
        &self,
        _context: crate::context::CommandExecutionContext<'_>,
    ) -> Result<crate::builtin::BuiltinExitCode, crate::error::Error> {
        // If specified, which_loop needs to be positive.
        if self.which_loop <= 0 {
            return Ok(BuiltinExitCode::InvalidUsage);
        }

        #[allow(clippy::cast_sign_loss)]
        Ok(BuiltinExitCode::ContinueLoop((self.which_loop - 1) as u8))
    }
}