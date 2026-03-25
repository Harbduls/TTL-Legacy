#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    vec, Address, Env, IntoVal,
};
use types::{ReleaseEvent, RELEASE_TOPIC};

fn setup() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    (env, owner, beneficiary)
}

#[test]
fn test_create_vault() {
    let (env, owner, beneficiary) = setup();
    let client = TtlVaultContractClient::new(&env, &env.register_contract(None, TtlVaultContract));

    let vault_id = client.create_vault(&owner, &beneficiary, &86400u64);
    assert_eq!(vault_id, 1);

    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.owner, owner);
    assert_eq!(vault.beneficiary, beneficiary);
    assert_eq!(vault.balance, 0);
}

#[test]
fn test_check_in_resets_timer() {
    let (env, owner, beneficiary) = setup();
    let client = TtlVaultContractClient::new(&env, &env.register_contract(None, TtlVaultContract));

    let vault_id = client.create_vault(&owner, &beneficiary, &86400u64);

    // Advance time by 12 hours
    env.ledger().with_mut(|l| l.timestamp += 43200);
    client.check_in(&vault_id);

    // TTL remaining should be close to full interval again
    let remaining = client.get_ttl_remaining(&vault_id);
    assert!(remaining > 43000 && remaining <= 86400);
}

#[test]
fn test_is_not_expired_before_interval() {
    let (env, owner, beneficiary) = setup();
    let client = TtlVaultContractClient::new(&env, &env.register_contract(None, TtlVaultContract));

    let vault_id = client.create_vault(&owner, &beneficiary, &86400u64);
    env.ledger().with_mut(|l| l.timestamp += 43200);

    assert!(!client.is_expired(&vault_id));
}

#[test]
fn test_is_expired_after_interval() {
    let (env, owner, beneficiary) = setup();
    let client = TtlVaultContractClient::new(&env, &env.register_contract(None, TtlVaultContract));

    let vault_id = client.create_vault(&owner, &beneficiary, &86400u64);
    env.ledger().with_mut(|l| l.timestamp += 90000); // past 24h

    assert!(client.is_expired(&vault_id));
}

#[test]
fn test_update_beneficiary() {
    let (env, owner, beneficiary) = setup();
    let client = TtlVaultContractClient::new(&env, &env.register_contract(None, TtlVaultContract));

    let vault_id = client.create_vault(&owner, &beneficiary, &86400u64);
    let new_beneficiary = Address::generate(&env);
    client.update_beneficiary(&vault_id, &new_beneficiary);

    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.beneficiary, new_beneficiary);
}

#[test]
fn test_trigger_release_emits_event() {
    let (env, owner, beneficiary) = setup();
    let contract_id = env.register_contract(None, TtlVaultContract);
    let client = TtlVaultContractClient::new(&env, &contract_id);

    let vault_id = client.create_vault(&owner, &beneficiary, &86400u64);

    // Advance past the check-in interval
    env.ledger().with_mut(|l| l.timestamp += 90000);

    client.trigger_release(&vault_id);

    let events = env.events().all();
    assert_eq!(events.len(), 1);

    let (emitted_contract, topics, data) = events.get(0).unwrap();
    assert_eq!(emitted_contract, contract_id);
    assert_eq!(topics, vec![&env, RELEASE_TOPIC.into_val(&env)]);

    let event: ReleaseEvent = data.into_val(&env);
    assert_eq!(event.vault_id, vault_id);
    assert_eq!(event.beneficiary, beneficiary);
    assert_eq!(event.amount, 0); // no deposit was made
}
