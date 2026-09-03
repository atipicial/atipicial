use super::super::proto::atipicialfs_v2;
use std::time::Duration;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};

const ATCFS_GRPC_CONNECT_TIMEOUT: Duration = Duration::from_millis(120_000);

pub(super) async fn atipicialfs_grpc_client(
    endpoint: &str,
) -> Result<atipicialfs_v2::object::object_service_client::ObjectServiceClient<Channel>, String> {
    let mut builder = Endpoint::from_shared(endpoint.to_string())
        .map_err(|err| format!("invalid atipicialfs endpoint: {err}"))?
        .connect_timeout(ATCFS_GRPC_CONNECT_TIMEOUT);
    if endpoint.to_ascii_lowercase().starts_with("https://") {
        builder = builder
            .tls_config(ClientTlsConfig::new())
            .map_err(|err| format!("invalid atipicialfs tls config: {err}"))?;
    }
    let channel = builder
        .connect()
        .await
        .map_err(|err| format!("atipicialfs grpc connect failed: {err}"))?;
    Ok(atipicialfs_v2::object::object_service_client::ObjectServiceClient::new(channel))
}
