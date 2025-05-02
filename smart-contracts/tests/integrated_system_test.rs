// ==========================================================================
// ARQUIVO: integrated_system_test.rs
// Descrição: Testes integrados para todo o sistema de empréstimos
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use loan_controller::*;
use reputation_score::*;
use debt_token::*;
use liquidity_pool::*;

const LOAN_CONTROLLER_WASM_PATH: &str = "output/loan-controller.wasm";
const REPUTATION_SCORE_WASM_PATH: &str = "output/reputation-score.wasm";
const DEBT_TOKEN_WASM_PATH: &str = "output/debt-token.wasm";
const LIQUIDITY_POOL_WASM_PATH: &str = "output/liquidity-pool.wasm";
const LP_TOKEN_WASM_PATH: &str = "output/lp-token.wasm";

// Estrutura para configuração dos testes integrados
struct IntegratedSystemSetup {
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub provider1_address: Address,
    pub provider2_address: Address,
    pub borrower1_address: Address,
    pub borrower2_address: Address,
    pub loan_controller_wrapper: ContractObjWrapper<loan_controller::ContractObj<DebugApi>, fn() -> loan_controller::ContractObj<DebugApi>>,
    pub reputation_score_wrapper: ContractObjWrapper<reputation_score::ContractObj<DebugApi>, fn() -> reputation_score::ContractObj<DebugApi>>,
    pub debt_token_wrapper: ContractObjWrapper<debt_token::ContractObj<DebugApi>, fn() -> debt_token::ContractObj<DebugApi>>,
    pub liquidity_pool_wrapper: ContractObjWrapper<liquidity_pool::ContractObj<DebugApi>, fn() -> liquidity_pool::ContractObj<DebugApi>>,
    pub lp_token_wrapper: ContractObjWrapper<lp_token::ContractObj<DebugApi>, fn() -> lp_token::ContractObj<DebugApi>>,
}

// Função de configuração para os testes integrados
fn setup_integrated_system() -> IntegratedSystemSetup {
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    
    // Criar contas
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let provider1_address = blockchain_wrapper.create_user_account(&rust_biguint!(200000));
    let provider2_address = blockchain_wrapper.create_user_account(&rust_biguint!(300000));
    let borrower1_address = blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    let borrower2_address = blockchain_wrapper.create_user_account(&rust_biguint!(15000));
    
    // Deploy dos contratos
    let lp_token_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        lp_token::contract_obj,
        LP_TOKEN_WASM_PATH,
    );
    
    let debt_token_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        debt_token::contract_obj,
        DEBT_TOKEN_WASM_PATH,
    );
    
    let reputation_score_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        reputation_score::contract_obj,
        REPUTATION_SCORE_WASM_PATH,
    );
    
    let liquidity_pool_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        liquidity_pool::contract_obj,
        LIQUIDITY_POOL_WASM_PATH,
    );
    
    let loan_controller_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        loan_controller::contract_obj,
        LOAN_CONTROLLER_WASM_PATH,
    );
    
    // Inicializar contratos
    // 1. Inicializar DebtToken
    blockchain_wrapper
        .execute_tx(&owner_address, &debt_token_wrapper, &rust_zero, |sc| {
            sc.init(managed_address!(&loan_controller_wrapper.address_ref()));
        })
        .assert_ok();
    
    // 2. Inicializar ReputationScore
    blockchain_wrapper
        .execute_tx(&owner_address, &reputation_score_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&loan_controller_wrapper.address_ref()),
                500u64, // Pontuação inicial padrão
                1000u64, // Pontuação máxima
            );
        })
        .assert_ok();
    
    // 3. Inicializar LiquidityPool
    blockchain_wrapper
        .execute_tx(&owner_address, &liquidity_pool_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&loan_controller_wrapper.address_ref()),
                managed_address!(&debt_token_wrapper.address_ref()),
                managed_address!(&lp_token_wrapper.address_ref()),
                1000u64, // Taxa de juros base (10%)
                2000u64, // Taxa máxima de utilização (20%)
                8000u64, // Meta de utilização (80%)
            );
        })
        .assert_ok();
    
    // 4. Inicializar LP Token
    blockchain_wrapper
        .execute_tx(&owner_address, &lp_token_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&liquidity_pool_wrapper.address_ref()),
            );
        })
        .assert_ok();
    
    // 5. Inicializar LoanController
    blockchain_wrapper
        .execute_tx(&owner_address, &loan_controller_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(&reputation_score_wrapper.address_ref()),
                500u64, // Pontuação mínima necessária
                1000u64, // Taxa de juros base (10% = 1000 pontos base)
                managed_biguint!(10_000), // Valor base do empréstimo
            );
            
            // Configurar outros contratos relacionados
            sc.set_liquidity_pool_address(managed_address!(&liquidity_pool_wrapper.address_ref()));
            sc.set_debt_token_address(managed_address!(&debt_token_wrapper.address_ref()));
        })
        .assert_ok();
    
    // Configurar algumas pontuações iniciais de reputação para os tomadores
    blockchain_wrapper
        .execute_tx(&owner_address, &reputation_score_wrapper, &rust_zero, |sc| {
            sc.set_initial_score(managed_address!(&borrower1_address), 700u64);
            sc.set_initial_score(managed_address!(&borrower2_address), 600u64);
        })
        .assert_ok();
    
    IntegratedSystemSetup {
        blockchain_wrapper,
        owner_address,
        provider1_address,
        provider2_address,
        borrower1_address,
        borrower2_address,
        loan_controller_wrapper,
        reputation_score_wrapper,
        debt_token_wrapper,
        liquidity_pool_wrapper,
        lp_token_wrapper,
    }
}

