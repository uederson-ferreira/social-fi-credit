// ==========================================================================
// ARQUIVO: liquidity_pool_fuzzy_test.rs
// Descrição: Testes fuzzy com entradas aleatórias para o contrato LiquidityPool
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use liquidity_pool::*;

const WASM_PATH: &str = "output/liquidity-pool.wasm";

// Estrutura para configuração dos testes
struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> liquidity_pool::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub loan_controller_address: Address,
    pub debt_token_address: Address,
    pub lp_token_address: Address,
    pub providers: Vec<Address>,
    pub borrowers: Vec<Address>,
    pub contract_wrapper: ContractObjWrapper<liquidity_pool::ContractObj<DebugApi>, ContractObjBuilder>,
}

// Função de configuração para os testes fuzzy
fn setup_fuzzy_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
    num_providers: usize,
    num_borrowers: usize,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> liquidity_pool::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let loan_controller_address = blockchain_wrapper.create_user_account(&rust_zero);
    let debt_token_address = blockchain_wrapper.create_user_account(&rust_zero);
    let lp_token_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    // Criar provedores de liquidez
    let mut providers = Vec::with_capacity(num_providers);
    for _ in 0..num_providers {
        let provider_address = blockchain_wrapper.create_user_account(&rust_biguint!(100000));
        providers.push(provider_address);
    }
    
    // Criar tomadores
    let mut borrowers = Vec::with_capacity(num_borrowers);
    for _ in 0..num_borrowers {
        let borrower_address = blockchain_wrapper.create_user_account(&rust_biguint!(10000));
        borrowers.push(borrower_address);
    }
    
    // Deploy do contrato
    let contract_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        builder,
        WASM_PATH,
    );
    
    // Inicialização do contrato
    blockchain_wrapper
        .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&loan_controller_address),
                managed_biguint!(1_000),
                10u64,
            );
        })
        .assert_ok();
    
    // Definir endereços dos contratos relacionados
    blockchain_wrapper
        .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
            sc.set_debt_token_address(managed_address!(&debt_token_address));
            sc.set_lp_token_address(managed_address!(&lp_token_address));
        })
        .assert_ok();
    
    ContractSetup {
        blockchain_wrapper,
        owner_address,
        loan_controller_address,
        debt_token_address,
        lp_token_address,
        providers,
        borrowers,
        contract_wrapper,
    }
}

// Função de configuração simples para testes individuais
fn setup_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> liquidity_pool::ContractObj<DebugApi>,
{
    // Reutilizamos a função de configuração fuzzy com um número mínimo de provedores e tomadores
    setup_fuzzy_contract(builder, 1, 1)
}

// Função para gerar um endereço aleatório
fn generate_random_address(rng: &mut StdRng) -> Address {
    let mut address_bytes = [0u8; 32];
    rng.fill(&mut address_bytes);
    Address::from_slice(&address_bytes)
}

// Teste fuzzy para múltiplos depósitos e retiradas
#[test]
fn test_deposit_withdraw_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(liquidity_pool::contract_obj, 10, 5);
    
    // Realizar depósitos aleatórios
    let mut total_deposits = BigUint::zero();
    let mut provider_deposits = vec![BigUint::zero(); setup.providers.len()];
    
    for _ in 0..50 {
        let provider_idx = rng.gen_range(0..setup.providers.len());
        let provider = &setup.providers[provider_idx];
        
        let amount = rng.gen_range(1000..10000u64);
        
        setup.blockchain_wrapper
            .execute_tx(provider, &setup.contract_wrapper, &rust_biguint!(amount), |sc| {
                sc.deposit_funds();
                
                // Simular emissão de tokens LP
                sc.lp_tokens_minted_endpoint(managed_address!(provider), managed_biguint!(amount));
            })
            .assert_ok();
        
        total_deposits += BigUint::from(amount);
        provider_deposits[provider_idx] += BigUint::from(amount);
    }
    
    // Verificar total de depósitos
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Converter o valor retornado pelo SC para BigUint normal
            let sc_liquidity = sc.total_liquidity().get().to_bigint().unwrap();
            assert_eq!(sc_liquidity, total_deposits);
        })
        .assert_ok();
    
    // Realizar retiradas aleatórias
    for _ in 0..30 {
        let provider_idx = rng.gen_range(0..setup.providers.len());
        let provider = &setup.providers[provider_idx];
        
        if provider_deposits[provider_idx] > BigUint::zero() {
            // Retirar até 90% do depósito deste provedor
            let max_withdraw = provider_deposits[provider_idx].clone() * BigUint::from(90u64) / BigUint::from(100u64);
            let withdraw_amount_u64 = max_withdraw.to_u64().unwrap_or(1);
            
            if withdraw_amount_u64 > 0 {
                let amount = rng.gen_range(1..=withdraw_amount_u64);
                
                setup.blockchain_wrapper
                    .execute_tx(provider, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        // Simular queima de tokens LP
                        sc.lp_tokens_burned_endpoint(managed_address!(provider), managed_biguint!(amount));
                        
                        // Retirar
                        sc.withdraw(managed_biguint!(amount));
                    })
                    .assert_ok();
                
                total_deposits -= BigUint::from(amount);
                provider_deposits[provider_idx] -= BigUint::from(amount);
            }
        }
    }
    
    // Verificar total final
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Converter o valor retornado pelo SC para BigUint normal
            let sc_liquidity = sc.total_liquidity().get().to_bigint().unwrap();
            assert_eq!(sc_liquidity, total_deposits);
        })
        .assert_ok();
}

