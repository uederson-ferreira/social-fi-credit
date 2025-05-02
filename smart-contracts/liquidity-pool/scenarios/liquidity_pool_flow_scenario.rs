// ==========================================================================
// ARQUIVO: liquidity_pool_flow_scenario.rs
// Descrição: Cenários de fluxo completo para o contrato LiquidityPool
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

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
    pub provider1_address: Address,
    pub provider2_address: Address,
    pub borrower_address: Address,
    pub contract_wrapper: ContractObjWrapper<liquidity_pool::ContractObj<DebugApi>, ContractObjBuilder>,
}

// Função de configuração para os testes
fn setup_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
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
    let provider1_address = blockchain_wrapper.create_user_account(&rust_biguint!(100000));
    let provider2_address = blockchain_wrapper.create_user_account(&rust_biguint!(200000));
    let borrower_address = blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    
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
                managed_address!(&debt_token_address),
                managed_address!(&lp_token_address),
                1000u64, // Taxa de juros base (10%)
                2000u64, // Taxa máxima de utilização (20%)
                8000u64, // Meta de utilização (80%)
            );
        })
        .assert_ok();
    
    ContractSetup {
        blockchain_wrapper,
        owner_address,
        loan_controller_address,
        debt_token_address,
        lp_token_address,
        provider1_address,
        provider2_address,
        borrower_address,
        contract_wrapper,
    }
}

// Cenário: Ciclo completo de depósito, empréstimo, pagamento e retirada
#[test]
fn test_complete_lifecycle_scenario() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Etapa 1: Depósito de liquidez pelos provedores
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(60000), |sc| {
            sc.deposit();
            
            // Simular emissão de tokens LP
            sc.lp_tokens_minted(managed_address!(&setup.provider1_address), managed_biguint!(60000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider2_address, &setup.contract_wrapper, &rust_biguint!(40000), |sc| {
            sc.deposit();
            
            // Simular emissão de tokens LP
            sc.lp_tokens_minted(managed_address!(&setup.provider2_address), managed_biguint!(40000));
        })
        .assert_ok();
    
    // Verificar estado após depósitos
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(100000));
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(60000));
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider2_address)).get(), managed_biguint!(40000));
        })
        .assert_ok();
    
    // Etapa 2: Empréstimo pelos usuários
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Calcular taxa de juros atual (deve ser menor que a base por baixa utilização)
            let interest_rate = sc.calculate_current_interest_rate();
            assert!(interest_rate < 1000u64);
            
            // Fazer empréstimo
            sc.borrow(
                managed_address!(&setup.borrower_address),
                managed_biguint!(70000),
                interest_rate
            );
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted(managed_address!(&setup.borrower_address), managed_biguint!(70000));
            
            // Verificar estado
            assert_eq!(sc.total_borrows().get(), managed_biguint!(70000));
            assert_eq!(sc.utilization_rate().get(), 7000u64); // 70%
        })
        .assert_ok();
    
    // Etapa 3: Acúmulo de juros
    // Simular o tempo passando e os juros acumulando (8% do empréstimo = 5600)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(5600), |sc| {
            sc.add_accumulated_interest(managed_biguint!(5600));
            
            // Verificar juros acumulados
            assert_eq!(sc.total_interest_accumulated().get(), managed_biguint!(5600));
        })
        .assert_ok();
    
    // Etapa 4: Distribuição de juros
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Distribuir juros
            sc.distribute_interest();
            
            // Verificar distribuição
            // 20% para reservas = 1120
            assert_eq!(sc.total_reserves().get(), managed_biguint!(1120));
            
            // 80% para provedores = 4480
            // Provider 1 (60%): 4480 * 0.6 = 2688
            // Provider 2 (40%): 4480 * 0.4 = 1792
            assert_eq!(sc.provider_interest(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(2688));
            assert_eq!(sc.provider_interest(&managed_address!(&setup.provider2_address)).get(), managed_biguint!(1792));
        })
        .assert_ok();
    
    // Etapa 5: Pagamento parcial do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(30000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned(managed_address!(&setup.borrower_address), managed_biguint!(30000));
            
            sc.repay();
            
            // Verificar estado
            assert_eq!(sc.total_borrows().get(), managed_biguint!(40000));
            assert_eq!(sc.utilization_rate().get(), 4000u64); // 40%
        })
        .assert_ok();
    
    // Etapa 6: Retirada parcial de liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular queima de tokens LP
            sc.lp_tokens_burned(managed_address!(&setup.provider1_address), managed_biguint!(20000));
            
            // Retirar
            sc.withdraw(managed_biguint!(20000));
            
            // Verificar estado
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(80000));
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(40000));
        })
        .assert_ok();
    
    // Etapa 7: Pagamento total do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(40000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned(managed_address!(&setup.borrower_address), managed_biguint!(40000));
            
            sc.repay();
            
            // Verificar estado
            assert_eq!(sc.total_borrows().get(), managed_biguint!(0));
            assert_eq!(sc.borrower_debt(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(0));
            assert_eq!(sc.utilization_rate().get(), 0u64); // 0%
        })
        .assert_ok();
    
    // Etapa 8: Retirada total da liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Retirar saldo + juros
            let balance = sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get();
            let interest = sc.provider_interest(&managed_address!(&setup.provider1_address)).get();
            let total_withdrawal = &balance + &interest;
            
            // Simular queima de tokens LP
            sc.lp_tokens_burned(managed_address!(&setup.provider1_address), balance.clone());
            
            // Retirar saldo
            sc.withdraw(balance);
            
            // Retirar juros
            sc.withdraw_interest();
            
            // Verificar estado
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(0));
            assert_eq!(sc.provider_interest(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(0));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider2_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Retirar saldo + juros
            let balance = sc.provider_liquidity(&managed_address!(&setup.provider2_address)).get();
            let interest = sc.provider_interest(&managed_address!(&setup.provider2_address)).get();
            
            // Simular queima de tokens LP
            sc.lp_tokens_burned(managed_address!(&setup.provider2_address), balance.clone());
            
            // Retirar saldo
            sc.withdraw(balance);
            
            // Retirar juros
            sc.withdraw_interest();
            
            // Verificar estado
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider2_address)).get(), managed_biguint!(0));
            assert_eq!(sc.provider_interest(&managed_address!(&setup.provider2_address)).get(), managed_biguint!(0));
        })
        .assert_ok();
    
    // Verificar estado final (só restam as reservas)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(0));
            assert_eq!(sc.total_borrows().get(), managed_biguint!(0));
            assert_eq!(sc.total_reserves().get(), managed_biguint!(1120)); // Reservas permanecem
        })
        .assert_ok();
}

