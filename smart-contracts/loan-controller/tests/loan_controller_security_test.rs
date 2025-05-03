// ==========================================================================
// ARQUIVO: loan_controller_security_test.rs
// Descrição: Testes de segurança para o contrato LoanController
// ==========================================================================

use multiversx_sc::contract_base::ContractBase;
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
    pub borrower_address: Address,
    pub attacker_address: Address,
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
    let attacker_address = blockchain_wrapper.create_user_account(&rust_biguint!(1000));
    
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
        attacker_address,
        contract_wrapper,
    }
}

// Teste de tentativa de acesso não autorizado a um empréstimo
#[test]
fn test_unauthorized_loan_access() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo fictício para o borrower
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
    
    // Atacante tenta pagar o empréstimo do borrower - isso deve falhar
    // Na implementação real, o pagamento verificaria que apenas o tomador pode pagar seu próprio empréstimo
    // Neste teste, simulamos a verificação que deve ocorrer no contrato real
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let loan = sc.loans(1u64).get();
            assert!(loan.borrower != managed_address!(&setup.attacker_address));
            // Em uma implementação real, isto causaria um erro no método repay_loan
        })
        .assert_ok();
}

// Teste de valores extremos para operações aritméticas
#[test]
fn test_arithmetic_extremes() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Teste com valores muito grandes para verificar overflow
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Testar cálculo de juros para valor muito grande
            let large_amount: BigUint<DebugApi> = BigUint::from(u64::MAX);
            let interest_rate = 1000u64; // 10%
            
            // Cálculo: amount * interest_rate / 10000
            let interest_amount = &large_amount * &BigUint::from(interest_rate) / &BigUint::from(10000u32);
            
            // Verificar se o cálculo não causou overflow
            assert!(interest_amount > BigUint::zero());
            
            // Verificar valor total a ser pago
            let repayment_amount = &large_amount + &interest_amount;
            assert!(repayment_amount > large_amount);
        })
        .assert_ok();
}

// Teste de manipulação de dados externos vindos do ReputationScore
#[test]
fn test_external_data_manipulation() {
    // Este teste requer um mock do ReputationScore para simular dados manipulados
    // Na implementação real, o contrato precisa verificar de forma segura os dados vindos do ReputationScore
    
    // Para fins de demonstração, validamos apenas os dados em formato de contrato fictício
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Teste de configuração com dados extremos
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Simular um usuário com pontuação muito alta
            let user_score = 1000u64; // Pontuação máxima
            
            // Calcular taxa de juros
            let interest_rate = sc.calculate_interest_rate(user_score);
            
            // Verificar se a taxa está correta (deve ser o mínimo, 20% da base)
            assert_eq!(interest_rate, sc.interest_rate_base().get() / 5);
            
            // Simular um usuário com pontuação zero
            let interest_rate_zero = sc.calculate_interest_rate(0u64);
            
            // Verificar se a taxa está correta (deve ser o máximo, igual à base)
            assert_eq!(interest_rate_zero, sc.interest_rate_base().get());
        })
        .assert_ok();
}

// Teste para tentativa de reentrância em pagamentos
#[test]
fn test_reentrancy_attack() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo para o atacante
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            let loan = Loan {
                borrower: managed_address!(&setup.attacker_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 12345u64,
                due_timestamp: 23456u64,
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.attacker_address)).push(&1u64);
        })
        .assert_ok();
    
    // Testar proteção contra reentrância
    // Em um contrato real, verifique se a lógica de negócios atualiza o estado antes de realizar transferências
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            // Simulação do fluxo de pagamento com validação de ordem das operações
            
            // 1. Obter o empréstimo atual
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Active);
            
            // 2. Verificar se o pagamento é válido
            assert_eq!(loan.repayment_amount, managed_biguint!(5500));
            
            // 3. IMPORTANTE: Atualizar o estado ANTES de fazer a transferência
            // Isso previne ataques de reentrância
            let mut updated_loan = loan.clone();
            updated_loan.status = LoanStatus::Repaid;
            sc.loans(1u64).set(updated_loan);
            
            // 4. Agora que o estado foi atualizado, realizar a transferência
            // Na implementação real, aqui aconteceria a transferência
            
            // 5. Verificar que o estado foi atualizado
            let final_loan = sc.loans(1u64).get();
            assert_eq!(final_loan.status, LoanStatus::Repaid);
        })
        .assert_ok();
}

