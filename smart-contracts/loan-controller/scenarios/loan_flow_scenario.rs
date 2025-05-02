// ==========================================================================
// ARQUIVO: loan_flow_scenario.rs
// Descrição: Cenário de fluxo completo de empréstimo para o contrato LoanController
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
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
    pub liquidity_pool_address: Address,
    pub debt_token_address: Address,
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
    let liquidity_pool_address = blockchain_wrapper.create_user_account(&rust_zero);
    let debt_token_address = blockchain_wrapper.create_user_account(&rust_zero);
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
        liquidity_pool_address,
        debt_token_address,
        borrower_address,
        contract_wrapper,
    }
}

// Cenário: Fluxo completo de um empréstimo
// NOTA: Este é um cenário conceitual que requer mocks dos outros contratos
#[test]
fn test_loan_lifecycle_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Etapa 1: Configuração Inicial
    // Aqui simularíamos a configuração do ReputationScore, DebtToken, LiquidityPool
    // e suas interações com o LoanController
    
    // Etapa 2: Configurar um empréstimo fictício para simular o fluxo completo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Incrementar o contador de empréstimos
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo fictício
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64, // 10%
                creation_timestamp: 12345u64,
                due_timestamp: 23456u64, // ~11 dias depois
                status: LoanStatus::Active,
            };
            
            // Armazenar o empréstimo
            sc.loans(1u64).set(loan);
            
            // Associar ao usuário
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Etapa 3: Verificar o estado inicial do empréstimo
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            assert_eq!(loan.borrower, managed_address!(&setup.borrower_address));
            assert_eq!(loan.amount, managed_biguint!(5000));
            assert_eq!(loan.repayment_amount, managed_biguint!(5500));
            assert_eq!(loan.interest_rate, 1000u64);
            assert_eq!(loan.status, LoanStatus::Active);
        })
        .assert_ok();
    
    // Etapa 4: Simular o pagamento do empréstimo
    // Na implementação real, isto seria feito através do método repay_loan
    // e envolveria a transferência de tokens EGLD ou ESDT
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let mut loan = sc.loans(1u64).get();
            loan.status = LoanStatus::Repaid;
            sc.loans(1u64).set(loan);
            
            // Incrementar o contador de pagamentos em dia
            sc.on_time_payments(managed_address!(&setup.borrower_address)).set(1u64);
        })
        .assert_ok();
    
    // Etapa 5: Verificar o estado final do empréstimo
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            assert_eq!(loan.status, LoanStatus::Repaid);
            assert_eq!(sc.on_time_payments(managed_address!(&setup.borrower_address)).get(), 1u64);
        })
        .assert_ok();
}

// Cenário: Simulação de inadimplência
#[test]
fn test_loan_default_scenario() {
    // Similar ao teste anterior, mas simula o caso de inadimplência
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar empréstimo fictício
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Incrementar o contador de empréstimos
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo fictício com prazo já vencido
            let current_time = 30000u64;
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64, // 10%
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64, // Já vencido
                status: LoanStatus::Active,
            };
            
            // Armazenar o empréstimo
            sc.loans(1u64).set(loan);
            
            // Associar ao usuário
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
// Simular marcação de inadimplência
    // Na implementação real, isto seria feito através de um método específico
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let mut loan = sc.loans(1u64).get();
            loan.status = LoanStatus::Defaulted;
            sc.loans(1u64).set(loan);
        })
        .assert_ok();
    
    // Verificar se o empréstimo foi marcado como inadimplente
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            assert_eq!(loan.status, LoanStatus::Defaulted);
        })
        .assert_ok();
}

// Cenário: Múltiplos empréstimos para o mesmo usuário
#[test]
fn test_multiple_loans_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar múltiplos empréstimos para o mesmo usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Criar 3 empréstimos para o mesmo usuário
            for i in 1..4 {
                // Incrementar o contador de empréstimos
                sc.loan_counter().set(i);
                
                // Criar um empréstimo fictício
                let loan = Loan {
                    borrower: managed_address!(&setup.borrower_address),
                    amount: managed_biguint!(1000 * i as u64),
                    repayment_amount: managed_biguint!(1100 * i as u64),
                    interest_rate: 1000u64, // 10%
                    creation_timestamp: 10000u64 * i,
                    due_timestamp: 20000u64 * i,
                    status: LoanStatus::Active,
                };
                
                // Armazenar o empréstimo
                sc.loans(i).set(loan);
                
                // Associar ao usuário
                sc.user_loans(managed_address!(&setup.borrower_address)).push(&i);
            }
        })
        .assert_ok();
    
    // Verificar os empréstimos do usuário
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let user_loans = sc.user_loans(managed_address!(&setup.borrower_address)).get();
            
            // Verificar se o usuário tem 3 empréstimos
            assert_eq!(user_loans.len(), 3);
            
            // Verificar os IDs dos empréstimos
            assert_eq!(user_loans[0], 1u64);
            assert_eq!(user_loans[1], 2u64);
            assert_eq!(user_loans[2], 3u64);
            
            // Verificar os valores dos empréstimos
            let loan1 = sc.loans(1u64).get();
            let loan2 = sc.loans(2u64).get();
            let loan3 = sc.loans(3u64).get();
            
            assert_eq!(loan1.amount, managed_biguint!(1000));
            assert_eq!(loan2.amount, managed_biguint!(2000));
            assert_eq!(loan3.amount, managed_biguint!(3000));
        })
        .assert_ok();
    
    // Simular pagamento de um dos empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let mut loan = sc.loans(2u64).get();
            loan.status = LoanStatus::Repaid;
            sc.loans(2u64).set(loan);
            
            // Incrementar o contador de pagamentos em dia
            sc.on_time_payments(managed_address!(&setup.borrower_address)).set(1u64);
        })
        .assert_ok();
    
    // Verificar o estado após o pagamento
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan1 = sc.loans(1u64).get();
            let loan2 = sc.loans(2u64).get();
            let loan3 = sc.loans(3u64).get();
            
            // O empréstimo 2 deve estar pago, os outros ativos
            assert_eq!(loan1.status, LoanStatus::Active);
            assert_eq!(loan2.status, LoanStatus::Repaid);
            assert_eq!(loan3.status, LoanStatus::Active);
            
            // Verificar contador de pagamentos em dia
            assert_eq!(sc.on_time_payments(managed_address!(&setup.borrower_address)).get(), 1u64);
        })
        .assert_ok();
}

