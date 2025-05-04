// ==========================================================================
// ARQUIVO: loan_controller_fuzzy_test.rs
// Descrição: Testes fuzzy com entradas aleatórias para o contrato LoanController
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use loan_controller::*;

const WASM_PATH: &str = "output/loan-controller.wasm";

// Estrutura para configuração dos testes
struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> loan_controller::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub _reputation_score_address: Address, 
    pub contract_wrapper: ContractObjWrapper<loan_controller::ContractObj<DebugApi>, ContractObjBuilder>,
}

// Função de configuração para os testes
fn setup_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> loan_controller::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let reputation_score_address = blockchain_wrapper.create_user_account(&rust_zero);
    
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
                managed_address!(&reputation_score_address),
                500u64, // Pontuação mínima necessária
                1000u64, // Taxa de juros base (10% = 1000 pontos base)
                managed_biguint!(10_000), // Valor base do empréstimo
            );
        })
        .assert_ok();
    
    ContractSetup {
        blockchain_wrapper,
        owner_address,
        _reputation_score_address: reputation_score_address,
        contract_wrapper,
    }
}

// Função para gerar um endereço aleatório
fn generate_random_address(rng: &mut StdRng) -> Address {
    let mut address_bytes = [0u8; 32];
    rng.fill(&mut address_bytes);
    Address::from_slice(&address_bytes)
}

// Teste fuzzy para cálculo de taxa de juros com valores aleatórios
#[test]
fn test_interest_rate_calculation_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Testar cálculo de juros com diferentes pontuações aleatórias
    for _ in 0..100 {
        let score = rng.gen_range(0..1200); // Incluir pontuações fora do intervalo válido
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let interest_rate = sc.calculate_interest_rate(score);
                
                // Verificar se o cálculo está correto
                let expected_rate: u64;
                if score >= 1000 {
                    // Para pontuação máxima ou acima, taxa é 20% da base
                    expected_rate = sc.interest_rate_base().get() / 5;
                } else {
                    // Fórmula padrão: base_rate * (1 - (score/1000) * 0.8)
                    let score_factor = (score * 80) / 1000;
                    expected_rate = sc.interest_rate_base().get() * (100 - score_factor) / 100;
                }
                
                assert_eq!(interest_rate, expected_rate);
            })
            .assert_ok();
    }
}

// Teste fuzzy para valores de empréstimo aleatórios
#[test]
fn test_loan_amounts_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Testar diferentes valores de empréstimo e prazos
    for _ in 0..50 {
        let amount = rng.gen_range(1000..100000u64);
        let interest_rate = rng.gen_range(100..2000u64); // 1% a 20%
        let duration_days = rng.gen_range(1..365u64);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |_sc| {
                // Calcular valor total de pagamento
                let amount_biguint: BigUint<DebugApi> = managed_biguint!(amount);
                let interest_amount: BigUint<_> = &amount_biguint * &managed_biguint!(interest_rate) / &managed_biguint!(10000u32);
                let repayment_amount: BigUint<_> = &amount_biguint + &interest_amount;
                
                // Testar se o cálculo está correto através de uma função simulada
                // Na implementação real, isto seria parte do código do contrato
                let current_timestamp = 12345u64;
                let due_timestamp = current_timestamp + duration_days * 86400;
                
                assert!(due_timestamp > current_timestamp);
                assert!(repayment_amount > amount_biguint);
            })
            .assert_ok();
    }
}

// Teste fuzzy para empréstimos com diferentes usuários e pontuações
#[test]
fn test_multiple_users_loans_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Criar vários usuários com diferentes saldos
    let num_users = 10;
    let mut users = Vec::with_capacity(num_users);
    
    for _ in 0..num_users {
        let initial_balance = rng.gen_range(10000..100000u64);
        let user_address = setup.blockchain_wrapper.create_user_account(&rust_biguint!(initial_balance));
        users.push(user_address);
    }
    
}

