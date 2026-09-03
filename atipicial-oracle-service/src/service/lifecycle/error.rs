use super::super::providers::OracleContractReadProvider;
use super::super::{OracleRuntimeProvider, OracleService, OracleServiceError};
use atipicial_execution::native_contract_provider::NativeContractProvider;
use tracing::error;

impl<R, P> OracleService<R, P>
where
    R: OracleRuntimeProvider + 'static,
    P: NativeContractProvider + OracleContractReadProvider + 'static,
{
    pub(crate) fn handle_error(&self, err: &OracleServiceError) {
        error!(target: "atipicial::oracle", error = %err, "oracle service error");
        self.settings.exception_policy.apply(|| self.stop());
    }
}