// Cenário de emergência no mercado
#[test]
fn test_emergency_market_scenario() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Etapa 1: Setup inicial com liquidez e empréstimos
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(80000), |sc| {
            sc.deposit();
            // Simular emissão de tokens LP
            sc.lp_tokens_minted(managed_address!(&setup.provider1_address), managed_biguint!(80000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider2_address, &setup.contract_wrapper, &rust_biguint!(120000), |sc| {
            sc.deposit();
            // Simular emissão de tokens LP
            sc.lp_tokens_minted(managed_address!(&setup.provider2_address), managed_biguint!(120000));
        })
        .assert_ok();
    
    // Realizar empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow(
                managed_address!(&setup.borrower_address),
                managed_biguint!(150000),
                1000u64
            );
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted(managed_address!(&setup.borrower_address), managed_biguint!(150000));
        })
        .assert_ok();
    
    // Etapa 2: Detecção de condições de mercado adversas
    // Simular um evento externo que causa pânico no mercado
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Ativar modo de emergência no contrato
            sc.activate_emergency_mode();
            assert!(sc.emergency_mode().get());
            
            // Pausar o contrato para evitar mais operações
            sc.pause();
            assert!(sc.is_paused().get());
            
            // Aumentar drasticamente as taxas para desencorajar novos empréstimos
            // quando o contrato for despausado
            sc.set_interest_rate_base(3000u64); // Triplo da taxa normal
            sc.set_max_utilization_rate(1000u64); // Reduzir limite máximo de utilização
            
            // Impor limite de retirada para evitar corrida aos saques
            sc.set_max_withdrawal_limit(2000u64); // 20% máximo por dia
        })
        .assert_ok();
    
    // Etapa 3: Tentativa de saque em massa (deve ser limitada)
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que o contrato está pausado
            assert!(sc.is_paused().get());
            
            // Na implementação real, qualquer tentativa de saque falharia
            // "Contract is paused"
        })
        .assert_ok();
    
    // Etapa 4: Implementação de protocolo de emergência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Ativar plano de recuperação
            sc.activate_recovery_plan();
            
            // Impor taxa de saída para retiradas (para desencorajar saques)
            sc.set_withdrawal_fee(500u64); // 5% de taxa de saída
            
            // Despausar o contrato com as novas restrições
            sc.unpause();
            assert!(!sc.is_paused().get());
        })
        .assert_ok();
    
    // Etapa 5: Retirada controlada
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar saldo atual
            let balance = sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get();
            assert_eq!(balance, managed_biguint!(80000));
            
            // Calcular valor máximo de retirada permitido
            let max_withdrawal_limit = sc.max_withdrawal_limit().get();
            let max_amount = &balance * &managed_biguint!(max_withdrawal_limit) / &managed_biguint!(10000);
            assert_eq!(max_amount, managed_biguint!(16000)); // 20% de 80000
            
            // Simular queima de tokens LP
            sc.lp_tokens_burned(managed_address!(&setup.provider1_address), max_amount.clone());
            
            // Retirar valor permitido
            sc.withdraw(max_amount.clone());
            
            // Verificar taxa de saída aplicada
            // 16000 * 5% = 800 de taxa
            assert_eq!(sc.withdrawal_fees_collected().get(), managed_biguint!(800));
            
            // Verificar saldo atualizado
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(64000));
        })
        .assert_ok();
    
    // Etapa 6: Gradual retorno à normalidade
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular melhora nas condições de mercado
            // Reduzir gradualmente as restrições
            
            // Aumentar limite de retirada
            sc.set_max_withdrawal_limit(5000u64); // 50%
            
            // Reduzir taxa de saída
            sc.set_withdrawal_fee(200u64); // 2%
            
            // Reduzir taxas de juros gradualmente
            sc.set_interest_rate_base(2000u64); // 20% -> Ainda alto mas menor
        })
        .assert_ok();
    
    // Etapa 7: Normalização completa
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Desativar modo de emergência
            sc.deactivate_emergency_mode();
            assert!(!sc.emergency_mode().get());
            
            // Restaurar parâmetros normais
            sc.set_max_withdrawal_limit(10000u64); // 100% - sem limite
            sc.set_withdrawal_fee(0u64); // Sem taxa de saída
            sc.set_interest_rate_base(1000u64); // Voltar à taxa original
            sc.set_max_utilization_rate(2000u64); // Restaurar limite original
            
            // Desativar plano de recuperação
            sc.deactivate_recovery_plan();
        })
        .assert_ok();
    
    // Etapa 8: Retiradas normais após crise
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Retirar saldo restante
            let balance = sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get();
            
            // Simular queima de tokens LP
            sc.lp_tokens_burned(managed_address!(&setup.provider1_address), balance.clone());
            
            // Retirar sem restrições
            sc.withdraw(balance);
            
            // Verificar saldo zerado
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(0));
            
            // Verificar que não há mais taxa de saída
            let initial_fees = sc.withdrawal_fees_collected().get();
            assert_eq!(initial_fees, managed_biguint!(800)); // Não mudou
        })
        .assert_ok();
}