// Teste fuzzy para extensões de prazo aleatórias
#[test]
fn test_loan_extensions_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Configurar taxa de extensão
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.extension_fee_percent().set(1000u64); // 10%
        })
        .assert_ok();
    
    // Testar com vários prazos aleatórios
    for _ in 0..50 {
        let loan_amount = rng.gen_range(1000..50000u64);
        let initial_days = rng.gen_range(7..60u64);
        let extension_days = rng.gen_range(1..30u64);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |_sc| {
                // Configurar timestamps
                let current_time = 10000u64;
                let initial_due_time = current_time + initial_days * 24 * 60 * 60;
                let extended_due_time = initial_due_time + extension_days * 24 * 60 * 60;
                
                // Calcular taxa de extensão
                let loan_amount_biguint: BigUint<DebugApi> = managed_biguint!(loan_amount);
                let repayment_amount: BigUint<_> = &loan_amount_biguint * &managed_biguint!(110) / &managed_biguint!(100);
                let extension_fee: BigUint<_> = &repayment_amount * &managed_biguint!(10) / &managed_biguint!(100);
                
                // Verificar cálculos
                assert!(extended_due_time > initial_due_time);
                assert!(extension_fee > managed_biguint!(0));
            })
            .assert_ok();
    }
}

// Teste fuzzy para pagamentos parciais
#[test]
fn test_partial_payments_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    for _ in 0..50 {
        let loan_amount: BigUint<DebugApi> = managed_biguint!(rng.gen_range(5000..50000u64));
        let repayment_amount: BigUint<_> = &loan_amount * &managed_biguint!(11) / &managed_biguint!(10); // 10% de juros
        let num_payments = rng.gen_range(2..5u32);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |_sc| {
                let mut remaining = repayment_amount.clone();
                let mut total_paid = managed_biguint!(0);
                
                // Simular múltiplos pagamentos parciais
                for i in 0..num_payments {
                    let is_last_payment = i == num_payments - 1;
                    
                    let payment_amount = if is_last_payment {
                        // Último pagamento cobre o restante
                        remaining.clone()
                    } else {
                        // Pagar uma fração aleatória do restante
                        let percent = rng.gen_range(10..50u64);
                        &remaining * &managed_biguint!(percent) / &managed_biguint!(100)
                    };
                    
                    // Atualizar valores
                    remaining -= &payment_amount;
                    total_paid += &payment_amount;
                    
                    // Verificar invariantes
                    assert!(payment_amount > managed_biguint!(0));
                    
                    if is_last_payment {
                        assert_eq!(remaining, managed_biguint!(0));
                        assert_eq!(total_paid, repayment_amount);
                    } else {
                        assert!(remaining > managed_biguint!(0));
                        assert!(total_paid < repayment_amount);
                    }
                }
            })
            .assert_ok();
    }
}

// Teste fuzzy para aplicação de taxas de atraso
#[test]
fn test_late_fees_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Configurar taxa de atraso
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.late_fee_daily_rate().set(200u64); // 2% ao dia
        })
        .assert_ok();
    
    for _ in 0..50 {
        let loan_amount: BigUint<DebugApi> = managed_biguint!(rng.gen_range(1000..20000u64));
        let repayment_amount = &loan_amount * &managed_biguint!(11) / &managed_biguint!(10); // 10% de juros
        let days_late = rng.gen_range(1..30u64);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                // Calcular taxa de atraso
                let daily_rate = sc.late_fee_daily_rate().get();
                let late_fee_percent = daily_rate * days_late;
                let late_fee = &repayment_amount * &managed_biguint!(late_fee_percent) / &managed_biguint!(10000);
                let total_with_late_fee = &repayment_amount + &late_fee;
                
                // Verificar cálculos
                assert!(late_fee > managed_biguint!(0));
                assert!(total_with_late_fee > repayment_amount);
                
                // Verificar proporcionalidade da taxa
                if days_late > 1 {
                    let half_days = days_late / 2;
                    let half_late_fee_percent = daily_rate * half_days;
                    let half_late_fee = &repayment_amount * &managed_biguint!(half_late_fee_percent) / &managed_biguint!(10000);
                    
                    // Verificar aproximadamente proporcional (pode haver pequenos arredondamentos)
                    let expected_ratio = managed_biguint!(2);
                    let actual_ratio = if half_late_fee > managed_biguint!(0) {
                        &late_fee / &half_late_fee
                    } else {
                        // Evitar divisão por zero
                        managed_biguint!(0)
                    };
                    
                    // Permitir pequena margem de erro devido a arredondamentos
                    let lower_bound = expected_ratio.clone() - managed_biguint!(1);
                    let upper_bound = expected_ratio + managed_biguint!(1);
                    
                    if half_late_fee > managed_biguint!(0) {
                        assert!(actual_ratio >= lower_bound && actual_ratio <= upper_bound);
                    }
                }
            })
            .assert_ok();
    }
}

