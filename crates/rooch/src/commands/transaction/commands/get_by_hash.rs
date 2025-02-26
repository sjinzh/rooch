// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::types::{CommandAction, WalletContextOptions};
use async_trait::async_trait;
use rooch_server::jsonrpc_types::TransactionView;
use rooch_types::{error::RoochResult, H256};

#[derive(Debug, clap::Parser)]
pub struct GetByHashCommand {
    #[clap(long)]
    pub hash: H256,

    // filter?
    // pub options:...
    #[clap(flatten)]
    pub(crate) context_options: WalletContextOptions,
}

#[async_trait]
impl CommandAction<Option<TransactionView>> for GetByHashCommand {
    async fn execute(self) -> RoochResult<Option<TransactionView>> {
        let client = self.context_options.build().await?.get_client().await?;

        let resp = client.get_transaction_by_hash(self.hash).await?;

        Ok(resp)
    }
}
