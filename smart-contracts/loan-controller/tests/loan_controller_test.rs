// ==========================================================================
// ARQUIVO: loan_controller_test.rs
// Descrição: Testes unitários básicos para o contrato LoanController
// ==========================================================================

use multiversx_sc_scenario::imports::{ManagedVec, ContractBase};
use multiversx_sc_scenario::api::DebugApi;
use multiversx_sc::types::Address;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
};

use loan_controller::*;

const WASM_PATH: &str = "output/loan-controller.wasm";

// Estrutura para configuração dos testes
struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> loan_controller::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub reputation_score_address: Address, 
    pub borrower_address: Address,
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
    let borrower_address = blockchain_wrapper.create_user_account(&rust_biguint!(1000));
    
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
        reputation_score_address,
        borrower_address,
        contract_wrapper,
    }
}

// Teste de inicialização do contrato
#[test]
fn test_init() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Verificar estado inicial
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(
                sc.reputation_score_address().get(), 
                managed_address!(&setup.reputation_score_address)
            );
            assert_eq!(sc.min_required_score().get(), 500u64);
            assert_eq!(sc.interest_rate_base().get(), 1000u64);
            assert_eq!(sc.base_loan_amount().get(), managed_biguint!(10_000));
            assert_eq!(sc.loan_counter().get(), 0u64);
        })
        .assert_ok();
}

// Teste de cálculo de taxa de juros
#[test]
fn test_calculate_interest_rate() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Pontuação baixa, deve ter taxa próxima da base
            let low_score_rate = sc.calculate_interest_rate(100u64);
            assert_eq!(low_score_rate, 920u64); // 1000 * (1 - (100/1000) * 0.8) = 920
            
            // Pontuação média, deve ter taxa intermediária
            let mid_score_rate = sc.calculate_interest_rate(500u64);
            assert_eq!(mid_score_rate, 600u64); // 1000 * (1 - (500/1000) * 0.8) = 600
            
            // Pontuação alta, deve ter taxa reduzida
            let high_score_rate = sc.calculate_interest_rate(900u64);
            assert_eq!(high_score_rate, 280u64); // 1000 * (1 - (900/1000) * 0.8) = 280
            
            // Pontuação máxima, deve ter taxa mínima
            let max_score_rate = sc.calculate_interest_rate(1000u64);
            assert_eq!(max_score_rate, 200u64); // 1000 / 5 = 200 (taxa mínima)
        })
        .assert_ok();
}

// Teste para verificar os mapeamentos de empréstimos
#[test]
fn test_loan_mappings() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo fictício para testar os mapeamentos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Incrementar o contador de empréstimos
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo fictício
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 12345u64,
                due_timestamp: 23456u64,
                status: LoanStatus::Active,
            };
            
            // Armazenar o empréstimo
            sc.loans(1u64).set(loan);
            
            // Associar ao usuário
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Verificar se o empréstimo foi armazenado corretamente
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            assert_eq!(loan.borrower, managed_address!(&setup.borrower_address));
            assert_eq!(loan.amount, managed_biguint!(5000));
            assert_eq!(loan.repayment_amount, managed_biguint!(5500));
            assert_eq!(loan.interest_rate, 1000u64);
            assert_eq!(loan.creation_timestamp, 12345u64);
            assert_eq!(loan.due_timestamp, 23456u64);
            assert_eq!(loan.status, LoanStatus::Active);
            
            // Verificar associação com o usuário
            let user_loans: Vec<u64> = sc.user_loans(managed_address!(&setup.borrower_address)).iter().collect();
            assert_eq!(user_loans.len(), 1);
            assert_eq!(user_loans[0], 1u64);
        })
        .assert_ok();
}

// ==========================================================================
// Testes adicionais para loan_controller.rs
// ==========================================================================

// Teste para solicitar um empréstimo com pontuação suficiente
#[test]
fn test_request_loan_success() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar uma pontuação de reputação fictícia alta
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                800u64, // Acima do mínimo (500)
            );
        })
        .assert_ok();
    
    // Solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Definir timestamp atual
            sc.get_block_timestamp();
            
            // Solicitar empréstimo
            let loan_id = sc.request_loan(
                managed_biguint!(10_000), // Valor do empréstimo
                LoanTerm::Standard,       // Termo padrão
            );
            
            // Verificar ID e contador
            sc.request_loan(
                managed_biguint!(10_000), // Valor do empréstimo
                LoanTerm::Standard,       // Termo padrão
            );
            assert_eq!(sc.loan_counter().get(), 1u64);
            assert_eq!(sc.loan_counter().get(), 1u64);
            
            // Obter e verificar dados do empréstimo
            let loan = sc.loans(1u64).get();
            
            // Verificar dados básicos
            assert_eq!(loan.borrower, managed_address!(&setup.borrower_address));
            assert_eq!(loan.amount, managed_biguint!(10_000));
            assert_eq!(loan.status, LoanStatus::Active);
            
            // Verificar cálculo de juros (800 score = 360 taxa)
            assert_eq!(loan.interest_rate, 360u64);
            
            // Verificar valor total com juros
            let expected_repayment = managed_biguint!(10_360); // 10000 + (10000 * 360 / 10000)
            assert_eq!(loan.repayment_amount, expected_repayment);
            
            // Verificar timestamps
            assert_eq!(loan.creation_timestamp, 10000u64);
            assert_eq!(loan.due_timestamp, 10000u64 + 30u64 * 24u64 * 60u64 * 60u64);
            
            // Verificar lista de empréstimos do usuário
            let user_loans: Vec<u64> = sc.user_loans(managed_address!(&setup.borrower_address)).iter().collect();
            assert_eq!(user_loans.len(), 1);
            assert_eq!(user_loans.get(0).unwrap(), &1u64);
        })
        .assert_ok();
}

// Teste para solicitar empréstimo com pontuação insuficiente
#[test]
fn test_request_loan_insufficient_score() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Simular resposta da pontuação de reputação baixa
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar uma pontuação de reputação abaixo do mínimo
            let score_value = 400u64; // Abaixo do mínimo (500)
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                score_value,
            );
        })
        .assert_ok();
    
    // Tentar solicitar empréstimo (deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simulate block timestamp by mocking or using a custom implementation
            let current_timestamp = sc.blockchain().get_block_timestamp();
            assert!(current_timestamp >= 10000, "Ensure the block timestamp is valid for the test.");
            
            // Deve lançar erro
            sc.request_loan(managed_biguint!(10_000), LoanTerm::Standard);
        })
        .assert_user_error("Insufficient reputation score");
}