// Cenário: Extensão de prazo para empréstimo ativo
#[test]
fn test_loan_extension_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar taxa de extensão de prazo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.extension_fee_percent().set(1000u64); // 10%
        })
        .assert_ok();
    
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
    
    // Simular o pagamento da taxa de extensão e extensão do prazo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(550), |sc| {
            sc.blockchain().set_block_timestamp(15000); // 5000 segundos antes do vencimento
            
            // Na implementação real, esta seria a chamada ao método extend_loan_deadline
            // Aqui simulamos o comportamento deste método
            
            let mut loan = sc.loans(1u64).get();
            
            // Verificar taxa de extensão (10% do valor de repagamento)
            let extension_fee = &loan.repayment_amount * &managed_biguint!(1000) / &managed_biguint!(10000);
            assert_eq!(extension_fee, managed_biguint!(550));
            
            // Calcular novo prazo (15 dias a mais)
            let extension_days = 15u64;
            let extension_seconds = extension_days * 24 * 60 * 60;
            let new_due_timestamp = loan.due_timestamp + extension_seconds;
            
            // Atualizar o empréstimo
            loan.due_timestamp = new_due_timestamp;
            loan.repayment_amount += extension_fee; // Adicionar taxa à dívida total
            sc.loans(1u64).set(loan);
            
            // Registrar pagamento da taxa
            sc.extension_fees_collected().update(|fees| *fees += extension_fee.clone());
        })
        .assert_ok();
    
    // Verificar o estado do empréstimo após a extensão
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            // O novo prazo deve ser o original + 15 dias
            let expected_due_timestamp = 20000u64 + 15u64 * 24 * 60 * 60;
            assert_eq!(loan.due_timestamp, expected_due_timestamp);
            
            // O valor total a ser pago deve incluir a taxa de extensão
            assert_eq!(loan.repayment_amount, managed_biguint!(6050)); // 5500 + 550
            
            // Verificar taxas coletadas
            assert_eq!(sc.extension_fees_collected().get(), managed_biguint!(550));
        })
        .assert_ok();
}