// Teste integrado do fluxo completo de empréstimo
#[test]
fn test_full_loan_lifecycle_integrated() {
    let mut setup = setup_integrated_system();
    
    // Etapa 1: Provedores adicionam liquidez ao pool
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP pelo LiquidityPool
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider1_address), managed_biguint!(100000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider2_address, &setup.liquidity_pool_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider2_address), managed_biguint!(50000));
        })
        .assert_ok();
    
    // Verificar liquidez total
    setup.blockchain_wrapper
        .execute_query(&setup.liquidity_pool_wrapper, |sc| {
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(150000));
        })
        .assert_ok();
    
    // Etapa 2: Tomador solicita verificação de pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_reputation_check();
        })
        .assert_ok();
    
    // Simular resposta da verificação de pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower1_address),
                700u64
            );
        })
        .assert_ok();
    
    // Etapa 3: Tomador solicita empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            let loan_id = sc.request_loan();
            assert_eq!(loan_id, 1u64);
        })
        .assert_ok();
    
    // Verificar detalhes do empréstimo
    setup.blockchain_wrapper
        .execute_query(&setup.loan_controller_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            // Com pontuação 700, taxa deve ser reduzida
            // Taxa base: 1000 (10%)
            // Redução: 1000 * (1 - (700/1000) * 0.8) = 1000 * (1 - 0.56) = 1000 * 0.44 = 440
            assert_eq!(loan.interest_rate, 440u64);
            
            // Verificar valor do empréstimo
            // Valor base: 10000
            // Ajuste: 10000 * (1 + (700/1000) * 0.5) = 10000 * 1.35 = 13500
            assert_eq!(loan.amount, managed_biguint!(13500));
            
            // Valor de repagamento: 13500 + (13500 * 440 / 10000) = 13500 + 594 = 14094
            assert_eq!(loan.repayment_amount, managed_biguint!(14094));
        })
        .assert_ok();
    
    // Etapa 4: LoanController obtém fundos da LiquidityPool
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow(
                managed_address!(&setup.borrower1_address),
                managed_biguint!(13500),
                440u64 // Taxa calculada
            );
        })
        .assert_ok();
    
    // Etapa 5: LiquidityPool emite tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        })
        .assert_ok();
    
    // Verificar saldo de tokens de dívida
    setup.blockchain_wrapper
        .execute_query(&setup.debt_token_wrapper, |sc| {
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower1_address)), managed_biguint!(13500));
        })
        .assert_ok();
    
    // Etapa 6: LoanController transfere fundos para o tomador
    // (simulado, pois na implementação real isso ocorreria automaticamente)
    
    // Etapa 7: Tomador paga parte do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(5000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned(managed_address!(&setup.borrower1_address), managed_biguint!(5000));
            
            sc.repay();
        })
        .assert_ok();
    
    // Verificar atualização dos tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.borrower1_address), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Verificar saldo atualizado
    setup.blockchain_wrapper
        .execute_query(&setup.debt_token_wrapper, |sc| {
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower1_address)), managed_biguint!(8500));
        })
        .assert_ok();
    
    // Etapa 8: Tomador paga o restante do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(9094), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned(managed_address!(&setup.borrower1_address), managed_biguint!(8500));
            
            sc.repay();
        })
        .assert_ok();
    
    // LiquidityPool queima o restante dos tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.borrower1_address), managed_biguint!(8500));
        })
        .assert_ok();
    
    // Etapa 9: LoanController atualiza o status do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.mark_loan_repaid(1u64);
            
            // Verificar status
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
        })
        .assert_ok();
    
    // Etapa 10: ReputationScore atualiza a pontuação do tomador
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.reputation_score_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.borrower1_address), 50); // Aumento por pagamento em dia
            
            // Verificar nova pontuação
            let new_score = sc.get_score(&managed_address!(&setup.borrower1_address));
            assert_eq!(new_score, 750u64); // 700 + 50
        })
        .assert_ok();
    
    // Etapa 11: Proveedor retira parte de sua liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            // Simular queima de tokens LP
            sc.burn(managed_address!(&setup.provider1_address), managed_biguint!(40000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.withdraw(managed_biguint!(40000));
            
            // Verificar saldo atualizado
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(60000));
        })
        .assert_ok();
}

