// Origin source https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/sources/tx_context.move#L24
// And do refactoring

module moveos_std::tx_context {
    use std::vector;
    use std::bcs;
    use std::hash;
    use moveos_std::bcd;
    use moveos_std::object_id::{Self, ObjectID};

    friend moveos_std::object;
    friend moveos_std::raw_table;
    friend moveos_std::account_storage;
    friend moveos_std::events;

    /// Number of bytes in an tx hash (which will be the transaction digest)
    const TX_HASH_LENGTH: u64 = 32;

    /// Expected an tx hash of length 32, but found a different length
    const EBadTxHashLength: u64 = 0;


    /// Information about the transaction currently being executed.
    /// This cannot be constructed by a transaction--it is a privileged object created by
    /// the VM and passed in to the entrypoint of the transaction as `&mut TxContext`.
    struct TxContext has drop {
        /// The address of the user that signed the current transaction
        sender: address,
        /// Hash of the current transaction
        tx_hash: vector<u8>,
        /// Counter recording the number of fresh id's created while executing
        /// this transaction. Always 0 at the start of a transaction
        ids_created: u64,
    }

    /// Return the address of the user that signed the current
    /// transaction
    public fun sender(self: &TxContext): address {
        self.sender
    } 

    /// Generate a new unique address,
    public fun fresh_address(ctx: &mut TxContext): address {
        let addr = derive_id(ctx.tx_hash, ctx.ids_created);
        ctx.ids_created = ctx.ids_created + 1;
        addr
    }

    /// Generate a new unique object ID
    public fun fresh_object_id(ctx: &mut TxContext): ObjectID {
        object_id::address_to_object_id(fresh_address(ctx))
    }

    public(friend) fun derive_id(hash: vector<u8>, index: u64): address {
        let bytes = hash;
        vector::append(&mut bytes, bcs::to_bytes(&index));
        //TODO change return type to h256 and use h256 to replace address?
        let id = hash::sha3_256(bytes);
        bcd::to_address(id)
    }

    /// Return the hash of the current transaction
    public fun tx_hash(self: &TxContext): vector<u8> {
        self.tx_hash
    }

    /// Return the number of id's created by the current transaction.
    /// Hidden for now, but may expose later
    fun ids_created(self: &TxContext): u64 {
        self.ids_created
    }

    
    #[test_only]
    /// Create a TxContext for unit test
    public fun new_test_context(sender: address): TxContext {
        let tx_hash = hash::sha3_256(b"test_tx");
        TxContext {
            sender,
            tx_hash,
            ids_created: 0,
        }
    }
}