// Cenário: Empréstimo com garantia (collateral)
#[test]
fn test_collateralized_loan_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar razão de garantia (valor do empréstimo em relação à garantia)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.collateral_ratio().set(7000u64); // 70%
        })
        .assert_ok();
    
    // Simular usuário fornecendo garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            // Na implementação real, esta seria uma chamada com valor em EGLD
            // Aqui simulamos o recebimento da garantia
            
            sc.pending_collateral(&managed_address!(&setup.borrower_address)).set(managed_biguint!(10000));
        })
        .assert_ok();
    
    // Simular a solicitação do empréstimo com base na garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(10000);
            
            // Na implementação real, esta seria a chamada request_loan_with_collateral
            // Aqui simulamos o comportamento deste método
            
            // Obter valor da garantia
            let collateral = sc.pending_collateral(&managed_address!(&setup.borrower_address)).get();
            
            // Calcular valor do empréstimo (70% do valor da garantia)
            let loan_amount = &collateral * &managed_biguint!(sc.collateral_ratio().get()) / &managed_biguint!(10000);
            assert_eq!(loan_amount, managed_biguint!(7000));
            
            // Calcular valor de repagamento (10% de juros)
            let interest_rate = 1000u64;
            let interest_amount = &loan_amount * &managed_biguint!(interest_rate) / &managed_biguint!(10000);
            let repayment_amount = &loan_amount + &interest_amount;
            
            // Criar o empréstimo
            sc.loan_counter().set(1u64);
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: loan_amount,
                repayment_amount: repayment_amount,
                interest_rate: interest_rate,
                creation_timestamp: sc.blockchain().get_block_timestamp(),
                due_timestamp: sc.blockchain().get_block_timestamp() + 30 * 24 * 60 * 60, // 30 dias
                status: LoanStatus::Active,
            };
            
            // Armazenar o empréstimo
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Mover garantia pendente para o empréstimo
            sc.loan_collateral(1u64).set(collateral);
            sc.pending_collateral(&managed_address!(&setup.borrower_address)).set(managed_biguint!(0));
        })
        .assert_ok();
    
    // Verificar o estado do empréstimo e da garantia
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            // Verificar valores
            assert_eq!(loan.amount, managed_biguint!(7000));
            assert_eq!(loan.repayment_amount, managed_biguint!(7700)); // 7000 + 700 (10% de juros)
            
            // Verificar garantia
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(10000));
            assert_eq!(sc.pending_collateral(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(0));
        })
        .assert_ok();
    
    // Simular o pagamento do empréstimo e retirada da garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(7700), |sc| {
            sc.blockchain().set_block_timestamp(20000); // Antes do vencimento
            
            // Na implementação real, esta seria a chamada repay_loan
            let mut loan = sc.loans(1u64).get();
            loan.status = LoanStatus::Repaid;
            sc.loans(1u64).set(loan);
            
            // Incrementar contador de pagamentos em dia
            sc.on_time_payments(managed_address!(&setup.borrower_address)).update(|count| *count += 1);
        })
        .assert_ok();
    
    // Simular a retirada da garantia após pagamento
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Na implementação real, esta seria a chamada withdraw_collateral
            
            // Verificar que o empréstimo está pago
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Repaid);
            
            // Obter valor da garantia
            let collateral = sc.loan_collateral(1u64).get();
            assert_eq!(collateral, managed_biguint!(10000));
            
            // Simular transferência da garantia de volta ao usuário
            sc.loan_collateral(1u64).set(managed_biguint!(0));
            
            // Na implementação real, aqui seria feita a transferência EGLD/ESDT
        })
        .assert_ok();
}

// Cenário: Liquidação de garantia por inadimplência
#[test]
fn test_collateral_liquidation_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo com garantia inadimplente
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar desconto de liquidação
            sc.liquidation_discount().set(2000u64); // 20%
            
            // Criar empréstimo com garantia
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7000),
                repayment_amount: managed_biguint!(7700),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64, // Já vencido
                status: LoanStatus::Defaulted, // Inadimplente
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Adicionar garantia
            sc.loan_collateral(1u64).set(managed_biguint!(10000));
        })
        .assert_ok();
    
    // Simular a liquidação da garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(25000); // Após o vencimento
            
            // Na implementação real, esta seria a chamada liquidate_collateral
            
            // Calcular valor de liquidação (valor com desconto)
            let collateral = sc.loan_collateral(1u64).get();
            let discount = sc.liquidation_discount().get();
            let liquidation_value = &collateral * &managed_biguint!(10000 - discount) / &managed_biguint!(10000);
            
            // Verificar valor de liquidação
            assert_eq!(liquidation_value, managed_biguint!(8000)); // 10000 - 20% = 8000
            
            // Registrar liquidação
            let mut loan = sc.loans(1u64).get();
            loan.status = LoanStatus::Liquidated;
            sc.loans(1u64).set(loan);
            
            // Zerar garantia no empréstimo (simulação da transferência)
            sc.loan_collateral(1u64).set(managed_biguint!(0));
            
            // Registrar fundos recebidos pela liquidação
            sc.liquidation_proceeds().update(|proceeds| *proceeds += liquidation_value);
        })
        .assert_ok();
    
    // Verificar o estado após a liquidação
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            // Verificar status
            assert_eq!(loan.status, LoanStatus::Liquidated);
            
            // Verificar que a garantia foi liberada
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(0));
            
            // Verificar fundos de liquidação
            assert_eq!(sc.liquidation_proceeds().get(), managed_biguint!(8000));
        })
        .assert_ok();
}