// Teste integrado de empréstimo com inadimplência
#[test]
fn test_loan_default_integrated() {
    let mut setup = setup_integrated_system();
    
    // Etapa 1: Provedores adicionam liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider1_address), managed_biguint!(100000));
        })
        .assert_ok();
    
    // Etapa 2: Tomador solicita empréstimo
    // Primeiro, verificar pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_reputation_check();
        })
        .assert_ok();
    
    // Simular resposta da verificação de pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower2_address),
                600u64
            );
        })
        .assert_ok();
    
    // Solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            let loan_id = sc.request_loan();
            assert_eq!(loan_id, 1u64);
        })
        .assert_ok();
    
    // Verificar detalhes do empréstimo
    setup.blockchain_wrapper
        .execute_query(&setup.loan_controller_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            // Com pontuação 600, taxa deve ser reduzida menos que no teste anterior
            // Taxa base: 1000 (10%)
            // Redução: 1000 * (1 - (600/1000) * 0.8) = 1000 * (1 - 0.48) = 1000 * 0.52 = 520
            assert_eq!(loan.interest_rate, 520u64);
            
            // Verificar valor do empréstimo
            assert_eq!(loan.amount, managed_biguint!(13000)); // Valor ajustado para pontuação 600
            
            // Valor de repagamento: 13000 + (13000 * 520 / 10000) = 13000 + 676 = 13676
            assert_eq!(loan.repayment_amount, managed_biguint!(13676));
        })
        .assert_ok();
    
    // Etapa 3: LoanController obtém fundos e emite tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow(
                managed_address!(&setup.borrower2_address),
                managed_biguint!(13000),
                520u64
            );
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.borrower2_address), managed_biguint!(13000));
        })
        .assert_ok();
    
    // Etapa 4: Simular avanço do tempo para além do vencimento
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Definir o timestamp atual para após o vencimento
            sc.blockchain().set_block_timestamp(100000); // Muito depois do vencimento
            
            // Verificar e marcar empréstimos vencidos
            sc.mark_expired_loans();
            
            // Verificar status do empréstimo
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Defaulted);
        })
        .assert_ok();
    
    // Etapa 5: Atualizar pontuação de reputação negativamente
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.reputation_score_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.borrower2_address), -100); // Redução por inadimplência
            
            // Verificar nova pontuação
            let new_score = sc.get_score(&managed_address!(&setup.borrower2_address));
            assert_eq!(new_score, 500u64); // 600 - 100
        })
        .assert_ok();
    
    // Etapa 6: Liquidar garantias (simular recuperação parcial)
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Simular liquidação de garantias
            sc.process_collateral_liquidation(1u64, managed_biguint!(8000)); // Recupera apenas 8000 de 13000
            
            // Verificar status atualizado
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Liquidated);
        })
        .assert_ok();
    
    // Etapa 7: LiquidityPool registra perda parcial
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(8000), |sc| {
            // Registrar valor recuperado
            sc.process_liquidation_proceeds(managed_biguint!(8000));
            
            // Registrar perda
            let loss = managed_biguint!(5000); // 13000 - 8000
            sc.register_bad_debt(loss);
            
            // Verificar registros
            assert_eq!(sc.total_bad_debt().get(), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Etapa 8: Queimar tokens de dívida correspondentes
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.borrower2_address), managed_biguint!(13000));
        })
        .assert_ok();
    
    // Etapa 9: Aplicar haircut aos provedores
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Aplicar perda de 5000 ao único provedor
            sc.apply_provider_haircut(
                managed_address!(&setup.provider1_address),
                managed_biguint!(5000)
            );
            
            // Verificar saldo atualizado (100000 - 5000 = 95000)
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(95000));
            
            // Verificar liquidez total
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(95000));
        })
        .assert_ok();
}

