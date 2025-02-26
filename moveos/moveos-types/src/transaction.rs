// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    event::Event, h256, h256::H256, move_types::FunctionId, state::StateChangeSet,
    tx_context::TxContext,
};
use move_core_types::{
    account_address::AccountAddress,
    effects::ChangeSet,
    language_storage::{ModuleId, TypeTag},
    vm_status::KeptVMStatus,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Call a Move script
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptCall {
    #[serde(with = "serde_bytes")]
    pub code: Vec<u8>,
    pub ty_args: Vec<TypeTag>,
    //TOOD custom serialize
    pub args: Vec<Vec<u8>>,
}

/// Call a Move function
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub function_id: FunctionId,
    pub ty_args: Vec<TypeTag>,
    //TOOD custom serialize
    pub args: Vec<Vec<u8>>,
}

impl FunctionCall {
    pub fn new(function_id: FunctionId, ty_args: Vec<TypeTag>, args: Vec<Vec<u8>>) -> Self {
        Self {
            function_id,
            ty_args,
            args,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MoveAction {
    //Execute a Move script
    Script(ScriptCall),
    //Execute a Move function
    Function(FunctionCall),
    //Publish Move modules
    ModuleBundle(Vec<Vec<u8>>),
}

impl MoveAction {
    pub fn new_module_bundle(modules: Vec<Vec<u8>>) -> Self {
        Self::ModuleBundle(modules)
    }
    pub fn new_function_call(
        function_id: FunctionId,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Self {
        Self::Function(FunctionCall {
            function_id,
            ty_args,
            args,
        })
    }
    pub fn new_script_call(code: Vec<u8>, ty_args: Vec<TypeTag>, args: Vec<Vec<u8>>) -> Self {
        Self::Script(ScriptCall {
            code,
            ty_args,
            args,
        })
    }
}

/// The MoveAction after verifier
#[derive(Clone, Debug)]
pub enum VerifiedMoveAction {
    Script {
        call: ScriptCall,
        resolved_args: Vec<Vec<u8>>,
    },
    Function {
        call: FunctionCall,
        resolved_args: Vec<Vec<u8>>,
    },
    ModuleBundle {
        module_bundle: Vec<Vec<u8>>,
        init_function_modules: Vec<ModuleId>,
    },
}

impl Display for VerifiedMoveAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifiedMoveAction::Script {
                call: _,
                resolved_args: _,
            } => {
                write!(f, "ScriptCall")
            }
            VerifiedMoveAction::Function {
                call,
                resolved_args: _,
            } => {
                write!(f, "FunctionCall(function_id: {})", call.function_id)
            }
            VerifiedMoveAction::ModuleBundle {
                module_bundle,
                init_function_modules,
            } => {
                write!(
                    f,
                    "ModuleBundle(module_bundle: {}, init_function_modules: {})",
                    module_bundle.len(),
                    init_function_modules.len()
                )
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MoveOSTransaction {
    pub ctx: TxContext,
    pub action: MoveAction,
}

impl MoveOSTransaction {
    /// Create a new MoveOS transaction
    /// This function only for test case usage
    pub fn new_for_test(sender: AccountAddress, action: MoveAction) -> Self {
        let sender_and_action = (sender, action);
        let tx_hash = h256::sha3_256_of(bcs::to_bytes(&sender_and_action).unwrap().as_slice());
        let ctx = TxContext::new(sender_and_action.0, tx_hash);
        Self {
            ctx,
            action: sender_and_action.1,
        }
    }

    pub fn new(ctx: TxContext, action: MoveAction) -> Self {
        Self { ctx, action }
    }
}

#[derive(Debug, Clone)]
pub struct VerifiedMoveOSTransaction {
    pub ctx: TxContext,
    pub action: VerifiedMoveAction,
}

/// TransactionOutput is the execution result of a MoveOS transaction
//TODO make TransactionOutput serializable
#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub status: KeptVMStatus,
    pub changeset: ChangeSet,
    pub state_changeset: StateChangeSet,
    pub events: Vec<Event>,
    pub gas_used: u64,
}

/// `TransactionExecutionInfo` represents the result of executing a transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionExecutionInfo {
    /// The hash of this transaction.
    pub tx_hash: H256,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root: H256,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root: H256,

    /// The amount of gas used.
    pub gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: KeptVMStatus,
}

impl TransactionExecutionInfo {
    pub fn new(
        tx_hash: H256,
        state_root: H256,
        event_root: H256,
        gas_used: u64,
        status: KeptVMStatus,
    ) -> TransactionExecutionInfo {
        TransactionExecutionInfo {
            tx_hash,
            state_root,
            event_root,
            gas_used,
            status,
        }
    }

    pub fn id(&self) -> H256 {
        h256::sha3_256_of(bcs::to_bytes(self).unwrap().as_slice())
    }
}