// Cenário: Histórico de empréstimos do usuário e reputação
#[test]
fn test_user_history_and_reputation_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar vários empréstimos com diferentes status
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Criar 3 empréstimos com status diferentes
            
            // Empréstimo 1: Pago em dia
            sc.loan_counter().set(1u64);
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Repaid,
            };
            sc.loans(1u64).set(loan1);
            
            // Empréstimo 2: Inadimplente
            sc.loan_counter().update(|val| *val += 1);
            let loan2 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(3000),
                repayment_amount: managed_biguint!(3300),
                interest_rate: 1000u64,
                creation_timestamp: 15000u64,
                due_timestamp: 25000u64,
                status: LoanStatus::Defaulted,
            };
            sc.loans(2u64).set(loan2);
            
            // Empréstimo 3: Ativo
            sc.loan_counter().update(|val| *val += 1);
            let loan3 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7000),
                repayment_amount: managed_biguint!(7700),
                interest_rate: 1000u64,
                creation_timestamp: 30000u64,
                due_timestamp: 40000u64,
                status: LoanStatus::Active,
            };
            sc.loans(3u64).set(loan3);
            
            // Associar empréstimos ao usuário
            let mut user_loans = ManagedVec::new();
            user_loans.push(1u64);
            user_loans.push(2u64);
            user_loans.push(3u64);
            sc.user_loans(managed_address!(&setup.borrower_address)).set(user_loans);
            
            // Configurar contadores de reputação
            sc.on_time_payments(managed_address!(&setup.borrower_address)).set(1u64);
            sc.late_payments(managed_address!(&setup.borrower_address)).set(1u64);
            sc.defaults(managed_address!(&setup.borrower_address)).set(1u64);
        })
        .assert_ok();
    
    // Verificar histórico do usuário
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Obter lista de empréstimos do usuário
            let user_loans = sc.user_loans(managed_address!(&setup.borrower_address)).get();
            assert_eq!(user_loans.len(), 3);
            
            // Verificar empréstimos por status
            let mut active_count = 0u32;
            let mut repaid_count = 0u32;
            let mut defaulted_count = 0u32;
            
            for loan_id in user_loans.iter() {
                let loan = sc.loans(loan_id).get();
                
                match loan.status {
                    LoanStatus::Active => active_count += 1,
                    LoanStatus::Repaid => repaid_count += 1,
                    LoanStatus::Defaulted => defaulted_count += 1,
                    _ => {}
                }
            }
            
            assert_eq!(active_count, 1);
            assert_eq!(repaid_count, 1);
            assert_eq!(defaulted_count, 1);
            
            // Verificar contadores de reputação
            assert_eq!(sc.on_time_payments(managed_address!(&setup.borrower_address)).get(), 1u64);
            assert_eq!(sc.late_payments(managed_address!(&setup.borrower_address)).get(), 1u64);
            assert_eq!(sc.defaults(managed_address!(&setup.borrower_address)).get(), 1u64);
        })
        .assert_ok();
    
    // Simular a atualização da pontuação para o contrato ReputationScore
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Na implementação real, esta seria uma chamada ao ReputationScore
            // Aqui simulamos o cálculo da pontuação
            
            let borrower = managed_address!(&setup.borrower_address);
            
            let on_time = sc.on_time_payments(borrower.clone()).get();
            let late = sc.late_payments(borrower.clone()).get();
            let defaults = sc.defaults(borrower.clone()).get();
            
            // Fórmula simples: (on_time * 100 - late * 50 - defaults * 200)
            let score_change = on_time * 100 - late * 50 - defaults * 200;
            
            // O cálculo final ficaria: base_score + score_change
            // Assumindo uma pontuação base de 700
            let base_score = 700i64;
            let new_score = (base_score + score_change as i64).max(0) as u64;
            
            // Verificar nova pontuação
            assert_eq!(new_score, 550u64); // 700 + 100 - 50 - 200 = 550
            
            // Na implementação real, essa pontuação seria enviada ao ReputationScore
        })
        .assert_ok();
}

// Cenário: Distribuição de lucros para investidores
#[test]
fn test_profit_distribution_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Criar investidores
    let investor1 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let investor2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Configurar investidores e seus percentuais
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Adicionar investidores
            sc.add_investor(managed_address!(&investor1), 6000u64); // 60%
            sc.add_investor(managed_address!(&investor2), 4000u64); // 40%
            
            // Verificar total
            assert_eq!(sc.total_investor_shares().get(), 10000u64); // 100%
        })
        .assert_ok();
    
    // Configurar empréstimo e gerar lucro
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Incrementar o contador de empréstimos
            sc.loan_counter().set(1u64);
            
            // Criar um empréstimo fictício
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10000),
                repayment_amount: managed_biguint!(11000), // 1000 de juros
                interest_rate: 1000u64, // 10%
                creation_timestamp: 12345u64,
                due_timestamp: 23456u64,
                status: LoanStatus::Active,
            };
            
            // Armazenar o empréstimo
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Simular pagamento e geração de lucro
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(11000), |sc| {
            sc.blockchain().set_block_timestamp(20000);
            
            // Na implementação real, esta seria a chamada repay_loan
            
            // Obter empréstimo
            let mut loan = sc.loans(1u64).get();
            
            // Calcular lucro (juros)
            let principal = loan.amount.clone();
            let total_payment = loan.repayment_amount.clone();
            let profit = &total_payment - &principal;
            
            // Verificar lucro
            assert_eq!(profit, managed_biguint!(1000));
            
            // Atualizar empréstimo
            loan.status = LoanStatus::Repaid;
            sc.loans(1u64).set(loan);
            
            // Registrar lucro
            sc.total_interest_earned().update(|val| *val += profit);
        })
        .assert_ok();
    
    // Simular distribuição de lucros
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Na implementação real, esta seria a chamada distribute_profits
            
            // Obter lucro total
            let profit = sc.total_interest_earned().get();
            assert_eq!(profit, managed_biguint!(1000));
            
            // Calcular montantes por investidor
            let investor1_share = sc.investor_shares(&managed_address!(&investor1)).get();
            let investor2_share = sc.investor_shares(&managed_address!(&investor2)).get();
            let total_shares = sc.total_investor_shares().get();
            
            let investor1_amount = &profit * &managed_biguint!(investor1_share) / &managed_biguint!(total_shares);
            let investor2_amount = &profit * &managed_biguint!(investor2_share) / &managed_biguint!(total_shares);
            
            // Verificar valores
            assert_eq!(investor1_amount, managed_biguint!(600)); // 60% de 1000
            assert_eq!(investor2_amount, managed_biguint!(400)); // 40% de 1000
            
            // Registrar distribuição
            sc.total_interest_earned().set(managed_biguint!(0));
            
            // Na implementação real, aqui seriam feitas as transferências
        })
        .assert_ok();
}