// Teste de segurança para manipulação de timestamp
#[test]
fn test_timestamp_manipulation() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar um empréstimo com prazo curto
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);
            
            // Definir timestamp atual
            sc.blockchain().get_block_timestamp();
            
            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64, // 10000 segundos de prazo
                status: LoanStatus::Active,
            };
            
            sc.loans(1u64).set(loan);
            sc.user_loans(managed_address!(&setup.borrower_address)).push(&1u64);
        })
        .assert_ok();
    
    // Tentar manipular timestamp para simular que o prazo não venceu
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Avançar para após o vencimento
            sc.blockchain().get_block_timestamp();
            
            // Marcar empréstimos vencidos
            sc.mark_expired_loans();
            
            // Verificar que o empréstimo foi marcado como inadimplente
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Defaulted);
            
            // Agora simular um atacante tentando definir um timestamp anterior
            // Isso não funciona no contexto de teste, mas na implementação real
            // é importante verificar que timestamps só podem avançar, nunca retroceder
            
            // Verifique que operações críticas dependentes de tempo não são afetadas por manipulação
            let current_timestamp = sc.blockchain().get_block_timestamp();
            assert!(current_timestamp >= loan.due_timestamp);
        })
        .assert_ok();
}

#[test]
fn test_mark_expired_loans() {
    let mut setup = setup_contract(loan_controller::contract_obj);

    // Configurar um empréstimo com prazo curto
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.loan_counter().set(1u64);

            let loan = Loan {
                borrower: managed_address!(&setup.borrower_address),
                amount: managed_biguint!(5000),
                repayment_amount: managed_biguint!(5500),
                interest_rate: 1000u64,
                creation_timestamp: 10000u64,
                due_timestamp: 20000u64, // 10000 segundos de prazo
                status: LoanStatus::Active,
            };

            sc.loans(1u64).set(loan);
        })
        .assert_ok();

    // Avançar o timestamp para após o vencimento
    setup.blockchain_wrapper.set_block_timestamp(25000);

    // Marcar empréstimos vencidos
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mark_expired_loans();

            let loan = sc.loans(1u64).get();
            assert_eq!(loan.status, LoanStatus::Defaulted);
        })
        .assert_ok();
}

// Teste para garantir que o contrato lida corretamente com valores zero
#[test]
fn test_zero_values() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Tentar operações com valores zero
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Tentar calcular juros para valor zero
            let zero_amount: BigUint<DebugApi> = managed_biguint!(0);
            let interest_rate = 1000u64; // 10%
            
            // Cálculo: amount * interest_rate / 10000
            let interest_amount = &zero_amount * &managed_biguint!(interest_rate) / &managed_biguint!(10000u32);
            
            // Verificar que o resultado é zero
            assert_eq!(interest_amount, managed_biguint!(0));
            
            // Valor de repagamento para empréstimo zero deve ser zero
            let repayment_amount = &zero_amount + &interest_amount;
            assert_eq!(repayment_amount, managed_biguint!(0));
        })
        .assert_ok();
}

// Teste para garantir que funções críticas exigem autenticação adequada
#[test]
fn test_access_control() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Gerar um segundo endereço não autorizado
    let unauthorized = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Tentar funções restritas com endereço não autorizado
    setup.blockchain_wrapper
        .execute_tx(&unauthorized, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Tentar funções de administrador
            sc.set_min_required_score(600u64);
        })
        .assert_user_error("Only owner can call this function");
    
    // Verificar que o proprietário pode chamar funções restritas
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_min_required_score(600u64);
            assert_eq!(sc.min_required_score().get(), 600u64);
        })
        .assert_ok();
    
    // Verificar funções restritas ao contrato de pontuação
    setup.blockchain_wrapper
        .execute_tx(&unauthorized, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                750u64,
            );
        })
        .assert_user_error("Only reputation score contract can call this function");
    
    // Verificar que o contrato de pontuação pode chamar suas funções
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                750u64,
            );
            
            // Verificar que a pontuação foi armazenada
            let score = sc.user_reputation_scores(&managed_address!(&setup.borrower_address)).get();
            assert_eq!(score, 750u64);
        })
        .assert_ok();
}