#[test]
fn test_multiple_loans_for_same_user() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Simular resposta da pontuação de reputação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let score_value = 800u64;
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                score_value,
            );
        })
        .assert_ok();
    
    // Solicitar primeiro empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Set timestamp for first loan
            let current_timestamp = sc.blockchain().get_block_timestamp();
            assert!(current_timestamp >= 10000, "Ensure the block timestamp is valid for the test.");
            
            let loan_id = sc.request_loan(managed_biguint!(10_000), LoanTerm::Standard);
            assert_eq!(loan_id, 1u64); // Direct comparison, no unwrap needed
        })
        .assert_ok();
    
    // Solicitar segundo empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            setup.blockchain_wrapper.set_block_timestamp(20000u64);
            
            let loan_id = sc.request_loan(managed_biguint!(10_000), LoanTerm::Standard);
            assert_eq!(loan_id, 2u64); // Should return 2 for second loan
            
            // Verificar associação com o usuário - deve ter 2 empréstimos
            let user_loans: Vec<u64> = sc.user_loans(managed_address!(&setup.borrower_address))
                .iter()
                .collect();
            assert_eq!(user_loans.len(), 2);
            assert_eq!(user_loans[0], 1u64);
            assert_eq!(user_loans[1], 2u64);
        })
        .assert_ok();
}

// Teste para pagamento de empréstimo com sucesso
#[test]
fn test_repay_loan_success() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Incrementar o contador de empréstimos
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo ativo
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500), // 10% de juros
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            // Armazenar o empréstimo
            sc.loans(1u64).set(loan);
            
            // Associar ao usuário
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Efetuar o pagamento do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            sc.blockchain().set_block_timestamp(15000); // Antes do vencimento
            
            sc.repay_loan(1u64);
            
            // Verificar que o empréstimo foi atualizado
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
            
            // Verificar saldo do contrato
            assert_eq!(sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()), managed_biguint!(5500));
        })
        .assert_ok();
}

// Teste para pagamento de empréstimo com valor incorreto
#[test]
fn test_repay_loan_incorrect_amount() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Tentar pagar com valor menor
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5000), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            sc.repay_loan(1u64);
        })
        .assert_user_error("Incorrect repayment amount");
}

// Teste para pagamento de empréstimo depois do vencimento
#[test]
fn test_repay_loan_after_due_date() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Pagar após o vencimento (taxa de atraso deve ser aplicada)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(6050), |sc| {
            // Definir timestamp após o vencimento (5 dias de atraso)
            sc.blockchain().set_block_timestamp(25000);
            
            // Configurar taxa de atraso de 2% ao dia
            sc.late_fee_daily_rate().set(200u64);
            
            sc.repay_loan(1u64);
            
            // Verificar que o empréstimo foi atualizado
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
            
            // Verificar saldo do contrato
            // O valor com atraso deve ser: 5500 + (5500 * 0.02 * 5) = 5500 + 550 = 6050
            assert_eq!(sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()), managed_biguint!(6050));
        })
        .assert_ok();
}

// Teste para tentar pagar empréstimo que não é do usuário
#[test]
fn test_repay_loan_not_borrower() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let other_user = setup.blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Outro usuário tenta pagar o empréstimo
    setup.blockchain_wrapper
        .execute_tx(&other_user, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            sc.repay_loan(1u64);
        })
        .assert_user_error("Only the borrower can repay their loan");
}

// Teste para tentar pagar empréstimo já pago
#[test]
fn test_repay_loan_already_repaid() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo já pago
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Repaid, // Já pago
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Tentar pagar novamente
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            sc.repay_loan(1u64);
        })
        .assert_user_error("Loan is not active");
}

// Teste para configuração de parâmetros apenas pelo proprietário
#[test]
fn test_owner_only_functions() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let non_owner = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Não-proprietário tenta atualizar parâmetros
    setup.blockchain_wrapper
        .execute_tx(&non_owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_min_required_score(600u64);
        })
        .assert_user_error("Only owner can call this function");
    
    // Proprietário atualiza parâmetros com sucesso
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_min_required_score(600u64);
            assert_eq!(sc.min_required_score().get(), 600u64);
            
            sc.set_interest_rate_base(1200u64);
            assert_eq!(sc.interest_rate_base().get(), 1200u64);
            
            sc.set_base_loan_amount(managed_biguint!(20_000));
            assert_eq!(sc.base_loan_amount().get(), managed_biguint!(20_000));
            
            sc.set_late_fee_daily_rate(300u64);
            assert_eq!(sc.late_fee_daily_rate().get(), 300u64);
        })
        .assert_ok();
}

// Teste para atualizar endereço do contrato de pontuação de reputação
#[test]
fn test_update_reputation_score_address() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let new_reputation_address = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Proprietário atualiza endereço
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_reputation_score_address(managed_address!(&new_reputation_address));
            assert_eq!(sc.reputation_score_address().get(), managed_address!(&new_reputation_address));
        })
        .assert_ok();
}

// Teste para calcular valor do empréstimo com base na pontuação
#[test]
fn test_calculate_loan_amount() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Configurar valor base
            sc.base_loan_amount().set(managed_biguint!(10_000));
            
            // Pontuação baixa (50% do valor base)
            let low_score_amount = sc.calculate_loan_amount(500u64);
            assert_eq!(low_score_amount, managed_biguint!(5_000));
            
            // Pontuação média (75% do valor base)
            let mid_score_amount = sc.calculate_loan_amount(750u64);
            assert_eq!(mid_score_amount, managed_biguint!(7_500));
            
            // Pontuação alta (100% do valor base)
            let high_score_amount = sc.calculate_loan_amount(1000u64);
            assert_eq!(high_score_amount, managed_biguint!(10_000));
            
            // Pontuação muito alta (150% do valor base)
            let max_score_amount = sc.calculate_loan_amount(1500u64);
            assert_eq!(max_score_amount, managed_biguint!(15_000));
        })
        .assert_ok();
}

// Teste para simular a chamada de retorno de verificação de reputação
#[test]
fn test_reputation_check_callback() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Simular a chamada de retorno do contrato de pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Armazenar pontuação para um usuário
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                750u64,
            );
            
            // Verificar se foi armazenado
            let score = sc.user_reputation_scores(&managed_address!(&setup.borrower_address)).get();
            assert_eq!(score, 750u64);
        })
        .assert_ok();
    
    // Verificar rejeição de chamadas de outros endereços
    let unauthorized = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    setup.blockchain_wrapper
        .execute_tx(&unauthorized, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                800u64,
            );
        })
        .assert_user_error("Only reputation score contract can call this function");
}

