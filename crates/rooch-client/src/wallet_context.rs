use crate::client_config::{ClientConfig, DEFAULT_EXPIRATION_SECS};
use crate::Client;
use anyhow::anyhow;
use move_core_types::account_address::AccountAddress;
use moveos_types::transaction::MoveAction;
use rooch_config::{rooch_config_dir, Config, PersistedConfig, ROOCH_CLIENT_CONFIG};
use rooch_key::keystore::AccountKeystore;
use rooch_server::jsonrpc_types::ExecuteTransactionResponseView;
use rooch_types::address::RoochAddress;
use rooch_types::crypto::{BuiltinScheme, Signature};
use rooch_types::error::{RoochError, RoochResult};
use rooch_types::transaction::{
    authenticator::Authenticator,
    rooch::{RoochTransaction, RoochTransactionData},
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct WalletContext {
    client: Arc<RwLock<Option<Client>>>,
    pub config: PersistedConfig<ClientConfig>,
}

impl WalletContext {
    pub async fn new(config_path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let config_dir = config_path.unwrap_or(rooch_config_dir()?);
        let config_path = config_dir.join(ROOCH_CLIENT_CONFIG);
        let config: ClientConfig = PersistedConfig::read(&config_path).map_err(|err| {
            anyhow!(
                "Cannot open wallet config file at {:?}. Err: {err}, Use `rooch init` to configuration",
                config_path
            )
        })?;

        let config = config.persisted(&config_path);
        Ok(Self {
            client: Default::default(),
            config,
        })
    }

    pub fn parse_account_arg(&self, arg: String) -> Result<AccountAddress, RoochError> {
        self.parse(arg)
    }

    pub fn parse_account_args(
        &self,
        args: BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, AccountAddress>, RoochError> {
        Ok(args
            .into_iter()
            .map(|(key, value)| (key, self.parse(value).unwrap()))
            .collect())
    }

    pub async fn get_client(&self) -> Result<Client, anyhow::Error> {
        // TODO: Check version

        let read = self.client.read().await;

        Ok(if let Some(client) = read.as_ref() {
            client.clone()
        } else {
            drop(read);
            let client = self
                .config
                .get_active_env()?
                .create_rpc_client(Duration::from_secs(DEFAULT_EXPIRATION_SECS), None)
                .await?;

            self.client.write().await.insert(client).clone()
        })
    }

    pub async fn sign_and_execute(
        &self,
        sender: RoochAddress,
        action: MoveAction,
    ) -> RoochResult<ExecuteTransactionResponseView> {
        let pk = self.config.keystore.get_key(&sender).ok().ok_or_else(|| {
            RoochError::SignMessageError(format!("Cannot find key for address: [{sender}]"))
        })?;

        let client = self.get_client().await?;

        let sequence_number = client
            .get_sequence_number(sender)
            .await
            .map_err(RoochError::from)?;
        log::debug!("use sequence_number: {}", sequence_number);
        let tx_data = RoochTransactionData::new(sender, sequence_number, action);
        let signature = Signature::new_hashed(tx_data.hash().as_bytes(), pk);
        let auth = match pk.public().scheme() {
            BuiltinScheme::Ed25519 => Authenticator::ed25519(signature),
            BuiltinScheme::Secp256k1 => todo!(),
            BuiltinScheme::MultiEd25519 => todo!(),
        };

        client
            .execute_tx(RoochTransaction::new(tx_data, auth))
            .await
            .map_err(|e| RoochError::TransactionError(e.to_string()))
    }

    fn parse(&self, account: String) -> Result<AccountAddress, RoochError> {
        if account.starts_with("0x") {
            AccountAddress::from_hex_literal(&account).map_err(|err| {
                RoochError::CommandArgumentError(format!("Failed to parse AccountAddress {}", err))
            })
        } else if let Ok(account_address) = AccountAddress::from_str(&account) {
            Ok(account_address)
        } else {
            let address = match account.as_str() {
                "default" => AccountAddress::from(self.config.active_address.unwrap()),
                _ => Err(RoochError::CommandArgumentError(
                    "Use rooch init configuration".to_string(),
                ))?,
            };

            Ok(address)
        }
    }
}