// Teste fuzzy para operações de empréstimo e pagamento
#[test]
fn test_borrow_repay_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(liquidity_pool::contract_obj, 5, 10);
    
    // Adicionar liquidez total
    let total_liquidity = 500000u64;
    
    setup.blockchain_wrapper
        .execute_tx(&setup.providers[0], &setup.contract_wrapper, &rust_biguint!(total_liquidity), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Realizar empréstimos aleatórios
    let mut total_borrows = BigUint::zero();
    let mut borrower_debts = vec![BigUint::zero(); setup.borrowers.len()];
    
    for _ in 0..30 {
        let borrower_idx = rng.gen_range(0..setup.borrowers.len());
        let borrower = &setup.borrowers[borrower_idx];
        
        // Limitar empréstimos a 80% da liquidez total
        let max_borrow = BigUint::from(total_liquidity) * BigUint::from(80u64) / BigUint::from(100u64);
        let available_borrow = if max_borrow > total_borrows.clone() {
            max_borrow - total_borrows.clone()
        } else {
            BigUint::zero()
        };
        
        if available_borrow > BigUint::zero() {
            let max_amount_u64 = available_borrow.to_u64().unwrap_or(1);
            let amount = rng.gen_range(1000..=max_amount_u64.min(50000));
            
            setup.blockchain_wrapper
                .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    // Calcular taxa de juros baseada na utilização atual
                    let current_utilization = sc.utilization_rate().get();
                    let interest_rate = sc.calculate_current_interest_rate();
                    
                    // Fazer empréstimo
                    sc.borrow(
                        managed_address!(borrower),
                        managed_biguint!(amount),
                        interest_rate
                    );
                    
                    // Simular emissão de tokens de dívida
                    sc.debt_tokens_minted_endpoint(managed_address!(borrower), managed_biguint!(amount));
                })
                .assert_ok();
            
            total_borrows += BigUint::from(amount);
            borrower_debts[borrower_idx] += BigUint::from(amount);
        }
    }
    
    // Verificar total de empréstimos
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Converter o valor retornado pelo SC para BigUint normal
            let sc_borrows = sc.total_borrows().get().to_bigint().unwrap();
            assert_eq!(sc_borrows, total_borrows);
        })
        .assert_ok();
    
    // Realizar pagamentos aleatórios
    for _ in 0..20 {
        let borrower_idx = rng.gen_range(0..setup.borrowers.len());
        let borrower = &setup.borrowers[borrower_idx];
        
        if borrower_debts[borrower_idx] > BigUint::zero() {
            // Pagar até 100% da dívida
            let max_repay = borrower_debts[borrower_idx].clone();
            let repay_amount_u64 = max_repay.to_u64().unwrap_or(1);
            
            if repay_amount_u64 > 0 {
                let amount = rng.gen_range(1..=repay_amount_u64);
                
                setup.blockchain_wrapper
                    .execute_tx(borrower, &setup.contract_wrapper, &rust_biguint!(amount), |sc| {
                        // Simular queima de tokens de dívida
                        sc.debt_tokens_burned_endpoint(managed_address!(borrower), managed_biguint!(amount));
                        
                        // Pagar
                        sc.repay();
                    })
                    .assert_ok();
                
                total_borrows -= BigUint::from(amount);
                borrower_debts[borrower_idx] -= BigUint::from(amount);
            }
        }
    }
    
    // Verificar total de empréstimos final
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.total_borrows().get(), managed_biguint!(total_borrows));
        })
        .assert_ok();
}

