use multiversx_sc::types::Address;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint, testing_framework::{BlockchainStateWrapper, ContractObjWrapper}, DebugApi
};

use reputation_score::*;

const WASM_PATH: &str = "output/reputation-score.wasm";

struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> reputation_score::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub oracle_address: Address,
    pub attacker_address: Address,
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
    let attacker_address = blockchain_wrapper.create_user_account(&rust_zero);
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
        attacker_address,
        user_address,
        contract_wrapper,
    }
}

#[test]
fn test_access_control_owner_functions() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Attacker tenta definir um novo endereço de oráculo
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_oracle_address(managed_address!(&setup.attacker_address));
        })
        .assert_error(4, "Endpoint can only be called by owner");
    
    // Verificar que o oráculo não foi alterado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.oracle_address().get(), managed_address!(&setup.oracle_address));
        })
        .assert_ok();
}

#[test]
fn test_access_control_oracle_functions() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Attacker tenta atualizar a pontuação como se fosse o oráculo
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 900u64);
        })
        .assert_error(4, "Only oracle can update scores");
    
    // Verificar que a pontuação não foi alterada (deve ser o valor padrão)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.get_user_score(managed_address!(&setup.user_address)), 0u64);
        })
        .assert_ok();
}

#[test]
fn test_oracle_not_configured() {
    // Criar nova configuração sem definir o oráculo
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let user_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    // Deploy contract
    let contract_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        reputation_score::contract_obj,
        WASM_PATH,
    );
    
    // Initialize contract
    blockchain_wrapper
        .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
            sc.init(0u64, 1000u64);
        })
        .assert_ok();
    
    // Tentar atualizar a pontuação sem configurar o oráculo primeiro
    // Este teste é para verificar se o contrato verifica corretamente que o oráculo está configurado
    blockchain_wrapper
        .execute_tx(&owner_address, &contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&user_address), 500u64);
        })
        .assert_error(4, "Oracle not configured");
}

#[test]
fn test_arithmetic_operations() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Configurar um caso com pontuação máxima
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.user_address), 1000u64);
        })
        .assert_ok();
    
    // Testar com um valor base extremamente grande para verificar se há problemas de overflow
    let very_large_value = u64::MAX;
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // O cálculo é: base_amount * user_score * 2 / max_score
            // Com user_score = 1000 e max_score = 1000, isso simplifica para base_amount * 2
            // Verificar se isso causa overflow
            let max_loan = sc.calculate_max_loan_amount(
                managed_address!(&setup.user_address),
                managed_biguint!(very_large_value),
            );
            
            // O resultado esperado é base * 2, verificar se o cálculo está correto
            assert_eq!(max_loan, managed_biguint!(very_large_value) * managed_biguint!(2u64));
        })
        .assert_ok();
}