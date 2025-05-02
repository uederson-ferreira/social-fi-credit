use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use reputation_score::*;

const WASM_PATH: &str = "output/reputation-score.wasm";

// Estrutura para configuração dos testes
struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> reputation_score::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub oracle_address: Address,
    pub users: Vec<Address>,
    pub contract_wrapper: ContractObjWrapper<reputation_score::ContractObj<DebugApi>, ContractObjBuilder>,
}

// Configurar o contrato e os usuários para o cenário
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
    
    // Criar vários usuários com diferentes pontuações para o cenário
    let mut users = Vec::new();
    for _ in 0..5 {
        users.push(blockchain_wrapper.create_user_account(&rust_zero));
    }
    
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
        owner_address,
        oracle_address,
        users,
        contract_wrapper,
    }
}

// Cenário: Avaliação de elegibilidade para empréstimo para diferentes usuários
#[test]
fn test_loan_eligibility_scenario() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Definir diferentes pontuações para cada usuário
    let scores = [200u64, 400u64, 600u64, 800u64, 1000u64];
    
    // Atribuir pontuações aos usuários
    for (i, user) in setup.users.iter().enumerate() {
        setup.blockchain_wrapper
            .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.update_score(managed_address!(user), scores[i]);
            })
            .assert_ok();
    }
    
    // Cenário 1: Verificar elegibilidade para empréstimo com requisito de 500 pontos
    let required_score = 500u64;
    for (i, user) in setup.users.iter().enumerate() {
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let is_eligible = sc.is_eligible_for_loan(managed_address!(user), required_score);
                
                // Usuários com pontuação >= 500 devem ser elegíveis
                let expected_eligibility = scores[i] >= required_score;
                assert_eq!(is_eligible, expected_eligibility);
            })
            .assert_ok();
    }
    
    // Cenário 2: Calcular valor máximo de empréstimo para cada usuário
    let base_amount = managed_biguint!(10_000u64);
    for (i, user) in setup.users.iter().enumerate() {
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let max_loan = sc.calculate_max_loan_amount(managed_address!(user), base_amount.clone());
                
                // Valor esperado: base_amount * (score / 1000) * 2
                let expected = base_amount.clone() * scores[i] * 2u64 / 1000u64;
                assert_eq!(max_loan, expected);
            })
            .assert_ok();
    }
    
    // Cenário 3: Atualizar as pontuações e verificar mudanças na elegibilidade
    // Reduzir pontuações de todos os usuários em 100 pontos
    for (i, user) in setup.users.iter().enumerate() {
        let new_score = if scores[i] >= 100 { scores[i] - 100 } else { 0 };
        
        setup.blockchain_wrapper
            .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.update_score(managed_address!(user), new_score);
            })
            .assert_ok();
        
        // Verificar elegibilidade novamente
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let is_eligible = sc.is_eligible_for_loan(managed_address!(user), required_score);
                let expected_eligibility = new_score >= required_score;
                
                assert_eq!(is_eligible, expected_eligibility);
                
                // Verificar também se a pontuação foi atualizada corretamente
                assert_eq!(sc.get_user_score(managed_address!(user)), new_score);
            })
            .assert_ok();
    }
    
    // Cenário 4: Testar o impacto no valor máximo de empréstimo após mudança de pontuação
    for (i, user) in setup.users.iter().enumerate() {
        let current_score = if scores[i] >= 100 { scores[i] - 100 } else { 0 };
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let max_loan = sc.calculate_max_loan_amount(managed_address!(user), base_amount.clone());
                
                // Novo valor esperado com pontuação reduzida
                let expected = base_amount.clone() * current_score * 2u64 / 1000u64;
                assert_eq!(max_loan, expected);
            })
            .assert_ok();
    }
}

// Cenário: Simulação de evolução de pontuação ao longo do tempo
#[test]
fn test_score_evolution_scenario() {
    let mut setup = setup_contract(reputation_score::contract_obj);
    
    // Usar apenas o primeiro usuário para este cenário
    let user = &setup.users[0];
    
    // Definir pontuação inicial
    let initial_score = 300u64;
    setup.blockchain_wrapper
        .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(user), initial_score);
        })
        .assert_ok();
    
    // Verificar elegibilidade inicial para diferentes níveis de empréstimo
    let loan_thresholds = [200u64, 400u64, 600u64, 800u64];
    
    // Verificar elegibilidade inicial
    for threshold in loan_thresholds.iter() {
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let is_eligible = sc.is_eligible_for_loan(managed_address!(user), *threshold);
                let expected = initial_score >= *threshold;
                assert_eq!(is_eligible, expected);
            })
            .assert_ok();
    }
    
    // Simular melhoria gradual de pontuação ao longo do tempo
    let score_improvements = [50u64, 100u64, 150u64, 200u64];
    let mut current_score = initial_score;
    
    for improvement in score_improvements.iter() {
        // Aumentar a pontuação
        current_score += improvement;
        if current_score > 1000 {
            current_score = 1000; // Não ultrapassar o máximo
        }
        
        // Atualizar pontuação
        setup.blockchain_wrapper
            .execute_tx(&setup.oracle_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.update_score(managed_address!(user), current_score);
            })
            .assert_ok();
        
        // Verificar elegibilidade para cada nível após a melhoria
        for threshold in loan_thresholds.iter() {
            setup.blockchain_wrapper
                .execute_query(&setup.contract_wrapper, |sc| {
                    let is_eligible = sc.is_eligible_for_loan(managed_address!(user), *threshold);
                    let expected = current_score >= *threshold;
                    assert_eq!(is_eligible, expected);
                })
                .assert_ok();
        }
        
        // Calcular valor máximo de empréstimo após cada melhoria
        let base_amount = managed_biguint!(10_000u64);
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let max_loan = sc.calculate_max_loan_amount(managed_address!(user), base_amount.clone());
                let expected = base_amount.clone() * current_score * 2u64 / 1000u64;
                assert_eq!(max_loan, expected);
            })
            .assert_ok();
    }
}