// Teste fuzzy para cálculo de taxas de juros com diferentes níveis de utilização
#[test]
fn test_interest_rate_calculation_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Testar cálculo de juros com diferentes taxas de utilização
    for _ in 0..100 {
        let utilization = rng.gen_range(0..10000u64);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                // Definir taxa de utilização
                sc.utilization_rate().set(utilization);
                
                // Calcular taxa de juros
                let interest_rate = sc.calculate_current_interest_rate();
                
                // Verificar se está dentro dos limites esperados
                if utilization <= sc.target_utilization_rate().get() {
                    // Abaixo ou igual à meta: taxa deve ser proporcional e <= base_rate
                    assert!(interest_rate <= sc.interest_rate_base().get());
                } else {
                    // Acima da meta: taxa deve crescer proporcionalmente
                    assert!(interest_rate >= sc.interest_rate_base().get());
                    
                    // Calcular taxa máxima (quando utilização = max_utilization_rate)
                    let max_rate = sc.interest_rate_base().get() * 2; // Exemplo: dobra quando chega ao máximo
                    
                    // Não deve exceder a taxa máxima
                    assert!(interest_rate <= max_rate);
                }
            })
            .assert_ok();
    }
}

// Teste fuzzy para distribuição de juros entre vários provedores
#[test]
fn test_interest_distribution_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(liquidity_pool::contract_obj, 10, 1);
    
    // Adicionar liquidez com valores aleatórios para cada provedor
    let mut total_liquidity = BigUint::zero();
    let mut provider_liquidity = vec![BigUint::zero(); setup.providers.len()];
    
    for (i, provider) in setup.providers.iter().enumerate() {
        let amount = rng.gen_range(5000..50000u64);
        
        setup.blockchain_wrapper
            .execute_tx(provider, &setup.contract_wrapper, &rust_biguint!(amount), |sc| {
                sc.deposit_funds();
                
                // Simular emissão de tokens LP
                sc.lp_tokens_minted_endpoint(managed_address!(provider), managed_biguint!(amount));
            })
            .assert_ok();
        
        total_liquidity += BigUint::from(amount);
        provider_liquidity[i] = BigUint::from(amount);
    }
    
    // Fazer um empréstimo grande usando 90% do pool
    let borrow_amount = total_liquidity.clone() * BigUint::from(90u64) / BigUint::from(100u64);
    let borrow_amount_u64 = borrow_amount.to_u64().unwrap_or(0);
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow(
                managed_address!(&setup.borrowers[0]),
                managed_biguint!(borrow_amount_u64),
                1200u64 // 12% de juros
            );
        })
        .assert_ok();
    
    // Simular acúmulo de juros (12% do valor emprestado)
    let interest_amount = borrow_amount.clone() * BigUint::from(1200u64) / BigUint::from(10000u64);
    let interest_amount_u64 = interest_amount.to_u64().unwrap_or(0);
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(interest_amount_u64), |sc| {
            sc.add_accumulated_interest_endpoint(managed_biguint!(interest_amount_u64));
        })
        .assert_ok();
    
    // Distribuir juros
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.distribute_interest_endpoint();
        })
        .assert_ok();
    
    // Verificar que cada provedor recebeu a proporção correta de juros
    let reserve_percent = 2000u64; // 20%
    let total_interest_for_providers = BigUint::from(interest_amount_u64) * BigUint::from(10000u64 - reserve_percent) / BigUint::from(10000u64);
    
    for (i, provider) in setup.providers.iter().enumerate() {
        // Calcular juros esperados para este provedor
        let provider_share = provider_liquidity[i].clone() * BigUint::from(10000u64) / total_liquidity.clone();
        let expected_interest = total_interest_for_providers.clone() * provider_share / BigUint::from(10000u64);
        let expected_interest_u64 = expected_interest.to_u64().unwrap_or(0);
        
        // Verificar juros recebidos
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let provider_interest = sc.provider_interest(&managed_address!(provider)).get();
                
                // Permitir uma pequena margem de erro devido a arredondamentos
                let diff = if provider_interest > managed_biguint!(expected_interest_u64) {
                    provider_interest.clone() - managed_biguint!(expected_interest_u64)
                } else {
                    managed_biguint!(expected_interest_u64) - provider_interest.clone()
                };
                
                // Erro não deve exceder 1 unidade por provedor
                assert!(diff <= managed_biguint!(1));
            })
            .assert_ok();
    }
}