// Teste para obter estatísticas dos empréstimos
#[test]
fn test_loan_statistics() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar múltiplos empréstimos para testar estatísticas
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(3u64);
            
            // Empréstimo 1: Ativo
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            sc.loans(1u64).set(loan1);
            
            // Empréstimo 2: Pago
            let loan2 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7000),
                repayment_amount: managed_biguint!(7700),
                interest_rate: 1000u64,
                creation_timestamp: 5000u64,
                due_timestamp: 15000u64,
                status: LoanStatus::Repaid,
            };
            sc.loans(2u64).set(loan2);
            
            // Empréstimo 3: Em atraso
            let loan3 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(3000),
                repayment_amount: managed_biguint!(3300),
                interest_rate: 1000u64,
                creation_timestamp: 1000u64,
                due_timestamp: 11000u64,
                status: LoanStatus::Active, // Ainda ativo mas já venceu
            };
            sc.loans(3u64).set(loan3);
            
            // Associar empréstimos ao usuário
            let mut user_loans = ManagedVec::new();
            user_loans.push(1u64);
            user_loans.push(2u64);
            user_loans.push(3u64);
            sc.user_loans(managed_address!(&setup.borrower_address)).set(user_loans, &sc.blockchain());
        })
        .assert_ok();
    
    // Set timestamp before executing query
    setup.blockchain_wrapper.set_block_timestamp(15001u64);

    // Verificar estatísticas
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {    
            // Total de empréstimos
            assert_eq!(sc.loan_counter().get(), 3u64);
            
            // Empréstimos ativos
            let active_loans = sc.get_active_loans_count();
            assert_eq!(active_loans, 2u64); // Empréstimos 1 e 3
            
            // Empréstimos pagos
            let repaid_loans = sc.get_repaid_loans_count();
            assert_eq!(repaid_loans, 1u64); // Empréstimo 2
            
            // Empréstimos em atraso
            let overdue_loans = sc.get_overdue_loans_count();
            assert_eq!(overdue_loans, 1u64); // Empréstimo 3
            
            // Valor total emprestado
            let total_loaned = sc.get_total_loan_amount();
            assert_eq!(total_loaned, managed_biguint!(15000)); // 5000 + 7000 + 3000
            
            // Valor total a ser repago
            let total_to_repay = sc.get_total_repayment_amount();
            assert_eq!(total_to_repay, managed_biguint!(16500)); // 5500 + 7700 + 3300
        })
        .assert_ok();
}

// Teste para saque de fundos pelo proprietário
#[test]
fn test_withdraw_funds() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Adicionar fundos ao contrato
    setup.blockchain_wrapper.add_egld_to_account(
        &setup.contract_wrapper.address_ref(),
        &rust_biguint!(10000),
    );
    
    // Proprietário saca fundos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let initial_owner_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.owner_address));
            
            sc.withdraw_funds();
            
            // Verificar que o saldo do contrato foi para zero
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(0)
            );
            
            // Verificar que o proprietário recebeu os fundos
            let final_owner_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.owner_address));
            assert_eq!(final_owner_balance, initial_owner_balance + managed_biguint!(10000));
        })
        .assert_ok();
    
    // Não-proprietário tenta sacar
    let non_owner = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    setup.blockchain_wrapper
        .execute_tx(&non_owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.withdraw_funds();
        })
        .assert_user_error("Only owner can call this function");
}

// Teste para pausar/despausar o contrato
#[test]
fn test_pause_unpause_contract() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Proprietário pausa o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause_contract();
            assert!(sc.is_paused().get());
        })
        .assert_ok();
    
    // Tentar solicitar empréstimo com contrato pausado
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.request_loan();
        })
        .assert_user_error("Contract is paused");
    
    // Proprietário despausa o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause_contract();
            assert!(!sc.is_paused().get());
        })
        .assert_ok();
    
    // Agora deve ser possível solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                800u64,
            );
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.request_loan();
        })
        .assert_ok();
}

// Teste para verificar o histórico de empréstimos de um usuário (continuação)
#[test]
fn test_user_loan_history() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar múltiplos empréstimos para um usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(3u64);
            
            // Empréstimo 1: Pago
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 1000u64,
                due_timestamp: 11000u64,
                status: LoanStatus::Repaid,
            };
            sc.loans(1u64).set(loan1);
            
            // Empréstimo 2: Pago
            let loan2 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7000),
                repayment_amount: managed_biguint!(7700),
                interest_rate: 1000u64,
                creation_timestamp: 5000u64,
                due_timestamp: 15000u64,
                status: LoanStatus::Repaid,
            };
            sc.loans(2u64).set(loan2);
            
            // Empréstimo 3: Ativo
            let loan3 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(3000),
                repayment_amount: managed_biguint!(3300),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            sc.loans(3u64).set(loan3);
            
            // Associar empréstimos ao usuário
            let mut user_loans = ManagedVec::new();
            user_loans.push(1u64);
            user_loans.push(2u64);
            user_loans.push(3u64);
            sc.user_loans(managed_address!(&setup.borrower_address)).set(user_loans);
        })
        .assert_ok();
    
    // Verificar histórico
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Obter empréstimos do usuário
            let user_loans = sc.get_user_loans(&managed_address!(&setup.borrower_address));
            assert_eq!(user_loans.len(), 3);
            
            // Verificar detalhes de empréstimos individuais
            let loan_details = sc.get_loan_details(1u64);
            assert_eq!(loan_details.status, LoanStatus::Repaid);
            
            // Verificar histórico completo
            let loan_history = sc.get_user_loan_history(&managed_address!(&setup.borrower_address));
            assert_eq!(loan_history.len(), 3);
            
            // Verificar empréstimos ativos do usuário
            let active_loans = sc.get_user_active_loans(&managed_address!(&setup.borrower_address));
            assert_eq!(active_loans.len(), 1);
            assert_eq!(active_loans[0], 3u64);
            
            // Verificar empréstimos pagos do usuário
            let repaid_loans = sc.get_user_repaid_loans(&managed_address!(&setup.borrower_address));
            assert_eq!(repaid_loans.len(), 2);
            assert!(repaid_loans.contains(&1u64));
            assert!(repaid_loans.contains(&2u64));
        })
        .assert_ok();
}

// Teste para lidar com empréstimos expirados
#[test]
fn test_handle_expired_loans() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar empréstimos vencidos e não vencidos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(3u64);
            
            // Empréstimo 1: Ativo, vencido
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 1000u64,
                due_timestamp: 11000u64, // Vencido
                status: LoanStatus::Active,
            };
            sc.loans(1u64).set(loan1);
            
            // Empréstimo 2: Ativo, não vencido
            let loan2 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7000),
                repayment_amount: managed_biguint!(7700),
                interest_rate: 1000u64,
                creation_timestamp: 5000u64,
                due_timestamp: 25000u64, // Não vencido
                status: LoanStatus::Active,
            };
            sc.loans(2u64).set(loan2);
            
            // Empréstimo 3: Já pago, vencido
            let loan3 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(3000),
                repayment_amount: managed_biguint!(3300),
                interest_rate: 1000u64,
                creation_timestamp: 1000u64,
                due_timestamp: 11000u64, // Vencido, mas já pago
                status: LoanStatus::Repaid,
            };
            sc.loans(3u64).set(loan3);
            
            // Associar empréstimos ao usuário
            let mut user_loans = ManagedVec::new();
            user_loans.push(1u64);
            user_loans.push(2u64);
            user_loans.push(3u64);
            sc.user_loans(managed_address!(&setup.borrower_address)).set(user_loans);
        })
        .assert_ok();
    
    // Marcar empréstimos vencidos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(15000); // Após vencimento do empréstimo 1
            
            // Marcar empréstimos vencidos
            sc.mark_expired_loans();
            
            // Verificar status do empréstimo 1 (deve estar como Defaulted)
            let loan1 = sc.loans(1u64).get();
            assert_eq!(loan1.status, LoanStatus::Defaulted);
            
            // Empréstimo 2 deve continuar Ativo
            let loan2 = sc.loans(2u64).get();
            assert_eq!(loan2.status, LoanStatus::Active);
            
            // Empréstimo 3 deve continuar como Repaid
            let loan3 = sc.loans(3u64).get();
            assert_eq!(loan3.status, LoanStatus::Repaid);
        })
        .assert_ok();
}