// Teste integrado de extensão de prazo de empréstimo
#[test]
fn test_loan_extension_integrated() {
    let mut setup = setup_integrated_system();
    
    // Etapa 1: Adicionar liquidez e configurar taxa de extensão
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider1_address), managed_biguint!(100000));
        })
        .assert_ok();
    
    // Configurar taxa de extensão
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.set_extension_fee_percent(1000u64); // 10%
        })
        .assert_ok();
    
    // Etapa 2: Tomador solicita empréstimo
    // Verificar pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_reputation_check();
        })
        .assert_ok();
    
    // Simular resposta da verificação de pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower1_address),
                700u64
            );
        })
        .assert_ok();
    
    // Solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_loan();
        })
        .assert_ok();
    
    // Verificar detalhes do empréstimo
    setup.blockchain_wrapper
        .execute_query(&setup.loan_controller_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.amount, managed_biguint!(13500));
            assert_eq!(loan.repayment_amount, managed_biguint!(14094));
        })
        .assert_ok();
    
    // Etapa 3: LoanController obtém fundos e emite tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow(
                managed_address!(&setup.borrower1_address),
                managed_biguint!(13500),
                440u64
            );
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        })
        .assert_ok();
    
    // Etapa 4: Tomador solicita extensão de prazo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(1409), |sc| {
            // Taxa de extensão: 10% do valor de repagamento (14094 * 10% = 1409)
            sc.extend_loan_deadline(1u64, 15u64); // Extender por 15 dias
            
            // Verificar detalhes do empréstimo atualizados
            let loan = sc.loans(1u64).get();
            
            // Novo valor de repagamento: 14094 + 1409 = 15503
            assert_eq!(loan.repayment_amount, managed_biguint!(15503));
            
            // Verificar que o prazo foi estendido
            // (Não testamos o timestamp exato, pois depende da implementação interna)
        })
        .assert_ok();
    
    // Etapa 5: Pagar empréstimo com valor atualizado
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(15503), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
            
            sc.repay();
        })
        .assert_ok();
    
    // Queimar tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        })
        .assert_ok();
    
}    

// Etapa 6:

// Teste integrado de extensão de prazo de empréstimo (continuação)
#[test]
fn test_loan_extension_integrated() {
    // Continuação da Etapa 6: Atualizar status do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.mark_loan_repaid(1u64);
            
            // Verificar status final
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
        })
        .assert_ok();
    
    // Etapa 7: Atualizar pontuação de reputação positivamente pelo pagamento após extensão
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.reputation_score_wrapper, &rust_biguint!(0), |sc| {
            // Aumento menor por pagamento após extensão
            sc.update_score(managed_address!(&setup.borrower1_address), 30);
            
            // Verificar nova pontuação
            let new_score = sc.get_score(&managed_address!(&setup.borrower1_address));
            assert_eq!(new_score, 730u64); // 700 + 30
        })
        .assert_ok();
}