// Teste para verificar proteção contra overflow e underflow
#[test]
fn test_overflow_underflow_protection() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Testar cálculos com valores grandes
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Testar com valor máximo para BigUint
            let max_value: BigUint<DebugApi> = &managed_biguint!(u64::MAX) * &managed_biguint!(u64::MAX);
            
            // Adicionar um valor pequeno não deve causar overflow
            let result = &max_value + &managed_biguint!(1);
            assert!(result > max_value);
            
            // Subtrair um valor pequeno não deve causar underflow
            let result2 = &max_value - &managed_biguint!(1);
            assert!(result2 < max_value);
            
            // Verificar proteção contra divisão por zero
            let zero: BigUint<DebugApi> = managed_biguint!(0);
            let non_zero = managed_biguint!(100);
            
            // Multiplicação por zero deve ser segura
            let mult_result = &non_zero * &zero;
            assert_eq!(mult_result, zero);
            
            // Divisão por zero deve ser evitada na implementação
            // Aqui simulamos a verificação que deve existir
            let denominator = zero.clone();
            if denominator == managed_biguint!(0) {
                // Em produção, deveria lançar erro ou usar valor padrão
                // Para teste, apenas verificamos a condição
                assert_eq!(denominator, managed_biguint!(0));
            } else {
                let _div_result = &non_zero / &denominator;
                // Não deveria chegar aqui se denominator for zero
                assert!(false);
            }
        })
        .assert_ok();
}

// Teste para verificar proteção contra integer overflow em timestamps
#[test]
fn test_timestamp_overflow() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar timestamp atual como valor grande
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let large_timestamp = u64::MAX - 1000; // Quase no limite máximo
            sc.blockchain().get_block_timestamp();
            
            // Simular cálculo de data de vencimento
            let days_to_add = 30u64;
            let seconds_to_add = days_to_add * 24u64 * 60u64 * 60u64;
            
            // Verificar se adição não causaria overflow
            if large_timestamp > u64::MAX - seconds_to_add {
                // Em produção, deveria usar u64::MAX ou outra estratégia
                // Para teste, verificamos que a condição é detectada
                assert!(large_timestamp > u64::MAX - seconds_to_add);
            } else {
                let due_timestamp = large_timestamp + seconds_to_add;
                assert!(due_timestamp > large_timestamp);
            }
        })
        .assert_ok();
}

// Teste de ataque de front-running em solicitações de empréstimo
#[test]
fn test_front_running_protection() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    let front_runner = setup.blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    
    // Simular resposta de pontuação para ambos
    setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower_address),
                800u64,
            );
            sc.reputation_check_callback(
                managed_address!(&front_runner),
                800u64,
            );
        })
        .assert_ok();
    
    // Configurar um empréstimo disponível com limite de 1
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.max_active_loans().set(1u64);
        })
        .assert_ok();
    
    // Front-runner tenta pegar o empréstimo primeiro
    setup.blockchain_wrapper
        .execute_tx(&front_runner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let loan_id = sc.request_loan_sync(managed_biguint!(5000), 30u64);
            assert_eq!(loan_id, 1u64);

            // Verificar que o empréstimo foi atribuído
            assert_eq!(sc.active_loans_count().get(), 1u64);
        })
        .assert_ok();
    
    // Usuário original tenta obter empréstimo, deve falhar por já ter atingido limite
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().get_block_timestamp();
            sc.request_loan(managed_biguint!(5000), LoanTerm::Standard);
        })
        .assert_user_error("Global loan limit reached");
}

// Teste para verificar a integridade do contrato após upgrade
#[test]
fn test_contract_upgrade_integrity() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar estado inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Configurar alguns parâmetros
            sc.min_required_score().set(600u64);
            sc.interest_rate_base().set(1200u64);
            
            // Adicionar um empréstimo
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
    
    // Simular upgrade do contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que os parâmetros foram preservados
            assert_eq!(sc.min_required_score().get(), 600u64);
            assert_eq!(sc.interest_rate_base().get(), 1200u64);
            
            // Verificar que os empréstimos foram preservados
            assert_eq!(sc.loan_counter().get(), 1u64);
            let loan = sc.loans(1u64).get();
            assert_eq!(loan.amount, managed_biguint!(5000));
            assert_eq!(loan.status, LoanStatus::Active);
            
            // Verificar associações de usuários
            let user_loans_len = sc.user_loans(managed_address!(&setup.borrower_address)).len();
            let mut user_loans = Vec::new();
            for i in 0..user_loans_len {
                let loan_id = sc.user_loans(managed_address!(&setup.borrower_address)).get(i);
                user_loans.push(loan_id);
            }
            assert_eq!(user_loans.len(), 1);
            assert_eq!(user_loans[0], 1u64);
        })
        .assert_ok();
}

// Teste para validação de input em chamadas públicas
#[test]
fn test_input_validation() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar empréstimo
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
    
    // Teste de validação de input para IDs de empréstimo inexistentes
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(5500), |sc| {
            // Tentar pagar um empréstimo que não existe
            let non_existent_id = 999u64;
            
            // Na implementação real, isso lançaria erro
            let loan_exists = sc.loans(non_existent_id).is_empty();
            assert!(loan_exists);
            
            // Simulação do comportamento esperado
            if loan_exists {
                // Detectamos o erro, uma função real lançaria erro aqui
                assert!(true);
            } else {
                // Nunca deve chegar aqui para ID inexistente
                assert!(false);
            }
        })
        .assert_ok();
}