// Teste para verificar a lógica de extensão de prazo de empréstimo
#[test]
fn test_extend_loan_deadline() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo ativo
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Taxa de extensão de prazo (10%)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.extension_fee_percent().set(1000u64); // 10%
        })
        .assert_ok();
    
    // Solicitar extensão de prazo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(550), |sc| {
            sc.blockchain().set_block_timestamp(18000); // Antes do vencimento
            
            // Extender prazo em 15 dias
            sc.extend_loan_deadline(1u64, 15u64);
            
            // Verificar novo prazo e valor de repagamento
            let loan = sc.loans(1u64).get();
            
            // Novo prazo: 20000 + (15 * 24 * 60 * 60) = 20000 + 1296000 = 1316000
            assert_eq!(loan.due_timestamp, 20000u64 + 15u64 * 24u64 * 60u64 * 60u64);
            
            // Novo valor de repagamento: 5500 + (5500 * 10% = 550) = 6050
            assert_eq!(loan.repayment_amount, managed_biguint!(6050));
        })
        .assert_ok();
}

// Teste para tentar extender prazo sem pagar a taxa correta
#[test]
fn test_extend_loan_deadline_incorrect_fee() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Taxa de extensão de prazo (10%)
            sc.extension_fee_percent().set(1000u64);
        })
        .assert_ok();
    
    // Solicitar extensão com valor incorreto
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(500), |sc| {
            sc.blockchain().set_block_timestamp(18000);
            
            // Deve falhar pois a taxa correta é 550 (10% de 5500)
            sc.extend_loan_deadline(1u64, 15u64);
        })
        .assert_user_error("Incorrect extension fee amount");
}

// Teste para tentar extender prazo de um empréstimo que não é do usuário
#[test]
fn test_extend_loan_deadline_not_borrower() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let other_user = setup.blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Taxa de extensão de prazo (10%)
            sc.extension_fee_percent().set(1000u64);
        })
        .assert_ok();
    
    // Outro usuário tenta extender o prazo
    setup.blockchain_wrapper
        .execute_tx(&other_user, &setup.contract_wrapper, &rust_biguint!(550), |sc| {
            sc.blockchain().set_block_timestamp(18000);
            
            sc.extend_loan_deadline(1u64, 15u64);
        })
        .assert_user_error("Only the borrower can extend their loan");
}

// Teste para tentar extender o prazo de um empréstimo já vencido
#[test]
fn test_extend_loan_deadline_already_expired() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Taxa de extensão de prazo (10%)
            sc.extension_fee_percent().set(1000u64);
        })
        .assert_ok();
    
    // Tentar extender após o vencimento
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(550), |sc| {
            sc.blockchain().set_block_timestamp(21000); // Após o vencimento
            
            sc.extend_loan_deadline(1u64, 15u64);
        })
        .assert_user_error("Cannot extend an expired loan");
}

// Teste para verificar a lógica de perda de garantia (collateral forfeiture)
#[test]
fn test_collateral_forfeiture() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo ativo com garantia
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Adicionar garantia
            sc.loan_collateral(1u64).set(managed_biguint!(7000));
        })
        .assert_ok();
    
    // Marcar como inadimplente e executar a garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(25000); // Após o vencimento
            
            // Marcar como inadimplente
            sc.mark_loan_defaulted(1u64);
            
            // Verificar status
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Defaulted);
            
            // Executar a garantia
            sc.forfeit_collateral(1u64);
            
            // Verificar que a garantia foi zerada
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(0));
            
            // Verificar que a garantia foi transferida para o contrato
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(7000)
            );
        })
        .assert_ok();
}

// Teste para tentativa não autorizada de executar a garantia
#[test]
fn test_unauthorized_collateral_forfeiture() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let non_owner = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Configurar um empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Defaulted, // Já marcado como inadimplente
            };
            
            sc.loans(1u64).set(loan);
            sc.loan_collateral(1u64).set(managed_biguint!(7000));
        })
        .assert_ok();
    
    // Usuário não autorizado tenta executar a garantia
    setup.blockchain_wrapper
        .execute_tx(&non_owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.forfeit_collateral(1u64);
        })
        .assert_user_error("Only owner can call this function");
}

// Teste para fornecer garantia para um empréstimo
#[test]
fn test_provide_collateral() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Fornecer garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(6000), |sc| {
            sc.provide_collateral(1u64);
            
            // Verificar que a garantia foi registrada
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(6000));
            
            // Verificar saldo do contrato
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(6000)
            );
        })
        .assert_ok();
}

// Teste para retirar garantia após pagamento do empréstimo
#[test]
fn test_withdraw_collateral_after_repayment() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Adicionar garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(6000), |sc| {
            sc.provide_collateral(1u64);
        })
        .assert_ok();
    
    // Pagar o empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            sc.repay_loan(1u64);
            
            // Verificar que o empréstimo foi pago
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
        })
        .assert_ok();
    
    // Retirar garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let initial_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.borrower_address));
            
            sc.withdraw_collateral(1u64);
            
            // Verificar que a garantia foi zerada
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(0));
            
            // Verificar que o valor foi transferido ao usuário
            let final_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.borrower_address));
            assert_eq!(final_balance, initial_balance + managed_biguint!(6000));
        })
        .assert_ok();
}

// Teste para tentar retirar garantia com empréstimo ainda ativo
#[test]
fn test_withdraw_collateral_with_active_loan() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active, // Ainda ativo
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            sc.loan_collateral(1u64).set(managed_biguint!(6000));
        })
        .assert_ok();
    
    // Tentar retirar garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.withdraw_collateral(1u64);
        })
        .assert_user_error("Cannot withdraw collateral for active or defaulted loans");
}