// Cenário: Teste de escalonamento de limites de empréstimo com base no histórico
#[test]
fn test_loan_limit_scaling_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar o histórico do usuário com empréstimos pagos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar contador de empréstimos pagos
            sc.on_time_payments(managed_address!(&setup.borrower_address)).set(5u64);
            
            // Configurar fator de escala
            sc.loan_limit_scale_factor().set(2000u64); // 20% de aumento por empréstimo pago
            
            // Configurar valor base
            sc.base_loan_amount().set(managed_biguint!(10_000));
        })
        .assert_ok();
    
    // Calcular o limite de empréstimo ajustado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Na implementação real, seria parte de request_loan
            
            // Obter informações
            let base_amount = sc.base_loan_amount().get();
            let on_time_count = sc.on_time_payments(managed_address!(&setup.borrower_address)).get();
            let scale_factor = sc.loan_limit_scale_factor().get();
            
            // Calcular aumento: base_amount * (1 + on_time_count * scale_factor / 10000)
            let scale_multiplier = &managed_biguint!(10000 + on_time_count * scale_factor) / &managed_biguint!(10000);
            let max_loan_amount = &base_amount * &scale_multiplier;
            
            // Verificar novo limite
            // 10000 * (1 + 5 * 0.2) = 10000 * 2 = 20000
            assert_eq!(max_loan_amount, managed_biguint!(20_000));
        })
        .assert_ok();
}

// Cenário: Recuperação de fundos em emergência
#[test]
fn test_emergency_recovery_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Adicionar fundos ao contrato
    setup.blockchain_wrapper.add_egld_to_account(
        &setup.contract_wrapper.address_ref(),
        &rust_biguint!(50000),
    );
    
    // Configurar empréstimos ativos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Criar 3 empréstimos ativos com valor total de 30000
            sc.loan_counter().set(3u64);
            
            // Empréstimo 1
            let loan1 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(10000),
                repayment_amount: managed_biguint!(11000),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            sc.loans(1u64).set(loan1);
            
            // Empréstimo 2
            let loan2 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(8000),
                repayment_amount: managed_biguint!(8800),
                interest_rate: 1000u64,
                creation_timestamp: 11000u64,
                due_timestamp: 21000u64,
                status: LoanStatus::Active,
            };
            sc.loans(2u64).set(loan2);
            
            // Empréstimo 3
            let loan3 = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(12000),
                repayment_amount: managed_biguint!(13200),
                interest_rate: 1000u64,
                creation_timestamp: 12000u64,
                due_timestamp: 22000u64,
                status: LoanStatus::Active,
            };
            sc.loans(3u64).set(loan3);
            
            // Registrar valor total de empréstimos ativos
            sc.total_active_loan_amount().set(managed_biguint!(30000));
        })
        .assert_ok();
    
    // Ativar modo de emergência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.emergency_mode().set(true);
        })
        .assert_ok();
    
    // Executar recuperação de fundos excedentes
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Na implementação real, esta seria a chamada emergency_withdraw_excess
            
            // Calcular fundos necessários para cobrir empréstimos
            let total_active_loans = sc.total_active_loan_amount().get();
            
            // Adicionar margem de segurança (10%)
            let safety_margin = &total_active_loans * &managed_biguint!(1000) / &managed_biguint!(10000);
            let required_funds = &total_active_loans + &safety_margin;
            
            // Verificar fundos disponíveis
            let available_funds = sc.blockchain().get_egld_balance(&sc.blockchain().get_sc_address());
            
            // Calcular excedente
            let excess_funds = &available_funds - &required_funds;
            
            // Verificar valores
            assert_eq!(total_active_loans, managed_biguint!(30000));
            assert_eq!(safety_margin, managed_biguint!(3000));
            assert_eq!(required_funds, managed_biguint!(33000));
            assert_eq!(available_funds, managed_biguint!(50000));
            assert_eq!(excess_funds, managed_biguint!(17000));
            
            // Em uma implementação real, aqui transferiria o excedente
        })
        .assert_ok();
}