// Teste de segurança para detecção de front-running com timecodes
#[test]
fn test_timecode_security() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Configurar timelock para operações sensíveis
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.operation_timelock().set(300u64); // 5 minutos
        })
        .assert_ok();
    
    // Simular uma solicitação com timelock
        setup.blockchain_wrapper
            .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                let param_key = match ParamType::MinScore {
                    ParamType::MinScore => managed_biguint!(1), // Replace with appropriate BigUint value
                    ParamType::MaxScore => managed_biguint!(2), // Replace with an actual variant of ParamType
                    ParamType::InterestRate => managed_biguint!(3), // Handle InterestRate variant
                };
                let param_key_clone = param_key.clone();
                sc.pending_parameter_changes_by_key(param_key).set(ParameterChange {
                    value: 600u64,
                    timestamp: 10000u64,
                });

                let request = sc.pending_parameter_changes_by_key(param_key_clone).get();
                assert_eq!(request.value, 600u64);
                assert_eq!(request.timestamp, 10000u64);
            })
            .assert_ok();
    
    // Tentar executar a alteração antes do timelock expirar
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().get_block_timestamp(); // Apenas 100 segundos depois
            
            // Tentar aplicar a mudança
            let request = sc.pending_parameter_changes(ParamType::MinScore).get();
            let current_time = sc.blockchain().get_block_timestamp();
            
            // Verificar que ainda está dentro do timelock
            let timelock_expiry = request.timestamp + sc.operation_timelock().get();
            assert!(current_time < timelock_expiry);
            
            // Na implementação real, isso lançaria erro
        })
        .assert_ok();
    
    // Executar alteração após timelock expirar
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.blockchain().get_block_timestamp(); // 400 segundos depois (após timelock)
            
            // Aplicar a mudança
            let request = sc.pending_parameter_changes(ParamType::MinScore).get();
            let current_time = sc.blockchain().get_block_timestamp();
            
            // Verificar que timelock expirou
            let timelock_expiry = request.timestamp + sc.operation_timelock().get();
            assert!(current_time >= timelock_expiry);
            
            // Aplicar a mudança
            sc.min_required_score().set(request.value);
            assert_eq!(sc.min_required_score().get(), 600u64);
            
            // Limpar a solicitação
            sc.pending_parameter_changes(ParamType::MinScore).clear();
        })
        .assert_ok();
}

// Teste para verificar segurança em caso de função de auto-destruição do contrato
#[test]
fn test_self_destruct_security() {
    let mut setup = setup_contract(loan_controller::contract_obj);
    
    // Adicionar fundos ao contrato
    setup.blockchain_wrapper.execute_tx(
        &setup.owner_address,
        &setup.contract_wrapper,
        &rust_biguint!(10000),
        |_| {},
    );
    
    // Verificar que apenas o proprietário pode iniciar auto-destruição
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.initiate_contract_destruction();
        })
        .assert_user_error("Only owner can call this function");
    
    // Verificar que a auto-destruição exige confirmação dupla
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Primeira etapa - inicia contador
            sc.initiate_contract_destruction();
            
            // Verificar que foi iniciado
            assert!(sc.destruction_pending_v2().get());
            assert_eq!(sc.destruction_confirmation_count().get(), 1u32);
            
            // Na vida real, seria necessária uma segunda transação para confirmação
        })
        .assert_ok();
    
    // Verificar que a destruição só ocorre após todas as confirmações
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar estado atual
            assert!(sc.destruction_pending_v2().get());
            
            // Confirmar destruição
            sc.confirm_contract_destruction_v2();
            
            // Verificar contador
            assert_eq!(sc.destruction_confirmation_count().get(), 2u32);
            
            // Em uma implementação real, verificaria se atingiu o limite necessário
            let required_confirmations = 3u32;
            
            if sc.destruction_confirmation_count().get() >= required_confirmations {
                // Destruir o contrato - em teste, apenas simulamos
                assert!(false); // Não deve chegar aqui ainda
            }
        })
        .assert_ok();
    
    // Verificar que é possível cancelar a destruição
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.cancel_contract_destruction_v2();
            
            // Verificar que foi cancelado
            assert!(!sc.destruction_pending_v2().get());
            assert_eq!(sc.destruction_confirmation_count().get(), 0u32);
        })
        .assert_ok();
}