// Teste para verificar a validação da taxa de juros mínima e máxima
#[test]
fn test_interest_rate_limits() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar limites de taxa de juros
            sc.min_interest_rate().set(200u64); // 2%
            sc.max_interest_rate().set(3000u64); // 30%
            
            // Verificar cálculo com limites
            
            // Taxa com score muito alto (abaixo do mínimo)
            let rate_high_score = sc.calculate_interest_rate_with_limits(1000u64);
            assert_eq!(rate_high_score, 200u64); // Taxa mínima
            
            // Taxa com score na média
            let rate_mid_score = sc.calculate_interest_rate_with_limits(500u64);
            assert_eq!(rate_mid_score, 600u64); // Dentro dos limites
            
            // Taxa com score muito baixo (acima do máximo)
            let rate_low_score = sc.calculate_interest_rate_with_limits(100u64);
            assert_eq!(rate_low_score, 3000u64); // Taxa máxima
        })
        .assert_ok();
}

// Teste para atualizar configurações de prazo de empréstimo
#[test]
fn test_loan_term_settings() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Atualizar configurações de prazo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Definir prazos padrão para diferentes tipos de empréstimo
            sc.standard_loan_term_days().set(30u64); // 30 dias
            sc.extended_loan_term_days().set(90u64); // 90 dias
            sc.max_loan_term_days().set(180u64); // 180 dias
            
            // Verificar configurações
            assert_eq!(sc.standard_loan_term_days().get(), 30u64);
            assert_eq!(sc.extended_loan_term_days().get(), 90u64);
            assert_eq!(sc.max_loan_term_days().get(), 180u64);
        })
        .assert_ok();
    
    // Verificar aplicação de prazos diferentes em novos empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular timestamp atual
            sc.blockchain().set_block_timestamp(10000);
            
            // Calcular prazo padrão
            let standard_due_date = sc.calculate_due_date(LoanTerm::Standard);
            assert_eq!(standard_due_date, 10000u64 + 30u64 * 24u64 * 60u64 * 60u64);
            
            // Calcular prazo estendido
            let extended_due_date = sc.calculate_due_date(LoanTerm::Extended);
            assert_eq!(extended_due_date, 10000u64 + 90u64 * 24u64 * 60u64 * 60u64);
            
            // Calcular prazo máximo
            let max_due_date = sc.calculate_due_date(LoanTerm::Maximum);
            assert_eq!(max_due_date, 10000u64 + 180u64 * 24u64 * 60u64 * 60u64);
        })
        .assert_ok();
}

// Teste para empréstimos com diferentes prazos (continuação)
#[test]
fn test_loans_with_different_terms() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar prazos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.standard_loan_term_days().set(30u64);
            sc.extended_loan_term_days().set(90u64);
            sc.max_loan_term_days().set(180u64);
        })
        .assert_ok();
    
    // Simular resposta da pontuação de reputação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let score_value = 800u64;
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                score_value,
            );
        })
        .assert_ok();
    
    // Solicitar empréstimo com prazo estendido
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(10000);
            
            // Solicitar empréstimo com prazo estendido
            let loan_id = sc.request_loan_with_term(LoanTerm::Extended);
            assert_eq!(loan_id.unwrap(), 1u64); // Ensure the function returns a Result<u64, _> or similar
            
            // Verificar dados do empréstimo
            let loan = sc.loans(1u64).get();
            
            // Verificar que o prazo foi configurado corretamente (90 dias)
            assert_eq!(loan.due_timestamp, 10000u64 + 90u64 * 24u64 * 60u64 * 60u64);
            
            // Taxa de juros deve ser mais alta para prazos mais longos
            // Assumindo fator de ajuste de 1.5x para prazo estendido
            let base_rate = sc.calculate_interest_rate(800u64);
            assert_eq!(loan.interest_rate, (base_rate as f64 * 1.5) as u64);
        })
        .assert_ok();
    
    // Solicitar outro empréstimo com prazo máximo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(20000);
            
            // Solicitar empréstimo com prazo máximo
            let loan_id = sc.request_loan_with_term(LoanTerm::Maximum);
            assert_eq!(loan_id, 2u64);
            
            // Verificar dados do empréstimo
            let loan = sc.loans(2u64).get();
            
            // Verificar que o prazo foi configurado corretamente (180 dias)
            assert_eq!(loan.due_timestamp, 20000u64 + 180u64 * 24u64 * 60u64 * 60u64);
            
            // Taxa de juros deve ser mais alta para prazos mais longos
            // Assumindo fator de ajuste de 2x para prazo máximo
            let base_rate = sc.calculate_interest_rate(800u64);
            assert_eq!(loan.interest_rate, (base_rate as f64 * 2.0) as u64);
        })
        .assert_ok();
}

// Teste para verificar a contabilidade correta de juros
#[test]
fn test_interest_accounting() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10_000),
                repayment_amount: managed_biguint!(11_000), // 10% de juros = 1000
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Pagar o empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(11_000), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            // Antes do pagamento, verificar o total de juros acumulados
            assert_eq!(sc.total_interest_earned().get(), managed_biguint!(0));
            
            sc.repay_loan(1u64);
            
            // Após o pagamento, verificar contabilidade de juros
            let interest_earned = managed_biguint!(1_000); // 11000 - 10000
            assert_eq!(sc.total_interest_earned().get(), interest_earned);
            
            // Verificar o saldo total do contrato (principal + juros)
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(11_000)
            );
        })
        .assert_ok();
}

// Teste para verificar a contabilidade de vários empréstimos
#[test]
fn test_multiple_loans_accounting() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let borrower2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    
    // Configurar dois empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(2u64);
            
            // Empréstimo 1
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10_000),
                repayment_amount: managed_biguint!(11_000), // 10% de juros
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            sc.loans(1u64).set(loan1);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Empréstimo 2
            let loan2 = Loan {
                borrower: managed_address!(&borrower2),
                amount: managed_biguint!(5_000),
                repayment_amount: managed_biguint!(5_500), // 10% de juros
                interest_rate: 1000u64,
                creation_timestamp: 11000u64,
                due_timestamp: 21000u64,
                status: LoanStatus::Active,
            };
            sc.loans(2u64).set(loan2);
            sc.user_loans(managed_address!(&borrower2)).push(&2u64);
        })
        .assert_ok();
    
    // Pagar o primeiro empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(11_000), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            sc.repay_loan(1u64);
            
            // Verificar juros após primeiro pagamento
            assert_eq!(sc.total_interest_earned().get(), managed_biguint!(1_000));
            
            // Verificar saldo
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(11_000)
            );
        })
        .assert_ok();
    
    // Pagar o segundo empréstimo
    setup.blockchain_wrapper
        .execute_tx(&borrower2, &setup.contract_wrapper, &rust_biguint!(5_500), |sc| {
            sc.blockchain().set_block_timestamp(16000);
            sc.repay_loan(2u64);
            
            // Verificar juros totais (1000 + 500 = 1500)
            assert_eq!(sc.total_interest_earned().get(), managed_biguint!(1_500));
            
            // Verificar saldo total (11000 + 5500 = 16500)
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(16_500)
            );
        })
        .assert_ok();
}