// Teste para funções administrativas de pause/unpause
#[test]
fn test_pause_unpause() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Verificar estado inicial (não pausado)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.is_paused(), false);
        })
        .assert_ok();
    
    // Pausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    // Verificar que está pausado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.is_paused(), true);
        })
        .assert_ok();
    
    // Despausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause();
        })
        .assert_ok();
    
    // Verificar que está despausado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.is_paused(), false);
        })
        .assert_ok();
}

// Teste para funções de atualização de parâmetros
#[test]
fn test_parameter_updates() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Testar atualização da taxa de juros base
    let new_base_rate = 1200u64; // 12%
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_interest_rate_base(new_base_rate);
        })
        .assert_ok();
    
    // Verificar valor atualizado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.interest_rate_base().get(), new_base_rate);
        })
        .assert_ok();
    
    // Testar atualização da taxa de utilização alvo
    let new_target_rate = 7500u64; // 75%
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_target_utilization_rate(new_target_rate);
        })
        .assert_ok();
    
    // Verificar valor atualizado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.target_utilization_rate().get(), new_target_rate);
        })
        .assert_ok();
    
    // Testar atualização do percentual de reserva
    let new_reserve_percent = 1500u64; // 15%
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_reserve_percent(new_reserve_percent);
        })
        .assert_ok();
    
    // Verificar valor atualizado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.reserve_percent().get(), new_reserve_percent);
        })
        .assert_ok();
}

// Teste para a função use_reserves_endpoint
#[test]
fn test_use_reserves() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial para o primeiro provedor
    setup.blockchain_wrapper
        .execute_tx(&setup.providers[0], &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Simular juros acumulados
    let interest_amount = 2000u64;
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(interest_amount), |sc| {
            sc.add_accumulated_interest_endpoint(managed_biguint!(interest_amount));
        })
        .assert_ok();
    
    // Distribuir juros, parte vai para as reservas
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.distribute_interest_endpoint();
        })
        .assert_ok();
    
    // Verificar valor das reservas
    let expected_reserves = BigUint::from(interest_amount) * BigUint::from(2000u64) / BigUint::from(10000u64); // 20% dos juros
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.total_reserves().get(), managed_biguint!(expected_reserves));
        })
        .assert_ok();
    
    // Destino para uso das reservas
    let reserves_target = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Usar metade das reservas
    let use_amount = expected_reserves.clone() / BigUint::from(2u64);
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.use_reserves_endpoint(
                managed_address!(&reserves_target),
                managed_biguint!(use_amount.clone())
            );
        })
        .assert_ok();
    
    // Verificar que as reservas foram reduzidas corretamente
    let expected_remaining = expected_reserves - use_amount;
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.total_reserves().get(), managed_biguint!(expected_remaining));
        })
        .assert_ok();
}

// Mock do contrato ReputationScore para testes
struct ReputationScoreMock {}

impl ReputationScoreMock {
    fn new() -> Self {
        ReputationScoreMock {}
    }
    
    fn is_eligible_for_loan(&self, _user: &Address, _min_score: u64) -> bool {
        true // Mock sempre retorna true para testes
    }
    
    fn calculate_max_loan_amount(&self, _user: &Address, base_amount: &BigUint) -> BigUint {
        base_amount.clone() * BigUint::from(2u64) // Mock dobra o valor base
    }
    
    fn get_user_score(&self, _user: &Address) -> u64 {
        85u64 // Mock retorna uma pontuação fixa
    }
}