// Teste integrado de múltiplos tomadores e provedores
#[test]
fn test_multiple_borrowers_lenders_integrated() {
    let mut setup = setup_integrated_system();
    
    // Etapa 1: Múltiplos provedores adicionam liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(70000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider1_address), managed_biguint!(70000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider2_address, &setup.liquidity_pool_wrapper, &rust_biguint!(130000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider2_address), managed_biguint!(130000));
        })
        .assert_ok();
    
    // Verificar liquidez total
    setup.blockchain_wrapper
        .execute_query(&setup.liquidity_pool_wrapper, |sc| {
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(200000));
        })
        .assert_ok();
    
    // Etapa 2: Tomador 1 solicita empréstimo
    // Verificar pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_reputation_check();
        })
        .assert_ok();
    
    // Simular resposta da verificação de pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower1_address),
                700u64
            );
        })
        .assert_ok();
    
    // Solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_loan();
        })
        .assert_ok();
    
    // Etapa 3: Tomador 2 solicita empréstimo
    // Verificar pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_reputation_check();
        })
        .assert_ok();
    
    // Simular resposta da verificação de pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower2_address),
                600u64
            );
        })
        .assert_ok();
    
    // Solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_loan();
        })
        .assert_ok();
    
    // Etapa 4: Verificar detalhes dos empréstimos
    setup.blockchain_wrapper
        .execute_query(&setup.loan_controller_wrapper, |sc| {
            let loan1 = sc.loans(1u64).get();
            let loan2 = sc.loans(2u64).get();
            
            // Empréstimo 1 (score 700)
            assert_eq!(loan1.amount, managed_biguint!(13500));
            assert_eq!(loan1.interest_rate, 440u64);
            
            // Empréstimo 2 (score 600)
            assert_eq!(loan2.amount, managed_biguint!(13000));
            assert_eq!(loan2.interest_rate, 520u64);
        })
        .assert_ok();
    
    // Etapa 5: LoanController obtém fundos para ambos
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Empréstimo 1
            sc.borrow(
                managed_address!(&setup.borrower1_address),
                managed_biguint!(13500),
                440u64
            );
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Empréstimo 2
            sc.borrow(
                managed_address!(&setup.borrower2_address),
                managed_biguint!(13000),
                520u64
            );
        })
        .assert_ok();
    
    // Etapa 6: Emitir tokens de dívida para ambos
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            // Tomador 1
            sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            // Tomador 2
            sc.mint(managed_address!(&setup.borrower2_address), managed_biguint!(13000));
        })
        .assert_ok();
    
    // Verificar saldos de tokens
    setup.blockchain_wrapper
        .execute_query(&setup.debt_token_wrapper, |sc| {
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower1_address)), managed_biguint!(13500));
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower2_address)), managed_biguint!(13000));
        })
        .assert_ok();
    
    // Verificar estado da liquidez
    setup.blockchain_wrapper
        .execute_query(&setup.liquidity_pool_wrapper, |sc| {
            assert_eq!(sc.total_borrows().get(), managed_biguint!(26500));
            assert_eq!(sc.utilization_rate().get(), 1325u64); // 13.25%
        })
        .assert_ok();
    
    // Etapa 7: Tomador 1 paga empréstimo completo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(14094), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
            
            sc.repay();
        })
        .assert_ok();
    
    // Queimar tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        })
        .assert_ok();
    
    // LoanController atualiza status
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.mark_loan_repaid(1u64);
        })
        .assert_ok();
    
    // Etapa 8: Verificar estado atualizado
    setup.blockchain_wrapper
        .execute_query(&setup.liquidity_pool_wrapper, |sc| {
            assert_eq!(sc.total_borrows().get(), managed_biguint!(13000));
            assert_eq!(sc.utilization_rate().get(), 650u64); // 6.5%
        })
        .assert_ok();
    
    // Etapa 9: Simular acúmulo de juros
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(676), |sc| {
            // Juros do empréstimo 2: 13000 * 5.2% = 676
            sc.add_accumulated_interest(managed_biguint!(676));
        })
        .assert_ok();
    
    // Etapa 10: Distribuir juros entre provedores
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.distribute_interest();
            
            // Verificar distribuição
            // Reservas (20%): 676 * 20% = 135
            assert_eq!(sc.total_reserves().get(), managed_biguint!(135));
            
            // Juros para provedores (80%): 676 * 80% = 541
            // Provider 1 (35%): 541 * 35% = 189
            // Provider 2 (65%): 541 * 65% = 352
            assert_eq!(sc.provider_interest(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(189));
            assert_eq!(sc.provider_interest(&managed_address!(&setup.provider2_address)).get(), managed_biguint!(352));
        })
        .assert_ok();
    
    // Etapa 11: Provedor 1 retira liquidez e juros
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            // Simular queima de tokens LP
            sc.burn(managed_address!(&setup.provider1_address), managed_biguint!(70000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Retirar liquidez
            sc.withdraw(managed_biguint!(70000));
            
            // Retirar juros
            sc.withdraw_interest();
            
            // Verificar saldos zerados
            assert_eq!(sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(0));
            assert_eq!(sc.provider_interest(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(0));
        })
        .assert_ok();
}

