use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use reputation_score::*;

const WASM_PATH: &str = "output/reputation-score.wasm";

struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> reputation_score::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub oracle_address: Address,
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
    
    // Deploy contract
    let contract_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        builder,
        WASM_PATH,
    );
    
    // Initialize contract with min_score=0, max_score=1000
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
        owner_address,
        oracle_address,
        contract_wrapper,
    }
}

// Função para gerar um endereço aleatório
fn generate_random_address(rng: &mut StdRng) -> Address {
    let mut address_bytes = [0u8; 32];
    rng.fill(&mut address_bytes);
    Address::from_slice(&address_bytes)
}

#[test]
fn test_update_score_fuzzy() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Use uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Gerar múltiplos usuários e pontuações aleatórias
    for _ in 0..50 {
        let random_user = generate_random_address(&mut rng);
        let score = rng.gen_range(0..1500); // Inclui pontuações fora do intervalo válido
        
        // O oráculo tenta atualizar a pontuação
        let result = setup.blockchain_wrapper
            .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.update_score(managed_address!(&random_user), score);
            });
        
        // Verificar se o resultado corresponde à expectativa baseada no intervalo
        if score <= 1000 {
            result.assert_ok();
            
            // Verificar se a pontuação foi realmente atualizada
            setup.blockchain_wrapper
                .execute_query(&setup.contract_wrapper, |sc| {
                    assert_eq!(sc.get_user_score(managed_address!(&random_user)), score);
                })
                .assert_ok();
        } else {
            result.assert_error(4, "Score out of valid range");
        }
    }
}

#[test]
fn test_calculate_max_loan_amount_fuzzy() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Use uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Gerar múltiplos usuários, pontuações e valores base aleatórios
    for _ in 0..50 {
        let random_user = generate_random_address(&mut rng);
        let score = rng.gen_range(0..1001); // Pontuações dentro do intervalo válido
        let base_amount = rng.gen_range(100..10001); // Valores base variados
        
        // Definir pontuação do usuário
        setup.blockchain_wrapper
            .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.update_score(managed_address!(&random_user), score);
            })
            .assert_ok();
        
        // Calcular valor máximo do empréstimo
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let max_loan = sc.calculate_max_loan_amount(
                    managed_address!(&random_user),
                    managed_biguint!(base_amount),
                );
                
                // Fórmula esperada: base_amount * (score / 1000) * 2
                let expected = base_amount as u128 * score as u128 * 2 / 1000;
                assert_eq!(max_loan, managed_biguint!(expected));
            })
            .assert_ok();
    }
}

// Teste de valores extremos para verificar comportamento em condições limites
#[test]
fn test_edge_cases() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Caso 1: Pontuação zero
    let user1 = Address::from_slice(&[1u8; 32]);
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&user1), 0u64);
        })
        .assert_ok();
    
    // Caso 2: Pontuação máxima
    let user2 = Address::from_slice(&[2u8; 32]);
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&user2), 1000u64);
        })
        .assert_ok();
    
    // Verificar cálculo de empréstimo com valor base zero
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan_zero = sc.calculate_max_loan_amount(
                managed_address!(&user2),
                managed_biguint!(0u64),
            );
            assert_eq!(loan_zero, managed_biguint!(0u64));
        })
        .assert_ok();
    
    // Verificar cálculo de empréstimo com valor base muito grande
    let large_base = u64::MAX;
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let max_loan = sc.calculate_max_loan_amount(
                managed_address!(&user2),
                managed_biguint!(large_base),
            );
            
            // User2 tem pontuação máxima, então o resultado esperado é base * 2
            let expected = BigUint::from(large_base) * BigUint::from(2u64);
            assert_eq!(max_loan, managed_biguint!(expected));
        })
        .assert_ok();
}