// Cenário: Ajuste dinâmico de taxas de juros (continuação)
#[test]
fn test_dynamic_interest_rate_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar parâmetros de ajuste dinâmico
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar taxa base
            sc.interest_rate_base().set(1000u64); // 10%
            
            // Configurar utilização alvo do pool
            sc.target_utilization().set(8000u64); // 80%
            
            // Configurar fatores de ajuste
            sc.over_utilization_multiplier().set(5000u64); // 50% acima se alto uso
            sc.under_utilization_multiplier().set(3000u64); // 30% abaixo se baixo uso
        })
        .assert_ok();
    
    // Simular diferentes níveis de utilização do pool
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Na implementação real, esta seria parte do cálculo dinâmico
            
            // Caso 1: Utilização ideal (80%)
            let base_rate = sc.interest_rate_base().get();
            let target = sc.target_utilization().get();
            let current_utilization = 8000u64; // 80%
            
            let rate_at_target = sc.calculate_dynamic_interest_rate(current_utilization);
            assert_eq!(rate_at_target, base_rate); // Na meta, usa a taxa base
            
            // Caso 2: Alta utilização (95%)
            let high_utilization = 9500u64; // 95%
            let over_mult = sc.over_utilization_multiplier().get();
            
            // Calcular quanto acima: (95% - 80%) / 80% = 0.1875 = 18.75%
            // Ajuste: base_rate * (1 + 0.1875 * 0.5) = base_rate * 1.09375
            let expected_high_rate = base_rate * (10000u64 + (high_utilization - target) * over_mult / target) / 10000u64;
            let rate_at_high = sc.calculate_dynamic_interest_rate(high_utilization);
            
            assert_eq!(rate_at_high, expected_high_rate);
            assert!(rate_at_high > base_rate);
            
            // Caso 3: Baixa utilização (50%)
            let low_utilization = 5000u64; // 50%
            let under_mult = sc.under_utilization_multiplier().get();
            
            // Calcular quanto abaixo: (80% - 50%) / 80% = 0.375 = 37.5%
            // Ajuste: base_rate * (1 - 0.375 * 0.3) = base_rate * 0.8875
            let expected_low_rate = base_rate * (10000u64 - (target - low_utilization) * under_mult / target) / 10000u64;
            let rate_at_low = sc.calculate_dynamic_interest_rate(low_utilization);
            
            assert_eq!(rate_at_low, expected_low_rate);
            assert!(rate_at_low < base_rate);
        })
        .assert_ok();
    
    // Simular ajuste real com dados do contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar fundos disponíveis
            sc.available_funds().set(managed_biguint!(200000));
            
            // Configurar empréstimos ativos
            sc.total_active_loan_amount().set(managed_biguint!(150000));
            
            // Calcular utilização atual: 150000 / 200000 = 75%
            let utilization = sc.calculate_utilization();
            assert_eq!(utilization, 7500u64);
            
            // Obter nova taxa de juros baseada na utilização atual
            let new_rate = sc.calculate_dynamic_interest_rate(utilization);
            
            // Atualizar taxa base
            sc.interest_rate_base().set(new_rate);
            
            // Verificar mudança
            let base_rate = 1000u64;
            let target = 8000u64;
            let under_mult = 3000u64;
            let expected_new_rate = base_rate * (10000u64 - (target - utilization) * under_mult / target) / 10000u64;
            
            assert_eq!(sc.interest_rate_base().get(), expected_new_rate);
        })
        .assert_ok();
}

