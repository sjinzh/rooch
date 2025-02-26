// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{authenticator::Authenticator, AbstractTransaction, AuthenticatorInfo};
use crate::address::EthereumAddress;
use anyhow::Result;
use ethers::utils::rlp::{Decodable, Rlp};
use move_core_types::account_address::AccountAddress;
use moveos_types::{
    h256::H256,
    transaction::{MoveAction, MoveOSTransaction},
    tx_context::TxContext,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EthereumTransaction(pub ethers::core::types::Transaction);

impl EthereumTransaction {
    //This function is just a demo, we should define the Ethereum calldata's MoveAction standard
    pub fn decode_calldata_to_action(&self) -> Result<MoveAction> {
        //Maybe we should use RLP to encode the MoveAction
        bcs::from_bytes(&self.0.input)
            .map_err(|e| anyhow::anyhow!("decode calldata to action failed: {}", e))
    }
}

impl AbstractTransaction for EthereumTransaction {
    fn transaction_type(&self) -> super::TransactionType {
        super::TransactionType::Ethereum
    }

    fn decode(bytes: &[u8]) -> Result<Self> {
        let rlp = Rlp::new(bytes);
        let mut tx = ethers::core::types::Transaction::decode(&rlp)?;
        tx.recover_from_mut()?;
        Ok(Self(tx))
    }

    fn encode(&self) -> Vec<u8> {
        self.0.rlp().to_vec()
    }

    fn tx_hash(&self) -> H256 {
        self.0.hash()
    }

    fn authenticator_info(&self) -> AuthenticatorInfo {
        AuthenticatorInfo {
            //TODO should change the seqence_number to u256?
            seqence_number: self.0.nonce.as_u64(),
            authenticator: Authenticator::secp256k1(ethers::core::types::Signature {
                r: self.0.r,
                s: self.0.s,
                v: self.0.v.as_u64(),
            }),
        }
    }

    fn construct_moveos_transaction(
        self,
        resolved_sender: AccountAddress,
    ) -> Result<MoveOSTransaction> {
        let action = self.decode_calldata_to_action()?;
        let tx_ctx = TxContext::new(resolved_sender, self.tx_hash());
        Ok(MoveOSTransaction::new(tx_ctx, action))
    }

    fn sender(&self) -> crate::address::MultiChainAddress {
        EthereumAddress(self.0.from).into()
    }
}