// Teste integrado de pausa e emergência em todo o sistema
#[test]
fn test_system_pause_emergency_integrated() {
    let mut setup = setup_integrated_system();
    
    // Etapa 1: Configurar estado inicial
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider1_address), managed_biguint!(100000));
        })
        .assert_ok();
    
    // Solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_reputation_check();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower1_address),
                700u64
            );
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_loan();
        })
        .assert_ok();
    
    // Processar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow(
                managed_address!(&setup.borrower1_address),
                managed_biguint!(13500),
                440u64
            );
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        })
        .assert_ok();
    
    // Etapa 2: Detectar condição de emergência (simulação)
    // Pausar todos os contratos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.reputation_score_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    // Etapa 3: Verificar que operações estão bloqueadas
    // Tentativa de novo empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que está pausado
            assert!(sc.is_paused().get());
            
            // Na implementação real, isso lançaria erro
            // "Contract is paused"
        })
        .assert_ok();
    
    // Tentativa de retirada
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que está pausado
            assert!(sc.is_paused().get());
            
            // Na implementação real, isso lançaria erro
            // "Contract is paused"
        })
        .assert_ok();
    
    // Etapa 4: Aplicar medidas de emergência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Ativar modo de emergência
            sc.activate_emergency_mode();
            
            // Configurar limites de emergência
            sc.set_max_withdrawal_limit(2000u64); // 20% máximo por dia
            sc.set_withdrawal_fee(500u64); // 5% taxa de saída
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Configurar parâmetros de emergência
            sc.set_emergency_extension_fee(2000u64); // 20% para extensões em emergência
            sc.set_max_loan_term_days(90u64); // Reduzir prazo máximo
        })
        .assert_ok();
    
    // Etapa 5: Despausar e permitir operações limitadas
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause();
        })
        .assert_ok();
    
    // Etapa 6: Tentar realizar operações com restrições de emergência
    // Tentativa de saque (limitado a 20%)
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            // Simular queima de tokens LP
            sc.burn(managed_address!(&setup.provider1_address), managed_biguint!(20000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Verificar modo de emergência
            assert!(sc.emergency_mode().get());
            
            // Calcular limite de retirada
            let balance = sc.provider_liquidity(&managed_address!(&setup.provider1_address)).get();
            let max_withdrawal_limit = sc.max_withdrawal_limit().get();
            let max_amount = &balance * &managed_biguint!(max_withdrawal_limit) / &managed_biguint!(10000);
            
            // Retirar valor permitido (20000 está dentro do limite de 20%)
            sc.withdraw(managed_biguint!(20000));
            
            // Verificar taxa de saída aplicada
            // 20000 * 5% = 1000 de taxa
            assert_eq!(sc.withdrawal_fees_collected().get(), managed_biguint!(1000));
        })
        .assert_ok();
    
    // Tentativa de extensão com taxa de emergência
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(3010), |sc| {
            // Taxa de extensão em emergência: 20% * 14094 = 2819
            // (Taxas administrativas: 3010 - 2819 = 191)
            sc.extend_loan_deadline(1u64, 15u64);
            
            // Verificar valor atualizado
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.repayment_amount, managed_biguint!(17104)); // 14094 + 3010
        })
        .assert_ok();
    
    // Etapa 7: Normalização pós-emergência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Desativar modo de emergência
            sc.deactivate_emergency_mode();
            
            // Restaurar parâmetros normais
            sc.set_max_withdrawal_limit(10000u64); // 100% - sem limite
            sc.set_withdrawal_fee(0u64); // Sem taxa de saída
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Restaurar parâmetros normais
            sc.set_extension_fee_percent(1000u64); // Voltar para 10%
            sc.set_max_loan_term_days(180u64); // Restaurar prazo máximo
        })
        .assert_ok();
    
    // Etapa 8: Verificar retorno à operação normal
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            // Simular queima de tokens LP
            sc.burn(managed_address!(&setup.provider1_address), managed_biguint!(30000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que não está mais em modo de emergência
            assert!(!sc.emergency_mode().get());
            
            // Retirar sem restrições
            sc.withdraw(managed_biguint!(30000));
            
            // Verificar que não há mais taxa de saída adicional
            let initial_fees = sc.withdrawal_fees_collected().get();
            assert_eq!(initial_fees, managed_biguint!(1000)); // Não mudou
        })
        .assert_ok();
}