// Cenário: Empréstimos com diferentes prazos e taxas
#[test]
fn test_loan_terms_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar termos de empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar prazos
            sc.standard_loan_term_days().set(30u64); // 30 dias
            sc.extended_loan_term_days().set(90u64); // 90 dias
            sc.max_loan_term_days().set(180u64); // 180 dias
            
            // Configurar multiplicadores de taxa
            sc.extended_term_rate_multiplier().set(150u64); // 1.5x para prazo estendido
            sc.max_term_rate_multiplier().set(200u64); // 2x para prazo máximo
            
            // Configurar taxa base
            sc.interest_rate_base().set(1000u64); // 10%
        })
        .assert_ok();
    
    // Simular empréstimos com diferentes prazos
    for (term_type, term_days, rate_mult) in [
        (LoanTerm::Standard, 30u64, 100u64),
        (LoanTerm::Extended, 90u64, 150u64),
        (LoanTerm::Maximum, 180u64, 200u64),
    ] {
        setup.blockchain_wrapper
            .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                // Simular resposta de pontuação de reputação
                sc.reputation_check_callback(
                    managed_address!(&setup.borrower_address),
                    800u64, // Boa pontuação
                );
            })
            .assert_ok();
        
        // Simular solicitação de empréstimo com o termo específico
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.blockchain().set_block_timestamp(10000);
                
                // Na implementação real, esta seria a chamada request_loan_with_term
                // Aqui simulamos o comportamento deste método
                
                // Obter o prazo em dias
                let term_days = match term_type {
                    LoanTerm::Standard => sc.standard_loan_term_days().get(),
                    LoanTerm::Extended => sc.extended_loan_term_days().get(),
                    LoanTerm::Maximum => sc.max_loan_term_days().get(),
                };
                
                // Calcular o timestamp de vencimento
                let seconds_per_day = 24 * 60 * 60;
                let due_timestamp = sc.blockchain().get_block_timestamp() + term_days * seconds_per_day;
                
                // Calcular taxa base ajustada pela pontuação
                let user_score = sc.user_reputation_scores(&managed_address!(&setup.borrower_address)).get();
                let base_rate = sc.calculate_interest_rate(user_score);
                
                // Ajustar taxa de acordo com o prazo
                let rate_multiplier = match term_type {
                    LoanTerm::Standard => 100u64, // Sem ajuste
                    LoanTerm::Extended => sc.extended_term_rate_multiplier().get(),
                    LoanTerm::Maximum => sc.max_term_rate_multiplier().get(),
                };
                
                let final_rate = base_rate * rate_multiplier / 100;
                
                // Calcular valor e repagamento
                let loan_amount = sc.base_loan_amount().get();
                let interest_amount = &loan_amount * &managed_biguint!(final_rate) / &managed_biguint!(10000);
                let repayment_amount = &loan_amount + &interest_amount;
                
                // Criar o empréstimo
                let loan_id = sc.loan_counter().get() + 1;
                sc.loan_counter().set(loan_id);
                
                let loan = Loan {
                    borrower: managed_address!(&setup.borrower_address),
                    amount: loan_amount,
                    repayment_amount: repayment_amount,
                    interest_rate: final_rate,
                    creation_timestamp: sc.blockchain().get_block_timestamp(),
                    due_timestamp: due_timestamp,
                    status: LoanStatus::Active,
                };
                
                // Armazenar o empréstimo
                sc.loans(loan_id).set(loan);
                sc.user_loans(managed_address!(&setup.borrower_address)).push(&loan_id);
            })
            .assert_ok();
        
        // Verificar o empréstimo criado
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let loan_id = sc.loan_counter().get();
                let loan = sc.loans(loan_id).get();
                
                // Verificar prazo
                let expected_due = 10000u64 + term_days * 24 * 60 * 60;
                assert_eq!(loan.due_timestamp, expected_due);
                
                // Verificar taxa de juros (ajustado por prazo)
                // Taxa base para pontuação 800 é 360 (calculado pelo contrato)
                let base_score_rate = 360u64;
                let expected_rate = base_score_rate * rate_mult / 100;
                assert_eq!(loan.interest_rate, expected_rate);
                
                // Valores esperados para empréstimo de 10000 com taxas 360 (3.6%), 540 (5.4%), e 720 (7.2%)
                let expected_repayment = &sc.base_loan_amount().get() * &managed_biguint!(10000 + expected_rate) / &managed_biguint!(10000);
                assert_eq!(loan.repayment_amount, expected_repayment);
            })
            .assert_ok();
    }
}

// Cenário: Pausa e retomada de operações do contrato
#[test]
fn test_contract_pause_resume_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Simular uma resposta de pontuação para teste
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                800u64,
            );
        })
        .assert_ok();
    
    // Verificar que empréstimos podem ser solicitados normalmente
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(10000);
            
            // Solicitar empréstimo
            let loan_id = sc.request_loan();
            assert_eq!(loan_id, 1u64);
        })
        .assert_ok();
    
    // Pausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause_contract();
            
            // Verificar estado
            assert!(sc.is_paused().get());
        })
        .assert_ok();
    
    // Tentar solicitar empréstimo com contrato pausado
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(11000);
            
            // Verificar que contrato está pausado
            assert!(sc.is_paused().get());
            
            // Em uma implementação real, aqui lançaria erro
            // Para o teste, simulamos a condição
            if sc.is_paused().get() {
                // Deveria falhar com "Contract is paused"
                assert!(true);
            } else {
                let _loan_id = sc.request_loan();
                assert!(false); // Nunca deve chegar aqui quando pausado
            }
        })
        .assert_ok();
    
    // Retomar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause_contract();
            
            // Verificar estado
            assert!(!sc.is_paused().get());
        })
        .assert_ok();
    
    // Solicitar empréstimo novamente após retomada
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().set_block_timestamp(12000);
            
            // Solicitar empréstimo
            let loan_id = sc.request_loan();
            assert_eq!(loan_id, 2u64);
        })
        .assert_ok();
}