// Teste para verificar a distribuição de lucros
#[test]
fn test_distribute_profits() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let investor1 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let investor2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Configurar e pagar um empréstimo para gerar juros
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar empréstimo
            sc.loan_counter().set(1u64);
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10_000),
                repayment_amount: managed_biguint!(11_000), // 1000 de juros
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Adicionar investidores
            sc.add_investor(managed_address!(&investor1), 6000u64); // 60%
            sc.add_investor(managed_address!(&investor2), 4000u64); // 40%
        })
        .assert_ok();
    
    // Pagar o empréstimo para gerar juros
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(11_000), |sc| {
            sc.repay_loan(1u64);
        })
        .assert_ok();
    
    // Distribuir lucros dos juros
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar saldo antes da distribuição
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(11_000)
            );
            
            // Distribuir apenas os juros (1000)
            sc.distribute_profits();
            
            // Verificar saldo após distribuição (deve manter o principal)
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(10_000) // Principal mantido
            );
            
            // Verificar que os investidores receberam suas partes dos juros
            assert_eq!(
                sc.blockchain().get_egld_balance(&managed_address!(&investor1)),
                managed_biguint!(600) // 60% de 1000
            );
            assert_eq!(
                sc.blockchain().get_egld_balance(&managed_address!(&investor2)),
                managed_biguint!(400) // 40% de 1000
            );
            
            // Verificar que o contador de juros foi zerado
            assert_eq!(sc.total_interest_earned().get(), managed_biguint!(0));
        })
        .assert_ok();
}

// Teste para verificar a validação de endereços ao adicionar investidores
#[test]
fn test_investor_address_validation() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Tentar adicionar o endereço zero como investidor
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let zero_address = managed_address!(&Address::zero());
            sc.add_investor(zero_address, 5000u64);
        })
        .assert_user_error("Invalid investor address");
}

// Teste para validar a soma das participações dos investidores
#[test]
fn test_investor_shares_validation() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let investor1 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let investor2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Adicionar investidor com 80% de participação
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.add_investor(managed_address!(&investor1), 8000u64); // 80%
        })
        .assert_ok();
    
    // Tentar adicionar outro investidor com 30% (total seria 110%)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.add_investor(managed_address!(&investor2), 3000u64); // 30%
        })
        .assert_user_error("Total investor shares cannot exceed 100%");
}

// Teste para remover investidor
#[test]
fn test_remove_investor() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let investor1 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let investor2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Adicionar investidores
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.add_investor(managed_address!(&investor1), 6000u64); // 60%
            sc.add_investor(managed_address!(&investor2), 4000u64); // 40%
            
            // Verificar total de participações
            assert_eq!(sc.total_investor_shares().get(), 10000u64); // 100%
        })
        .assert_ok();
    
    // Remover um investidor
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.remove_investor(managed_address!(&investor1));
            
            // Verificar que o investidor foi removido
            assert_eq!(sc.investor_shares(&managed_address!(&investor1)).get(), 0u64);
            
            // Verificar total de participações
            assert_eq!(sc.total_investor_shares().get(), 4000u64); // 40%
        })
        .assert_ok();
}

// Teste para verificar limite de empréstimos por usuário
#[test]
fn test_user_loan_limit() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar limite de empréstimos por usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.max_loans_per_user().set(2u64);
        })
        .assert_ok();
    
    // Simular resposta da pontuação de reputação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let score_value = 800u64;
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                score_value,
            );
        })
        .assert_ok();
    
    // Configurar dois empréstimos existentes
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(2u64);
            
            // Empréstimo 1
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            sc.loans(1u64).set(loan1);
            
            // Empréstimo 2
            let loan2 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(3000),
                repayment_amount: managed_biguint!(3300),
                interest_rate: 1000u64,
                creation_timestamp: 11000u64,
                due_timestamp: 21000u64,
                status: LoanStatus::Active,
            };
            sc.loans(2u64).set(loan2);
            
            // Associar ambos os empréstimos ao usuário
            let mut user_loans = ManagedVec::new();
            user_loans.push(1u64);
            user_loans.push(2u64);
            sc.user_loans(managed_address!(&setup.borrower_address)).set(user_loans);
        })
        .assert_ok();
    
    // Tentar solicitar um terceiro empréstimo (deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(12000);
            sc.request_loan();
        })
        .assert_user_error("User has reached maximum allowed loans");
}

// Teste para verificar limite de valor de empréstimo baseado em garantia
#[test]
fn test_loan_amount_based_on_collateral() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar razão de garantia (valor do empréstimo não pode exceder 70% da garantia)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.collateral_ratio().set(7000u64); // 70%
        })
        .assert_ok();
    
    // Fornecer garantia antes de solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(10_000), |sc| {
            sc.provide_collateral_for_new_loan();
            
            // Verificar que a garantia foi registrada
            assert_eq!(sc.pending_collateral(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(10_000));
        })
        .assert_ok();
    
    // Simular resposta da pontuação de reputação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let score_value = 800u64;
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                score_value,
            );
        })
        .assert_ok();
    
    // Solicitar empréstimo baseado na garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(10000);
            
            // Solicitar empréstimo
            let loan_id = sc.request_loan_with_collateral();
            assert_eq!(loan_id, 1u64);
            
            // Verificar dados do empréstimo (valor deve ser 70% da garantia = 7000)
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.amount, managed_biguint!(7_000));
            
            // Verificar que a garantia pendente foi movida para a garantia do empréstimo
            assert_eq!(sc.pending_collateral(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(0));
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(10_000));
        })
        .assert_ok();
}

// Teste para tentativa de solicitar empréstimo com garantia insuficiente
#[test]
fn test_loan_with_insufficient_collateral() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar valor mínimo de garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.min_collateral_amount().set(managed_biguint!(5_000));
        })
        .assert_ok();
    
    // Fornecer garantia insuficiente
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(4_000), |sc| {
            sc.provide_collateral_for_new_loan();
        })
        .assert_ok();
    
    // Simular resposta da pontuação de reputação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let score_value = 800u64;
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                score_value,
            );
        })
        .assert_ok();
    
    // Tentar solicitar empréstimo com garantia insuficiente
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.request_loan_with_collateral();
        })
        .assert_user_error("Insufficient collateral provided");
}

// Teste para verificar cancelamento de solicitação de empréstimo e devolução de garantia
#[test]
fn test_cancel_loan_request_and_return_collateral() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Fornecer garantia para empréstimo futuro
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(10_000), |sc| {
            sc.provide_collateral_for_new_loan();
            
            // Verificar que a garantia foi registrada
            assert_eq!(sc.pending_collateral(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(10_000));
        })
        .assert_ok();
    
    // Cancelar solicitação e devolver garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let initial_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.borrower_address));
            
            sc.cancel_loan_request();
            
            // Verificar que a garantia pendente foi zerada
            assert_eq!(sc.pending_collateral(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(0));
            
            // Verificar que a garantia foi devolvida ao usuário
            let final_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.borrower_address));
            assert_eq!(final_balance, initial_balance + managed_biguint!(10_000));
        })
        .assert_ok();
}