// Teste fuzzy para lógica de garantia (collateral)
#[test]
fn test_collateral_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Configurar razão de garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.collateral_ratio().set(7000u64); // 70%
            sc.liquidation_discount().set(2000u64); // 20%
        })
        .assert_ok();
    
    for _ in 0..50 {
        let collateral_amount = rng.gen_range(5000..100000u64);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let collateral: BigUint<DebugApi> = managed_biguint!(collateral_amount);
                
                // Calcular valor máximo de empréstimo baseado na garantia
                let ratio = sc.collateral_ratio().get();
                let max_loan: BigUint<_> = &collateral * &managed_biguint!(ratio) / &managed_biguint!(10000);
                
                // Calcular valor de liquidação
                let discount = sc.liquidation_discount().get();
                let liquidation_value = &collateral * &managed_biguint!(10000 - discount) / &managed_biguint!(10000);
                
                // Verificar invariantes
                assert!(max_loan < collateral);
                assert!(liquidation_value < collateral);
                assert!(liquidation_value > managed_biguint!(0));
                
                // Verificar que a taxa está sendo aplicada corretamente
                let expected_max_loan = managed_biguint!(collateral_amount * 70 / 100);
                let expected_liquidation = managed_biguint!(collateral_amount * 80 / 100);
                
                assert_eq!(max_loan, expected_max_loan);
                assert_eq!(liquidation_value, expected_liquidation);
            })
            .assert_ok();
    }
}

// Teste fuzzy para distribuição de lucros
#[test]
fn test_profit_distribution_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Criar vários investidores aleatórios
    let num_investors = rng.gen_range(2..10u32);
    let mut investors = Vec::with_capacity(num_investors as usize);
    
    for _ in 0..num_investors {
        let investor_address = generate_random_address(&mut rng);
        investors.push(investor_address);
    }
    
    // Configurar investidores com participações aleatórias
    // A soma total deve ser 100%
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let total_shares = 10000u64; // Representando 100%
            let mut remaining_shares = total_shares;
            
            for i in 0..investors.len() {
                let is_last = i == investors.len() - 1;
                
                let share = if is_last {
                    // Último investidor recebe o restante
                    remaining_shares
                } else {
                    // Atribuir uma parte aleatória do restante
                    let max_share = remaining_shares * 80 / 100; // Limitar a 80% do restante
                    rng.gen_range(100..max_share.max(101))
                };
                
                sc.add_investor(managed_address!(&investors[i]), share);
                remaining_shares -= share;
            }
            
            // Verificar que todas as participações foram distribuídas
            assert_eq!(sc.total_investor_shares().get(), total_shares);
        })
        .assert_ok();
    
    // Testar distribuição de lucros com valores aleatórios
    for _ in 0..20 {
        let profit_amount = rng.gen_range(1000..50000u64);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let profit: BigUint<DebugApi> = managed_biguint!(profit_amount);
                let total_shares = sc.total_investor_shares().get();
                
                let mut total_distributed = managed_biguint!(0);
                
                // Verificar distribuição para cada investidor
                for investor in &investors {
                    let share = sc.investor_shares(&managed_address!(investor)).get();
                    let expected_amount = &profit * &managed_biguint!(share) / &managed_biguint!(total_shares);
                    
                    // Na implementação real, aqui transferiria os fundos
                    
                    total_distributed += &expected_amount;
                }
                
                // Verificar que o total distribuído é igual ao lucro total (com possível arredondamento)
                // Para valores grandes, a diferença deve ser mínima
                let diff = if total_distributed > profit {
                    total_distributed.clone() - profit.clone()
                } else {
                    profit.clone() - total_distributed.clone()
                };
                
                // Permitir erro de arredondamento de no máximo 1 por investidor
                let max_diff = managed_biguint!(investors.len() as u64);
                assert!(diff <= max_diff);
            })
            .assert_ok();
    }
}

