use multiversx_sc::types::Address;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use reputation_score::*;

const WASM_PATH: &str = "output/reputation-score.wasm";

struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> reputation_score::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub oracle_address: Address,
    pub user_address: Address,
    pub contract_wrapper: ContractObjWrapper<reputation_score::ContractObj<DebugApi>, ContractObjBuilder>,
}

fn setup_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> reputation_score::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let oracle_address = blockchain_wrapper.create_user_account(&rust_zero);
    let user_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    // Deploy contract
    let contract_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        builder,
        WASM_PATH,
    );
    
    // Initialize contract
    blockchain_wrapper
        .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
            sc.init(0u64, 1000u64);
        })
        .assert_ok();
    
    // Set oracle address
    blockchain_wrapper
        .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
            sc.set_oracle_address(managed_address!(&oracle_address));
        })
        .assert_ok();
    
    ContractSetup {
        blockchain_wrapper,
        oracle_address,
        user_address,
        contract_wrapper,
    }
}

#[test]
fn test_init() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Verify initial state
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.min_score().get(), 0u64);
            assert_eq!(sc.max_score().get(), 1000u64);
        })
        .assert_ok();
}

#[test]
fn test_set_oracle_address() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Verify oracle address was set correctly
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.oracle_address().get(), managed_address!(&setup.oracle_address));
        })
        .assert_ok();
    
    // Test that non-owner cannot change oracle address
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_oracle_address(managed_address!(&setup.user_address));
        })
        .assert_error(4, "Endpoint can only be called by owner");
}

#[test]
fn test_update_score() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Test that oracle can update score
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 750u64);
        })
        .assert_ok();
    
    // Verify score was updated
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.get_user_score(managed_address!(&setup.user_address)), 750u64);
        })
        .assert_ok();
    
    // Test that non-oracle cannot update score
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 800u64);
        })
        .assert_error(4, "Only oracle can update scores");
    
    // Test score outside valid range
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 1200u64);
        })
        .assert_error(4, "Score out of valid range");
}

#[test]
fn test_get_user_score() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Test default score for user without explicit score
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.get_user_score(managed_address!(&setup.user_address)), 0u64);
        })
        .assert_ok();
    
    // Set a score and verify
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 500u64);
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.get_user_score(managed_address!(&setup.user_address)), 500u64);
        })
        .assert_ok();
}

#[test]
fn test_is_eligible_for_loan() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Set user score
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 600u64);
        })
        .assert_ok();
    
    // Test eligibility - should be eligible
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert!(sc.is_eligible_for_loan(managed_address!(&setup.user_address), 500u64));
        })
        .assert_ok();
    
    // Test eligibility - should not be eligible
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert!(!sc.is_eligible_for_loan(managed_address!(&setup.user_address), 700u64));
        })
        .assert_ok();
}

#[test]
fn test_calculate_max_loan_amount() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Set user score to 500 (half of max 1000)
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 500u64);
        })
        .assert_ok();
    
    // Calculate max loan amount with base amount 1000
    // Expected: 1000 * (500/1000) * 2 = 1000
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let max_loan = sc.calculate_max_loan_amount(
                managed_address!(&setup.user_address),
                managed_biguint!(1000u64),
            );
            assert_eq!(max_loan, managed_biguint!(1000u64));
        })
        .assert_ok();
    
    // Set user score to maximum (1000)
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 1000u64);
        })
        .assert_ok();
    
    // Calculate max loan amount with base amount 1000
    // Expected: 1000 * (1000/1000) * 2 = 2000
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let max_loan = sc.calculate_max_loan_amount(
                managed_address!(&setup.user_address),
                managed_biguint!(1000u64),
            );
            assert_eq!(max_loan, managed_biguint!(2000u64));
        })
        .assert_ok();
}