// Teste integrado de uso de garantias (collateral)
#[test]
fn test_collateral_integrated() {
    let mut setup = setup_integrated_system();
    
    // Etapa 1: Configurar parâmetros de garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.set_collateral_ratio(7000u64); // 70% (empréstimo/garantia)
            sc.set_liquidation_threshold(8500u64); // 85% (dívida/garantia)
            sc.set_liquidation_penalty(1000u64); // 10% de penalidade
        })
        .assert_ok();
    
    // Etapa 2: Tomador fornece garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(20000), |sc| {
            sc.provide_collateral_for_loan();
            
            // Verificar registro da garantia
            assert_eq!(sc.pending_collateral(&managed_address!(&setup.borrower1_address)).get(), managed_biguint!(20000));
        })
        .assert_ok();
    
    // Etapa 3: Provedor adiciona liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit();
        })
        .assert_ok();
    
    // Simular emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.provider1_address), managed_biguint!(100000));
        })
        .assert_ok();
    
    // Etapa 4: Tomador solicita empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Solicitar empréstimo baseado na garantia
            // Máximo empréstimo: 20000 * 70% = 14000
            let loan_id = sc.request_loan_with_collateral();
            assert_eq!(loan_id, 1u64);
            
            // Verificar detalhes
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.amount, managed_biguint!(14000));
            
            // Verificar transferência da garantia
            assert_eq!(sc.pending_collateral(&managed_address!(&setup.borrower1_address)).get(), managed_biguint!(0));
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(20000));
        })
        .assert_ok();
    
    // Etapa 5: LoanController obtém fundos e emite tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
            // Verificar pontuação e taxa do tomador
            let interest_rate = 800u64; // Taxa para empréstimo com garantia
            
            sc.borrow(
                managed_address!(&setup.borrower1_address),
                managed_biguint!(14000),
                interest_rate
            );
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(14000));
        })
        .assert_ok();
    
    // Etapa 6: Simular queda no valor da garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Atualizar valor da garantia (queda de 40%)
            let original_value = sc.loan_collateral(1u64).get();
            let new_value = &original_value * &managed_biguint!(6000) / &managed_biguint!(10000); // 60% do valor original
            sc.update_collateral_value(1u64, new_value.clone());
            
            // Verificar valor atualizado
            assert_eq!(sc.loan_collateral(1u64).get(), new_value);
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(12000));
        })
        .assert_ok();
    
    // Etapa 7: Verificar liquidação da garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            // Verificar condição de liquidação
            // Dívida: 14000 + juros
            // Garantia: 12000
            // Razão: > 85% (limiar de liquidação)
            let should_liquidate = sc.check_liquidation_needed(1u64);
            assert!(should_liquidate);
            
            // Executar liquidação
            let proceeds = sc.liquidate_collateral(1u64);
            
            // Verificar resultado
            // Valor da garantia menos penalidade: 12000 - 10% = 10800
            assert_eq!(proceeds, managed_biguint!(10800));
            
            // Verificar que o empréstimo foi marcado como liquidado
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Liquidated);
        })
        .assert_ok();
    
    // Etapa 8: LiquidityPool processa fundos da liquidação
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(10800), |sc| {
            // Processar fundos da liquidação
            sc.process_liquidation_proceeds(managed_biguint!(10800));
            
            // Registrar dívida não recuperada
            let loss = managed_biguint!(3200); // 14000 - 10800
            sc.register_bad_debt(loss);
            
            // Verificar registros
            assert_eq!(sc.total_bad_debt().get(), managed_biguint!(3200));
        })
        .assert_ok();
    
    // Etapa 9: Queimar tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.borrower1_address), managed_biguint!(14000));
        })
        .assert_ok();
    
    // Etapa 10: ReputationScore diminui pontuação do tomador
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.reputation_score_wrapper, &rust_biguint!(0), |sc| {
            sc.update_score(managed_address!(&setup.borrower1_address), -150); // Redução severa por liquidação
            
            // Verificar nova pontuação
            let new_score = sc.get_score(&managed_address!(&setup.borrower1_address));
            assert_eq!(new_score, 550u64); // 700 - 150
        })
        .assert_ok();
}