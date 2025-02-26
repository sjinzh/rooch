// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::crypto::{BuiltinScheme, Signature};
use anyhow::Result;

#[cfg(any(test, feature = "fuzzing"))]
use ethers::types::U256;
#[cfg(any(test, feature = "fuzzing"))]
use fastcrypto::ed25519::Ed25519KeyPair;
#[cfg(any(test, feature = "fuzzing"))]
use fastcrypto::traits::KeyPair;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
#[cfg(any(test, feature = "fuzzing"))]
use rand::{rngs::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};

/// A `Authenticator` is an an abstraction of a account authenticator.
/// It is a part of `AccountAbstraction`

pub trait BuiltinAuthenticator {
    fn scheme(&self) -> BuiltinScheme;
    fn encode(&self) -> Vec<u8>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519Authenticator {
    pub signature: Signature,
}

impl BuiltinAuthenticator for Ed25519Authenticator {
    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::Ed25519
    }
    fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode should success.")
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for Ed25519Authenticator {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_ed25519_authenticator().boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_ed25519_authenticator()(
        seed in any::<u64>(),
        message in vec(any::<u8>(), 1..1000)
    ) -> Ed25519Authenticator {
        let mut rng = StdRng::seed_from_u64(seed);
        let ed25519_keypair: Ed25519KeyPair = Ed25519KeyPair::generate(&mut rng);
        Ed25519Authenticator {
            signature: Signature::new_hashed(&message, &ed25519_keypair)
        }
    }
}

// TODO: MultiEd25519

#[derive(Clone, Debug)]
pub struct Secp256k1Authenticator {
    pub signature: ethers::core::types::Signature,
}

impl BuiltinAuthenticator for Secp256k1Authenticator {
    fn scheme(&self) -> BuiltinScheme {
        BuiltinScheme::Secp256k1
    }
    fn encode(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("encode should success.")
    }
}

impl Serialize for Secp256k1Authenticator {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(::serde::Serialize)]
        #[serde(rename = "Secp256k1Authenticator")]
        struct Value {
            signature: Vec<u8>,
        }
        Value {
            signature: self.signature.to_vec(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Secp256k1Authenticator {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(::serde::Deserialize)]
        #[serde(rename = "Secp256k1Authenticator")]
        struct Value {
            signature: Vec<u8>,
        }
        let value = Value::deserialize(deserializer)?;
        let signature = ethers::core::types::Signature::try_from(value.signature.as_slice())
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(Secp256k1Authenticator { signature })
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for Secp256k1Authenticator {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        arb_secp256k1_authenticator().boxed()
    }
    type Strategy = BoxedStrategy<Self>;
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn arb_secp256k1_authenticator()(
     r in vec(any::<u64>(), 4..=4).prop_map(|v| U256(v.try_into().unwrap())),
     s in vec(any::<u64>(), 4..=4).prop_map(|v| U256(v.try_into().unwrap())),
     // Although v is an u64 type, it is actually an u8 value.
     v in any::<u8>().prop_map(<u64>::from),
    ) -> Secp256k1Authenticator {
        Secp256k1Authenticator {
            signature: ethers::core::types::Signature {r, s, v},
        }
    }
}

impl<T> From<T> for Authenticator
where
    T: BuiltinAuthenticator,
{
    fn from(value: T) -> Self {
        let scheme = value.scheme() as u64;
        let payload = value.encode();
        Authenticator { scheme, payload }
    }
}

impl From<Signature> for Authenticator {
    fn from(sign: Signature) -> Self {
        match sign.to_public_key().unwrap().scheme() {
            BuiltinScheme::Ed25519 => Authenticator::ed25519(sign),
            BuiltinScheme::Secp256k1 => todo!(),
            BuiltinScheme::MultiEd25519 => todo!(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Authenticator {
    pub scheme: u64,
    pub payload: Vec<u8>,
}

impl Authenticator {
    /// Unique identifier for the signature scheme
    pub fn scheme(&self) -> u64 {
        self.scheme
    }

    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(signature: Signature) -> Self {
        Ed25519Authenticator { signature }.into()
    }

    /// Create a single-signature secp256k1 authenticator
    pub fn secp256k1(signature: ethers::core::types::Signature) -> Self {
        Secp256k1Authenticator { signature }.into()
    }

    /// Create a custom authenticator
    pub fn new(scheme: u64, payload: Vec<u8>) -> Self {
        Self { scheme, payload }
    }
}

impl FromStr for Authenticator {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(s)?;
        bcs::from_bytes(bytes.as_slice()).map_err(Into::into)
    }
}

impl fmt::Display for Authenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Authenticator[scheme id: {:?}, payload: {}]",
            self.scheme(),
            hex::encode(&self.payload),
        )
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_secp256k1_authenticator_serialize_deserialize(authenticator in any::<super::Secp256k1Authenticator>()) {
            let serialized = serde_json::to_string(&authenticator).unwrap();
            let deserialized: super:: Secp256k1Authenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(authenticator.signature, deserialized.signature);
        }

        #[test]
        fn test_ed25519_authenticator_serialize_deserialize(authenticator in any::<super::Ed25519Authenticator>()) {
            let serialized = serde_json::to_string(&authenticator).unwrap();
            let deserialized: super::Ed25519Authenticator = serde_json::from_str(&serialized).unwrap();
            assert_eq!(authenticator.signature, deserialized.signature);
        }
    }
}