// Teste para integração com ReputationScore
#[test]
fn test_reputation_score_integration() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Criar um usuário para o teste
    let test_user = setup.blockchain_wrapper.create_user_account(&rust_biguint!(1000));
    
    // Inicializar mock do ReputationScore
    let reputation_mock = ReputationScoreMock::new();
    
    // Simular uma tentativa de empréstimo com verificação de reputação
    let base_amount = BigUint::from(1000u64);
    let is_eligible = reputation_mock.is_eligible_for_loan(&test_user, 70u64);
    let max_loan = if is_eligible {
        reputation_mock.calculate_max_loan_amount(&test_user, &base_amount)
    } else {
        BigUint::zero()
    };
    
    // Verificar valores do mock
    assert_eq!(is_eligible, true);
    assert_eq!(max_loan, BigUint::from(2000u64));
    assert_eq!(reputation_mock.get_user_score(&test_user), 85u64);
    
    // Se elegível e com fundos suficientes no pool, executar o empréstimo
    // Primeiro adiciona liquidez ao pool
    setup.blockchain_wrapper
        .execute_tx(&setup.providers[0], &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Em seguida, faz o empréstimo através do controlador de empréstimos
    if is_eligible && max_loan > BigUint::zero() {
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                // Usamos o valor máximo calculado pelo sistema de reputação
                sc.borrow(
                    managed_address!(&test_user),
                    managed_biguint!(max_loan),
                    1000u64 // 10% de juros
                );
            })
            .assert_ok();
        
        // Verificar dívida do usuário
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                assert_eq!(sc.get_borrower_debt(managed_address!(&test_user)), managed_biguint!(max_loan));
            })
            .assert_ok();
    }
}

// Teste para verificar o processo de rendimento pendente
#[test]
fn test_process_pending_yield() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    let deposit_amount = 10000u64;
    setup.blockchain_wrapper
        .execute_tx(&setup.providers[0], &setup.contract_wrapper, &rust_biguint!(deposit_amount), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Avançar o tempo em 1 ano para acumular rendimento
    let seconds_in_year = 31_536_000u64;
    setup.blockchain_wrapper.add_block_time_cache_advance(seconds_in_year);
    
    // Fazer um depósito adicional para acionar o processamento de rendimento pendente
    setup.blockchain_wrapper
        .execute_tx(&setup.providers[0], &setup.contract_wrapper, &rust_biguint!(1000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Verificar se o rendimento foi adicionado corretamente
    // Com yield_percentage de 10 (0.1%), o rendimento esperado após 1 ano é de 0.1% do depósito
    let expected_yield = BigUint::from(deposit_amount) * BigUint::from(10u64) / BigUint::from(10000u64);
    
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let provider_funds = sc.get_provider_funds(managed_address!(&setup.providers[0]));
            
            // O valor total deve ser o depósito inicial + depósito adicional + rendimento
            let expected_total = BigUint::from(deposit_amount) + BigUint::from(1000u64) + expected_yield;
            assert_eq!(provider_funds.amount, expected_total);
        })
        .assert_ok();
}

// Teste para verificar a atualização da taxa de utilização
#[test]
fn test_utilization_rate_update() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    let liquidity_amount = 100000u64;
    setup.blockchain_wrapper
        .execute_tx(&setup.providers[0], &setup.contract_wrapper, &rust_biguint!(liquidity_amount), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Verificar taxa de utilização inicial (deve ser 0)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.utilization_rate().get(), 0u64);
        })
        .assert_ok();
    
    // Fazer um empréstimo de 50% da liquidez
    let borrow_amount = liquidity_amount / 2;
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow(
                managed_address!(&setup.borrowers[0]),
                managed_biguint!(borrow_amount),
                1000u64 // 10% de juros
            );
        })
        .assert_ok();
    
    // Verificar taxa de utilização após empréstimo (deve ser 5000 = 50%)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.utilization_rate().get(), 5000u64);
        })
        .assert_ok();
    
    // Fazer um pagamento de metade do empréstimo
    let repay_amount = borrow_amount / 2;
    setup.blockchain_wrapper
        .execute_tx(&setup.borrowers[0], &setup.contract_wrapper, &rust_biguint!(repay_amount), |sc| {
            sc.repay();
        })
        .assert_ok();
    
    // Verificar taxa de utilização após pagamento (deve ser 2500 = 25%)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.utilization_rate().get(), 2500u64);
        })
        .assert_ok();
}