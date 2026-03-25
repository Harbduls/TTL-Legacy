use soroban_sdk::{contracttype, symbol_short, Address, Symbol};

pub const RELEASE_TOPIC: Symbol = symbol_short!("release");

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Vault(u64),
    VaultCount,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ReleaseStatus {
    Locked,
    Released,
}

#[contracttype]
#[derive(Clone)]
pub struct ReleaseEvent {
    pub vault_id: u64,
    pub beneficiary: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct Vault {
    pub owner: Address,
    pub beneficiary: Address,
    pub balance: i128,
    pub check_in_interval: u64, // seconds
    pub last_check_in: u64,     // ledger timestamp
    pub status: ReleaseStatus,
}