// Teste para liquidação de garantia por valor menor em leilão
#[test]
fn test_liquidate_collateral_via_auction() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let winning_bidder = setup.blockchain_wrapper.create_user_account(&rust_biguint!(8_000));
    
    // Configurar um empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo vencido
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7_000),
                repayment_amount: managed_biguint!(7_700),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Defaulted, // Já marcado como inadimplente
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Adicionar garantia (valor maior que o empréstimo)
            sc.loan_collateral(1u64).set(managed_biguint!(10_000));
        })
        .assert_ok();
    
    // Simular leilão com lance vencedor
    setup.blockchain_wrapper
        .execute_tx(&winning_bidder, &setup.contract_wrapper, &rust_biguint!(8_000), |sc| {
            sc.blockchain().set_block_timestamp(25000); // Após o vencimento
            
            // Executar liquidação via leilão
            sc.liquidate_collateral_via_auction(1u64);
            
            // Verificar que a garantia foi transferida ao licitante vencedor
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(0));
            
            // Verificar que o valor do lance foi recebido pelo contrato
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(8_000)
            );
            
            // Verificar que o empréstimo foi marcado como liquidado
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Liquidated);
        })
        .assert_ok();
}

// Teste para pagamento parcial de empréstimo
#[test]
fn test_partial_loan_repayment() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo ativo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10_000),
                repayment_amount: managed_biguint!(11_000),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Habilitar pagamentos parciais
            sc.allow_partial_repayments().set(true);
        })
        .assert_ok();
    
    // Fazer pagamento parcial
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5_000), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            // Pagar parte do empréstimo
            sc.partial_repay_loan(1u64);
            
            // Verificar que o valor de repagamento foi reduzido
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.repayment_amount, managed_biguint!(6_000)); // 11000 - 5000 = 6000
            
            // Verificar que o empréstimo continua ativo
            assert_eq!(loan.status, LoanStatus::Active);
            
            // Verificar pagamentos registrados
            assert_eq!(sc.loan_payments(1u64).get(), managed_biguint!(5_000));
        })
        .assert_ok();
    
    // Pagar o restante
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(6_000), |sc| {
            sc.blockchain().set_block_timestamp(16000);
            
            // Pagar o restante
            sc.partial_repay_loan(1u64);
            
            // Verificar que o empréstimo foi marcado como pago
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
            assert_eq!(loan.repayment_amount, managed_biguint!(0));
            
            // Verificar pagamentos totais
            assert_eq!(sc.loan_payments(1u64).get(), managed_biguint!(11_000));
        })
        .assert_ok();
}

// Teste para verificar a lógica de tempo de carência
#[test]
fn test_grace_period() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar período de carência de 5 dias
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.grace_period_days().set(5u64);
        })
        .assert_ok();
    
    // Configurar um empréstimo vencido mas dentro do período de carência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10_000),
                repayment_amount: managed_biguint!(11_000),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64, // Vencido na data 20000
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Verificar que o empréstimo não é marcado como inadimplente durante o período de carência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // 3 dias depois do vencimento (dentro do período de carência)
            sc.blockchain().set_block_timestamp(20000u64 + 3u64 * 24u64 * 60u64 * 60u64);
            
            // Tentar marcar empréstimos vencidos
            sc.mark_expired_loans();
            
            // Verificar que o empréstimo ainda está ativo
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Active);
        })
        .assert_ok();
    
    // Verificar que o empréstimo é marcado como inadimplente após o período de carência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // 6 dias depois do vencimento (após o período de carência)
            sc.blockchain().set_block_timestamp(20000u64 + 6u64 * 24u64 * 60u64 * 60u64);
            
            // Marcar empréstimos vencidos
            sc.mark_expired_loans();
            
            // Verificar que o empréstimo foi marcado como inadimplente
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Defaulted);
        })
        .assert_ok();
}

// Teste para verificar tentativa de pagamento durante o período de carência
#[test]
fn test_repayment_during_grace_period() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar período de carência e taxa de atraso
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.grace_period_days().set(5u64);
            sc.late_fee_daily_rate().set(200u64); // 2% ao dia
        })
        .assert_ok();
    
    // Configurar um empréstimo vencido
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10_000),
                repayment_amount: managed_biguint!(11_000),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Pagar o empréstimo durante o período de carência (taxa de atraso deve ser aplicada)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(11_660), |sc| {
            // 3 dias depois do vencimento (dentro do período de carência)
            sc.blockchain().set_block_timestamp(20000u64 + 3u64 * 24u64 * 60u64 * 60u64);
            
            // Pagar o empréstimo
            sc.repay_loan(1u64);
            
            // Verificar que o empréstimo foi pago
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
            
            // Verificar que a taxa de atraso foi aplicada corretamente
            // 3 dias de atraso a 2% ao dia: 11000 + (11000 * 0.02 * 3) = 11000 + 660 = 11660
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(11_660)
            );
        })
        .assert_ok();
}

// Teste para verificar a aplicação de taxas de atraso progressivas
#[test]
fn test_progressive_late_fees() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar taxas de atraso progressivas
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Taxa de atraso inicial - 1% ao dia
            sc.late_fee_daily_rate().set(100u64);
            
            // Taxa de atraso após 10 dias - 2% ao dia
            sc.progressive_late_fee_threshold_days().set(10u64);
            sc.progressive_late_fee_daily_rate().set(200u64);
        })
        .assert_ok();
    
    // Configurar um empréstimo vencido
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10_000),
                repayment_amount: managed_biguint!(11_000),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Pagar o empréstimo após o limite progressivo (15 dias de atraso)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(12_650), |sc| {
            // 15 dias depois do vencimento
            sc.blockchain().set_block_timestamp(20000u64 + 15u64 * 24u64 * 60u64 * 60u64);
            
            // Pagar o empréstimo
            sc.repay_loan(1u64);
            
            // Verificar que o empréstimo foi pago
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
            
            // Verificar que a taxa de atraso progressiva foi aplicada corretamente
            // Primeiros 10 dias a 1% ao dia: 11000 * 0.01 * 10 = 1100
            // Próximos 5 dias a 2% ao dia: 11000 * 0.02 * 5 = 1100
            // Total de juros de atraso: 1100 + 550 = 1650
            // Valor total: 11000 + 1650 = 12650
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(12_650)
            );
        })
        .assert_ok();
}