// Cenário de ajuste automático de taxas
#[test]
fn test_dynamic_rate_adjustment_scenario() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Etapa 1: Configuração inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar parâmetros para ajuste dinâmico
            sc.set_interest_rate_base(1000u64); // 10% taxa base
            sc.set_target_utilization_rate(8000u64); // 80% meta de utilização
            sc.set_max_utilization_rate(2000u64); // 20% taxa adicional no máximo
            
            // Configurar multiplicadores para ajuste
            sc.set_rate_adjustment_speed(500u64); // Velocidade de ajuste 5%
            sc.enable_dynamic_rate_adjustment();
        })
        .assert_ok();
    
    // Etapa 2: Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit();
            // Simular emissão de tokens LP
            sc.lp_tokens_minted(managed_address!(&setup.provider1_address), managed_biguint!(100000));
        })
        .assert_ok();
    
    // Etapa 3: Primeiro empréstimo (baixa utilização)
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Utilização de 30% - abaixo da meta
            sc.borrow(
                managed_address!(&setup.borrower_address),
                managed_biguint!(30000),
                1000u64 // Taxa base inicial
            );
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted(managed_address!(&setup.borrower_address), managed_biguint!(30000));
            
            // Verificar taxa de utilização
            assert_eq!(sc.utilization_rate().get(), 3000u64); // 30%
        })
        .assert_ok();
    
    // Etapa 4: Primeiro ajuste - reduzir a taxa para incentivar empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Executar ajuste automático de taxa
            sc.adjust_interest_rate_automatically();
            
            // Calcular taxa esperada
            // utilização atual (30%) vs meta (80%) = déficit de 50%
            // ajuste = 5% * 50% = 2.5% para baixo
            // Nova taxa = 10% - 2.5% = 7.5%
            let expected_new_rate = 750u64; // 7.5%
            
            // Verificar nova taxa
            assert_eq!(sc.interest_rate_base().get(), expected_new_rate);
        })
        .assert_ok();
    
    // Etapa 5: Segundo empréstimo com a nova taxa reduzida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emprestar mais 40% do pool
            sc.borrow(
                managed_address!(&setup.borrower_address),
                managed_biguint!(40000),
                750u64 // Taxa reduzida
            );
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted(managed_address!(&setup.borrower_address), managed_biguint!(40000));
            
            // Verificar taxa de utilização
            assert_eq!(sc.utilization_rate().get(), 7000u64); // 70%
        })
        .assert_ok();
    
    // Etapa 6: Segundo ajuste - taxa ainda reduzida mas menos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Executar ajuste automático de taxa
            sc.adjust_interest_rate_automatically();
            
            // Calcular taxa esperada
            // utilização atual (70%) vs meta (80%) = déficit de 10%
            // ajuste = 5% * 10% = 0.5% para baixo
            // Nova taxa = 7.5% - 0.5% = 7%
            let expected_new_rate = 700u64; // 7%
            
            // Verificar nova taxa
            assert_eq!(sc.interest_rate_base().get(), expected_new_rate);
        })
        .assert_ok();
    
    // Etapa 7: Terceiro empréstimo - ultrapassando a meta
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emprestar mais 20% do pool (total 90%)
            sc.borrow(
                managed_address!(&setup.borrower_address),
                managed_biguint!(20000),
                700u64 // Taxa atual
            );
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted(managed_address!(&setup.borrower_address), managed_biguint!(20000));
            
            // Verificar taxa de utilização
            assert_eq!(sc.utilization_rate().get(), 9000u64); // 90%
        })
        .assert_ok();
    
    // Etapa 8: Terceiro ajuste - aumentar a taxa para desincentivar empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Executar ajuste automático de taxa
            sc.adjust_interest_rate_automatically();
            
            // Calcular taxa esperada
            // utilização atual (90%) vs meta (80%) = excesso de 10%
            // ajuste = 5% * 10% = 0.5% para cima
            // Nova taxa = 7% + 0.5% = 7.5%
            let expected_new_rate = 750u64; // 7.5%
            
            // Verificar nova taxa
            assert_eq!(sc.interest_rate_base().get(), expected_new_rate);
        })
        .assert_ok();
    
    // Etapa 9: Pagamento de parte do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned(managed_address!(&setup.borrower_address), managed_biguint!(50000));
            
            sc.repay();
            
            // Verificar nova taxa de utilização
            assert_eq!(sc.utilization_rate().get(), 4000u64); // 40%
        })
        .assert_ok();
    
    // Etapa 10: Quarto ajuste - reduzir novamente a taxa
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Executar ajuste automático de taxa
            sc.adjust_interest_rate_automatically();
            
            // Calcular taxa esperada
            // utilização atual (40%) vs meta (80%) = déficit de 40%
            // ajuste = 5% * 40% = 2% para baixo
            // Nova taxa = 7.5% - 2% = 5.5%
            let expected_new_rate = 550u64; // 5.5%
            
            // Verificar nova taxa
            assert_eq!(sc.interest_rate_base().get(), expected_new_rate);
        })
        .assert_ok();
}

