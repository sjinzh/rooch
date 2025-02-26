// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    addresses::MOVEOS_STD_ADDRESS, object::ObjectID, state_resolver, tx_context::TxContext,
};
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::IdentStr, move_resource::MoveStructType,
};
use serde::{Deserialize, Serialize};

pub const GLOBAL_OBJECT_STORAGE_HANDLE: ObjectID = state_resolver::GLOBAL_OBJECT_STORAGE_HANDLE;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObjectStorage {
    pub handle: ObjectID,
}

pub const STORAGE_CONTEXT_MODULE_NAME: &IdentStr = ident_str!("storage_context");
pub const STORAGE_CONTEXT_STRUCT_NAME: &IdentStr = ident_str!("StorageContext");

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StorageContext {
    pub tx_context: TxContext,
    pub object_storage: ObjectStorage,
}

impl StorageContext {
    /// New global storage context
    pub fn new(tx_context: TxContext) -> Self {
        Self {
            tx_context,
            object_storage: ObjectStorage {
                handle: GLOBAL_OBJECT_STORAGE_HANDLE,
            },
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        bcs::to_bytes(&self).unwrap()
    }
}

impl MoveStructType for StorageContext {
    const ADDRESS: AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = STORAGE_CONTEXT_MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = STORAGE_CONTEXT_STRUCT_NAME;
}