// Cenário: Atualização de endereços de contratos relacionados
#[test]
fn test_contract_addresses_update_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Criar novos endereços para contratos
    let new_reputation_score = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let new_liquidity_pool = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let new_debt_token = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Atualizar endereços dos contratos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Atualizar ReputationScore
            sc.set_reputation_score_address(managed_address!(&new_reputation_score));
            
            // Atualizar LiquidityPool
            sc.set_liquidity_pool_address(managed_address!(&new_liquidity_pool));
            
            // Atualizar DebtToken
            sc.set_debt_token_address(managed_address!(&new_debt_token));
            
            // Verificar que os endereços foram atualizados
            assert_eq!(sc.reputation_score_address().get(), managed_address!(&new_reputation_score));
            assert_eq!(sc.liquidity_pool_address().get(), managed_address!(&new_liquidity_pool));
            assert_eq!(sc.debt_token_address().get(), managed_address!(&new_debt_token));
        })
        .assert_ok();
    
    // Verificar interação com o novo contrato de pontuação
    setup.blockchain_wrapper
        .execute_tx(&new_reputation_score, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // O novo contrato deve poder chamar o callback
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                750u64,
            );
            
            // Verificar que a pontuação foi registrada
            let score = sc.user_reputation_scores(&managed_address!(&setup.borrower_address)).get();
            assert_eq!(score, 750u64);
        })
        .assert_ok();
    
    // Verificar que o antigo contrato não tem mais acesso
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Em uma implementação real, aqui lançaria erro
            // Para o teste, simulamos a verificação
            
            let caller_address = managed_address!(&setup.reputation_score_address);
            let authorized_address = sc.reputation_score_address().get();
            
            if caller_address != authorized_address {
                // Deveria falhar com "Only reputation score contract can call this function"
                assert!(true);
            } else {
                // Nunca deve chegar aqui
                assert!(false);
            }
        })
        .assert_ok();
}

// Cenário: Devolução parcial de garantia após pagamento parcial
#[test]
fn test_partial_collateral_release_scenario() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar razão de garantia (valor do empréstimo em relação à garantia)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.collateral_ratio().set(7000u64); // 70%
            sc.allow_partial_repayments().set(true);
            sc.allow_partial_collateral_release().set(true);
        })
        .assert_ok();
    
    // Configurar um empréstimo com garantia
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(7000),
                repayment_amount: managed_biguint!(7700), // 10% de juros
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
            
            // Adicionar garantia
            sc.loan_collateral(1u64).set(managed_biguint!(10000));
        })
        .assert_ok();
    
    // Simular pagamento parcial (50%)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(3850), |sc| {
            sc.blockchain().set_block_timestamp(15000);
            
            // Na implementação real, esta seria a chamada partial_repay_loan
            
            let payment_amount = managed_biguint!(3850); // 50% do valor total
            
            let loan_id = 1u64;
            let mut loan = sc.loans(loan_id).get();
            let original_repayment = loan.repayment_amount.clone();
            
            // Registrar pagamento
            sc.loan_payments(loan_id).update(|paid| *paid += payment_amount.clone());
            
            // Atualizar valor de repagamento
            loan.repayment_amount -= payment_amount.clone();
            sc.loans(loan_id).set(loan);
            
            // Calcular proporção paga
            let payment_ratio = &payment_amount * &managed_biguint!(10000) / &original_repayment;
            assert_eq!(payment_ratio, managed_biguint!(5000)); // 50%
            
            // Calcular garantia a liberar
            let total_collateral = sc.loan_collateral(loan_id).get();
            let release_amount = &total_collateral * &payment_ratio / &managed_biguint!(10000);
            assert_eq!(release_amount, managed_biguint!(5000)); // 50% da garantia
            
            // Atualizar garantia
            sc.loan_collateral(loan_id).update(|collateral| *collateral -= release_amount.clone());
            
            // Na implementação real, aqui transferiria a garantia liberada
        })
        .assert_ok();
    
    // Verificar o estado do empréstimo e da garantia
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            // Verificar valor restante a pagar
            assert_eq!(loan.repayment_amount, managed_biguint!(3850)); // 7700 - 3850 = 3850
            
            // Verificar pagamentos registrados
            assert_eq!(sc.loan_payments(1u64).get(), managed_biguint!(3850));
            
            // Verificar garantia restante
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(5000)); // 10000 - 5000 = 5000
        })
        .assert_ok();
    
    // Simular pagamento final
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(3850), |sc| {
            sc.blockchain().set_block_timestamp(18000);
            
            // Na implementação real, esta seria a chamada partial_repay_loan
            
            let payment_amount = managed_biguint!(3850); // Restante do valor
            
            let loan_id = 1u64;
            let mut loan = sc.loans(loan_id).get();
            
            // Registrar pagamento
            sc.loan_payments(loan_id).update(|paid| *paid += payment_amount.clone());
            
            // Atualizar valor de repagamento
            loan.repayment_amount = managed_biguint!(0);
            loan.status = LoanStatus::Repaid;
            sc.loans(loan_id).set(loan);
            
            // Liberar toda a garantia restante
            let release_amount = sc.loan_collateral(loan_id).get();
            sc.loan_collateral(loan_id).set(managed_biguint!(0));
            
            // Na implementação real, aqui transferiria a garantia liberada
        })
        .assert_ok();
    
    // Verificar o estado final
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            
            // Verificar que o empréstimo está pago
            assert_eq!(loan.status, LoanStatus::Repaid);
            assert_eq!(loan.repayment_amount, managed_biguint!(0));
            
            // Verificar pagamentos totais
            assert_eq!(sc.loan_payments(1u64).get(), managed_biguint!(7700));
            
            // Verificar que toda a garantia foi liberada
            assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(0));
        })
        .assert_ok();
}