// Cenário de liquidação em caso de crise
#[test]
fn test_liquidation_crisis_scenario() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Etapa 1: Configurar o pool e fazer empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit();
            // Simular emissão de tokens LP
            sc.lp_tokens_minted(managed_address!(&setup.provider1_address), managed_biguint!(100000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider2_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit();
            // Simular emissão de tokens LP
            sc.lp_tokens_minted(managed_address!(&setup.provider2_address), managed_biguint!(50000));
        })
        .assert_ok();
    
    // Realizar empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emprestar 70% do pool
            sc.borrow(
                managed_address!(&setup.borrower_address),
                managed_biguint!(105000),
                1000u64
            );
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted(managed_address!(&setup.borrower_address), managed_biguint!(105000));
            
            // Verificar taxa de utilização
            assert_eq!(sc.utilization_rate().get(), 7000u64); // 70%
        })
        .assert_ok();
    
    // Etapa 2: Simular uma queda no valor da garantia dos empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Ativar modo de crise
            sc.activate_crisis_mode();
            
            // Registrar perda de valor nas garantias
            sc.record_collateral_value_loss(3000u64); // Perda de 30%
        })
        .assert_ok();
    
    // Etapa 3: Iniciar protocolo de liquidação
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Pausar o contrato para evitar mais operações
            sc.pause();
            
            // Calcular valor recuperável das garantias
            // 105000 emprestados - 30% de perda = 73500 recuperáveis
            let recoverable_value = managed_biguint!(73500);
            
            // Registrar perda total
            let total_loss = managed_biguint!(31500); // 105000 - 73500
            sc.register_bad_debt(total_loss.clone());
            
            // Verificar dívida incobrável registrada
            assert_eq!(sc.total_bad_debt().get(), total_loss);
        })
        .assert_ok();
    
    // Etapa 4: Liquidação das garantias
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(73500), |sc| {
            // Simular liquidação das garantias e recuperação de valor
            sc.process_liquidation_proceeds(managed_biguint!(73500));
            
            // Verificar que o valor foi recebido
            assert_eq!(sc.liquidation_proceeds().get(), managed_biguint!(73500));
            
            // Atualizar empréstimos para refletir a liquidação
            sc.debt_tokens_burned(managed_address!(&setup.borrower_address), managed_biguint!(105000));
            sc.write_off_loan(managed_address!(&setup.borrower_address));
            
            // Verificar que os empréstimos foram zerados
            assert_eq!(sc.total_borrows().get(), managed_biguint!(0));
            assert_eq!(sc.borrower_debt(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(0));
        })
        .assert_ok();
    
    // Etapa 5: Cálculo e aplicação de haircut aos provedores
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Calcular porcentagem de perda para os provedores
            // Total liquidez: 150000, Perda: 31500
            // Haircut: 31500 / 150000 = 21%
            let haircut_percent = 2100u64; // 21%
            
            // Aplicar haircut ao Provider 1 (100000 * 21% = 21000)
            let provider1_haircut = managed_biguint!(21000);
            let provider1_new_balance = managed_biguint!(79000); // 100000 - 21000
            sc.apply_provider_haircut(
                managed_address!(&setup.provider1_address),
                provider1_haircut
            );
            
            // Aplicar haircut ao Provider 2 (50000 * 21% = 10500)
            let provider2_haircut = managed_biguint!(10500);
            let provider2_new_balance = managed_biguint!(39500); // 50000 - 10500
            sc.apply_provider_haircut(
                managed_address!(&setup.provider2_address),
                provider2_haircut
            );
            
            // Verificar novos saldos
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), provider1_new_balance);
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider2_address)).get(), provider2_new_balance);
            
            // Verificar liquidez total atualizada (150000 - 31500 = 118500)
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(118500));
        })
        .assert_ok();
    
    // Etapa 6: Reabrir o pool após crise
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Desativar modo de crise
            sc.deactivate_crisis_mode();
            
            // Despausar o contrato
            sc.unpause();
            
            // Redefinir parâmetros para serem mais conservadores
            sc.set_max_utilization_rate(1500u64); // Reduzir limite para 15%
            sc.set_target_utilization_rate(7000u64); // Reduzir meta para 70%
            
            // Exigir mais garantia para novos empréstimos
            sc.set_collateral_requirement(15000u64); // 150% de garantia
        })
        .assert_ok();
    
    // Etapa 7: Provedores podem retirar fundos com o haircut aplicado
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar saldo atual
            let balance = sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get();
            assert_eq!(balance, managed_biguint!(79000));
            
            // Simular queima de tokens LP
            sc.lp_tokens_burned(managed_address!(&setup.provider1_address), balance.clone());
            
            // Retirar saldo
            sc.withdraw(balance);
            
            // Verificar saldo zerado
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(0));
        })
        .assert_ok();
}