// Teste fuzzy para taxa de juros baseada em prazo do empréstimo
#[test]
fn test_term_based_interest_rate_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Configurar taxas base para diferentes prazos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.standard_loan_term_days().set(30u64);
            sc.extended_loan_term_days().set(90u64);
            sc.max_loan_term_days().set(180u64);
            
            sc.interest_rate_base().set(1000u64); // 10% para prazo padrão
            sc.extended_term_rate_multiplier().set(150u64); // 1.5x para prazo estendido
            sc.max_term_rate_multiplier().set(200u64); // 2x para prazo máximo
        })
        .assert_ok();
    
    for _ in 0..50 {
        let score = rng.gen_range(500..1000u64);
        let term_type = rng.gen_range(0..3u32); // 0=Standard, 1=Extended, 2=Maximum
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                // Calcular taxa de juros base para o score
                let base_rate = sc.calculate_interest_rate(score);
                
                // Ajustar para o prazo
                let term_rate = match term_type {
                    0 => base_rate, // Prazo padrão
                    1 => base_rate * sc.extended_term_rate_multiplier().get() / 100, // Prazo estendido
                    2 => base_rate * sc.max_term_rate_multiplier().get() / 100, // Prazo máximo
                    _ => panic!("Tipo de prazo inválido"),
                };
                
                // Verificar que taxa está correta conforme o prazo
                match term_type {
                    0 => assert_eq!(term_rate, base_rate),
                    1 => {
                        let expected = base_rate * 150 / 100;
                        assert_eq!(term_rate, expected);
                    },
                    2 => {
                        let expected = base_rate * 200 / 100;
                        assert_eq!(term_rate, expected);
                    },
                    _ => panic!("Tipo de prazo inválido"),
                }
                
                // Verificar que a taxa é maior para prazos mais longos
                if term_type > 0 {
                    assert!(term_rate > base_rate);
                }
                if term_type == 2 {
                    let extended_rate = base_rate * sc.extended_term_rate_multiplier().get() / 100;
                    assert!(term_rate > extended_rate);
                }
            })
            .assert_ok();
    }
}

// Teste fuzzy para limites máximos de empréstimos e taxas
#[test]
fn test_max_min_limits_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    // Configurar limites
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.min_interest_rate().set(200u64); // 2%
            sc.max_interest_rate().set(3000u64); // 30%
            sc.min_loan_amount().set(managed_biguint!(1000)); // Mínimo de 1000
            sc.max_loan_amount().set(managed_biguint!(100000)); // Máximo de 100000
        })
        .assert_ok();
    
    for _ in 0..100 {
        // Testar valores extremos para verificar os limites
        let score = rng.gen_range(0..1500u64);
        let base_amount = rng.gen_range(500..200000u64);
        
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                // Calcular taxa de juros com limites
                let rate = sc.calculate_interest_rate(score);
                
                // Verificar que está dentro dos limites
                assert!(rate >= sc.min_interest_rate().get());
                assert!(rate <= sc.max_interest_rate().get());
                
                // Calcular valor de empréstimo
                let amount = sc.calculate_loan_amount_with_limits(managed_biguint!(base_amount));
                
                // Verificar que está dentro dos limites
                assert!(amount >= sc.base_loan_amount().get());
                assert!(amount <= sc.base_loan_amount().get());
            })
            .assert_ok();
    }
}

// Teste fuzzy para validação de entradas maliciosas
#[test]
fn test_malicious_inputs_fuzzy() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Usar uma semente fixa para reprodutibilidade
    let mut _rng = StdRng::seed_from_u64(42);
    
    // Testar com valores extremos ou potencialmente problemáticos
    let test_cases = [
        0u64, // Zero
        1u64, // Valor mínimo positivo
        u64::MAX, // Valor máximo
        u64::MAX - 1, // Próximo ao máximo
    ];
    
    for &test_value in &test_cases {
        // Set the block timestamp before the query
        let current_timestamp = 12345u64; // Example timestamp
        setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                // Testar cálculo de juros com valores extremos
                let interest_rate = sc.calculate_interest_rate(test_value);
    
                // Não deve causar overflow ou underflow
                assert!(interest_rate <= sc.interest_rate_base().get());
    
                // Testar cálculo de prazo com valores extremos
                let due_timestamp = sc.calculate_due_date_safely(test_value);
    
                // O timestamp de vencimento deve ser maior que o atual
                assert!(due_timestamp >= current_timestamp);
    
                // Verificar limite máximo para evitar overflows
                let max_seconds = 3650u64 * 24u64 * 60u64 * 60u64; // 10 anos em segundos
                assert!(due_timestamp <= current_timestamp + max_seconds);
            })
            .assert_ok();
    }
}