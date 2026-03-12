use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

macro_rules! string_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

string_id_type!(Cid);
string_id_type!(Hash);
string_id_type!(Signature);
string_id_type!(CaseId);
string_id_type!(NodeId);
string_id_type!(PointerAlias);
string_id_type!(ReceiptCid);
string_id_type!(ProofPackCid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BudgetAmount(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReasonCode(pub u32);

impl ReasonCode {
    pub const NONE: Self = Self(0);
    pub const MISSING_EVIDENCE: Self = Self(0x01);
    pub const UNANCHORED: Self = Self(0x02);
    pub const RESOURCE_VIOLATION: Self = Self(0x04);
    pub const ZERO_GUESS: Self = Self(0x08);
    pub const SILICON_DOUBT: Self = Self(0x10);
    pub const SILICON_NOT_OK: Self = Self(0x20);
}

pub trait Canonical {
    fn canonical(&self) -> String;
}

pub fn canonical_join(parts: &[&str]) -> String {
    parts.join("|")
}

pub fn hash_canonical(parts: &[&str]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(canonical_join(parts));
    Hash::new(hex::encode(hasher.finalize()))
}