// Teste para verificar o limite de empréstimos global
#[test]
fn test_global_loan_limit() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let borrower2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(1000));
    
    // Configurar limite global de empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Limite de 2 empréstimos ativos
            sc.max_active_loans().set(2u64);
        })
        .assert_ok();
    
    // Simular resposta da pontuação de reputação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar scores para ambos os usuários
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                800u64,
            );
            sc.reputation_check_callback(
                managed_address!(&borrower2),
                800u64,
            );
        })
        .assert_ok();
    
    // Configurar dois empréstimos existentes
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(2u64);
            
            // Empréstimo 1
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            sc.loans(1u64).set(loan1);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Empréstimo 2
            let loan2 = Loan {
                borrower: managed_address!(&borrower2),
                amount: managed_biguint!(3000),
                repayment_amount: managed_biguint!(3300),
                interest_rate: 1000u64,
                creation_timestamp: 11000u64,
                due_timestamp: 21000u64,
                status: LoanStatus::Active,
            };
            sc.loans(2u64).set(loan2);
            sc.user_loans(managed_address!(&borrower2)).push(&2u64);
            
            // Atualizar contador de empréstimos ativos
            sc.active_loans_count().set(2u64);
        })
        .assert_ok();
    
    // Tentar solicitar um terceiro empréstimo (deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(12000);
            sc.request_loan();
        })
        .assert_user_error("Global loan limit reached");
}

// Teste para verificar o cálculo do valor de liquidação da garantia com desconto
#[test]
fn test_liquidation_discount() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar desconto de liquidação (20%)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.liquidation_discount().set(2000u64); // 20%
        })
        .assert_ok();
    
    // Configurar um empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7_000),
                repayment_amount: managed_biguint!(7_700),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Defaulted,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Adicionar garantia (valor maior que o empréstimo)
            sc.loan_collateral(1u64).set(managed_biguint!(10_000));
            
            // Calcular valor de liquidação
            let liquidation_value = sc.calculate_liquidation_value(1u64);
            
            // Valor com desconto: 10000 * (1 - 0.2) = 10000 * 0.8 = 8000
            assert_eq!(liquidation_value, managed_biguint!(8_000));
        })
        .assert_ok();
}

// Teste para verificar recuperação de emergência de fundos
#[test]
fn test_emergency_funds_recovery() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Adicionar fundos ao contrato
    setup.blockchain_wrapper.add_egld_to_account(
        &setup.contract_wrapper.address_ref(),
        &rust_biguint!(10000),
    );
    
    // Ativar modo de emergência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.emergency_mode().set(true);
        })
        .assert_ok();
    
    // Recuperar fundos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let initial_owner_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.owner_address));
            
            sc.emergency_withdraw();
            
            // Verificar que o saldo do contrato foi para zero
            assert_eq!(
                sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address()),
                managed_biguint!(0)
            );
            
            // Verificar que o proprietário recebeu os fundos
            let final_owner_balance = sc.blockchain().get_egld_balance(&managed_address!(&setup.owner_address));
            assert_eq!(final_owner_balance, initial_owner_balance + managed_biguint!(10000));
        })
        .assert_ok();
}

// Teste para validação da duração máxima de empréstimo
#[test]
fn test_maximum_loan_duration() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar duração máxima de empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.max_loan_term_days().set(180u64); // 180 dias
        })
        .assert_ok();
    
    // Tentar solicitar extensão que ultrapasse a duração máxima
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo próximo do limite máximo
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 10000u64 + 170u64 * 24u64 * 60u64 * 60u64, // 170 dias após criação
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Configurar taxa de extensão
            sc.extension_fee_percent().set(1000u64); // 10%
        })
        .assert_ok();
    
    // Tentar extender prazo além do limite máximo (170 + 30 > 180)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(550), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            // Tentar extender por 30 dias (ultrapassaria o limite máximo)
            sc.extend_loan_deadline(1u64, 30u64);
        })
        .assert_user_error("Extension would exceed maximum loan duration");
    
    // Extensão dentro do limite deve ser permitida
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(550), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            // Extender por apenas 10 dias (dentro do limite)
            sc.extend_loan_deadline(1u64, 10u64);
            
            // Verificar novo prazo
            let loan = sc.loans(1u64).get();
            let expected_due = 10000u64 + 170u64 * 24u64 * 60u64 * 60u64 + 10u64 * 24u64 * 60u64 * 60u64;
            assert_eq!(loan.due_timestamp, expected_due);
        })
        .assert_ok();
}

// Teste para lógica de blacklist de usuários
#[test]
fn test_blacklist_functionality() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Colocar um usuário na blacklist
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.add_to_blacklist(managed_address!(&setup.borrower_address));
            
            // Verificar que o usuário está na blacklist
            assert!(sc.is_blacklisted(&managed_address!(&setup.borrower_address)));
        })
        .assert_ok();
    
    // Simular resposta da pontuação de reputação
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let score_value = 800u64;
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                score_value,
            );
        })
        .assert_ok();
    
    // Usuário na blacklist tenta solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(10000);
            sc.request_loan();
        })
        .assert_user_error("User is blacklisted");
    
    // Remover usuário da blacklist
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.remove_from_blacklist(managed_address!(&setup.borrower_address));
            
            // Verificar que o usuário não está mais na blacklist
            assert!(!sc.is_blacklisted(&managed_address!(&setup.borrower_address)));
        })
        .assert_ok();
    
    // Agora o usuário deve conseguir solicitar empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(11000);
            
            let loan_id = sc.request_loan(managed_biguint!(10_000), LoanTerm::Standard);
            assert_eq!(loan_id, 1u64);
        })
        .assert_ok();
}

// Teste para verificar a atualização da pontuação de reputação
#[test]
fn test_reputation_score_update() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Pagar o empréstimo e atualizar pontuação
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            sc.blockchain().set_block_timestamp(15000); // Pago antes do vencimento
            
            // Mock para verificar se a função de atualização de pontuação foi chamada
            sc.expect_update_reputation_score().set(true);
            
            sc.repay_loan(1u64);
            
            // Verificar que a função de atualização foi chamada
            assert!(sc.update_reputation_score_called().get());
            
            // Verificar parâmetros corretos (pagamento em dia)
            assert_eq!(sc.last_reputation_update_borrower().get(), managed_address!(&setup.borrower_address));
            assert_eq!(sc.last_reputation_update_value().get(), 50u64); // Aumento por pagamento em dia
        })
        .assert_ok();
    
    // Configurar outro empréstimo para teste de atraso
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(2u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(2u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&2u64);
            
            // Resetar flags
            sc.expect_update_reputation_score().set(true);
            sc.update_reputation_score_called().set(false);
        })
        .assert_ok();
    
    // Pagar o empréstimo com atraso
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            sc.blockchain().set_block_timestamp(25000); // Pago após o vencimento
            
            sc.repay_loan(2u64);
            
            // Verificar que a função de atualização foi chamada
            assert!(sc.update_reputation_score_called().get());
            
            // Verificar parâmetros corretos (pagamento com atraso)
            assert_eq!(sc.last_reputation_update_borrower().get(), managed_address!(&setup.borrower_address));
            assert_eq!(sc.last_reputation_update_value().get(), -30i64 as u64); // Redução por pagamento em atraso
        })
        .assert_ok();
}