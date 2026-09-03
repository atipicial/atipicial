use super::super::super::{AtipicialFsAuth, AtipicialFsCommand, AtipicialFsRequest, OracleAtipicialFsProtocol};
use super::super::client::atipicialfs_grpc_client;
use atipicial_payloads::OracleResponseCode;
use atipicial_payloads::oracle_response::MAX_RESULT_SIZE;
use atipicial_wallets::KeyPair;

impl OracleAtipicialFsProtocol {
    pub(in super::super::super) async fn execute_grpc_request(
        &self,
        endpoint: &str,
        request: AtipicialFsRequest,
        auth: &AtipicialFsAuth,
        oracle_key: &KeyPair,
    ) -> (OracleResponseCode, String) {
        let address = match request.build_atipicialfs_grpc_address() {
            Ok(address) => address,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };
        let mut client = match atipicialfs_grpc_client(endpoint).await {
            Ok(client) => client,
            Err(_) => return (OracleResponseCode::Error, String::new()),
        };

        match request.command {
            AtipicialFsCommand::Payload => {
                self.fetch_payload_grpc(&mut client, &address, auth, oracle_key)
                    .await
            }
            AtipicialFsCommand::Header => {
                self.fetch_header_grpc(&mut client, &address, auth, oracle_key)
                    .await
            }
            AtipicialFsCommand::Range(range) => {
                if range.length > MAX_RESULT_SIZE as u64 {
                    return (OracleResponseCode::ResponseTooLarge, String::new());
                }
                self.fetch_range_grpc(&mut client, &address, range, auth, oracle_key)
                    .await
            }
            AtipicialFsCommand::Hash(range) => {
                self.fetch_hash_grpc(&mut client, &address, range, auth, oracle_key)
                    .await
            }
        }
    }
}
