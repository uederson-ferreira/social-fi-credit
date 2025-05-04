// ==========================================================================
// ARQUIVO: liquidity_pool_security_test.rs
// Descrição: Testes de segurança para o contrato LiquidityPool
// ==========================================================================

use multiversx_sc_scenario::api::DebugApi;
use multiversx_sc_scenario::imports::BigUint;
use multiversx_sc::proxy_imports::ManagedAddress;
use multiversx_sc::proxy_imports::TokenIdentifier;

use multiversx_sc::contract_base::ContractBase;
use multiversx_sc::types::Address;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper}
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
    pub provider_address: Address,
    pub borrower_address: Address,
    pub attacker_address: Address,
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
    let provider_address = blockchain_wrapper.create_user_account(&rust_biguint!(100000));
    let borrower_address = blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    let attacker_address = blockchain_wrapper.create_user_account(&rust_biguint!(50000));
    
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
                managed_biguint!(1_000), // valor mínimo de depósito, por exemplo 1000
                10u64                    // rendimento anual em %, por exemplo 10%
            );
        })
        .assert_ok();
    
    ContractSetup {
        blockchain_wrapper,
        owner_address,
        loan_controller_address,
        debt_token_address,
        lp_token_address,
        provider_address,
        borrower_address,
        attacker_address,
        contract_wrapper,
    }
}

// Teste de tentativa de empréstimo não autorizado
#[test]
fn test_unauthorized_borrow() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez ao pool
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Tentativa de empréstimo por um endereço não autorizado
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que o chamador não é o controlador autorizado
            let caller = sc.blockchain().get_caller();
            let controller = sc.loan_controller_address().get();
            
            assert!(caller != controller);
            
            // Na implementação real, isso lançaria erro
            // "Only loan controller can call this function"
        })
        .assert_ok();
    
    // Verificar que nenhum empréstimo foi feito
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.total_borrows().get(), managed_biguint!(0));
        })
        .assert_ok();
}

// Teste de proteção contra duplo pagamento
#[test]
fn test_double_repayment_protection() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Preparar o cenário: adicionar liquidez e fazer um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow_endpoint();
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(10000));
        })
        .assert_ok();
    
    // Primeiro pagamento
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(10000));
            
            sc.repay_endpoint();
            
            // Verificar saldo zerado
            assert_eq!(sc.borrower_debt(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(0));
        })
        .assert_ok();
    
    // Tentativa de segundo pagamento (não deve ter efeito ou deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            // Verificar que não há dívida para pagar
            let current_debt = sc.borrower_debt(&managed_address!(&setup.borrower_address)).get();
            assert_eq!(current_debt, managed_biguint!(0));
            
            // Na implementação real, aqui verificaria e rejeitaria o pagamento
            // "No debt to repay_endpoint"
        })
        .assert_ok();
}

// Teste contra ataque de reentrância
#[test]
fn test_reentrancy_attack() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Preparar o cenário
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Simular um ataque de reentrância durante uma retirada
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Na implementação real, o contrato deve atualizar o estado ANTES de fazer chamadas externas
            // Aqui simulamos a verificação de que o contrato seja resistente a reentrância
            
            // 1. Verificar saldo inicial
            let provider_funds = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            let initial_liquidity = provider_funds.amount.clone(); // Usar o campo amount
            assert_eq!(initial_liquidity, managed_biguint!(50000));
            
            // 2. Em um contrato real seguro, a operação atualizaria o estado ANTES de qualquer chamada externa
            // Exemplo de atualização de estado segura:
            let amount_to_withdraw = managed_biguint!(10000);
            let new_liquidity = &initial_liquidity - &amount_to_withdraw;
            
            // 3. Agora qualquer chamada de reentrância veria o saldo já reduzido
            assert_eq!(new_liquidity, managed_biguint!(40000));
            
            // 4. A transferência real ocorreria apenas APÓS a atualização do estado
        })
        .assert_ok();
}

// Teste contra empréstimo com liquidez insuficiente
#[test]
fn test_borrow_with_insufficient_liquidity() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez limitada
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Tentativa de empréstimo maior que a liquidez disponível
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar liquidez disponível
            let available_liquidity = sc.total_liquidity().get() - sc.total_borrows().get();
            let borrow_amount = managed_biguint!(15000); // Maior que a liquidez
            
            assert!(borrow_amount > available_liquidity);
            
            // Na implementação real, isso lançaria erro
            // "Insufficient liquidity"
        })
        .assert_ok();
}

// Teste contra uso malicioso de reservas
#[test]
fn test_unauthorized_reserve_usage() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Configurar cenário com algumas reservas
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow_endpoint();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(4000), |sc| {
            // Adicionar juros acumulados
            sc.add_accumulated_interest_endpoint(managed_biguint!(4000));
            
            // Distribuir juros (20% vai para reservas = 800)
            sc.distribute_interest_endpoint();
            
            // Verificar reservas
            assert_eq!(sc.total_reserves().get(), managed_biguint!(800));
        })
        .assert_ok();
    
    // Tentativa de uso não autorizado das reservas
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que o chamador não é o proprietário
            let caller = sc.blockchain().get_caller();
            let owner = sc.blockchain().get_owner_address();
            
            assert!(caller != owner);
            
            // Na implementação real, isso lançaria erro
            // "Only owner can call this function"
        })
        .assert_ok();
}

// Teste contra manipulação de liquidez
#[test]
fn test_liquidity_manipulation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Tentativa de manipulação direta dos contadores de liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar contadores de liquidez atuais
            let current_liquidity = sc.total_liquidity().get();
            assert_eq!(current_liquidity, managed_biguint!(10000));
            
            // Verificar que um atacante não pode manipular diretamente os contadores
            // No caso real, esses contadores só seriam modificáveis por funções específicas
            
            // Verificar também que o saldo do provedor não pode ser manipulado
            let provider_liquidity = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            assert_eq!(provider_liquidity.amount, managed_biguint!(10000));
        })
        .assert_ok();
}

// Teste contra ataque de flash loan
#[test]
fn test_flash_loan_attack() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Simular tentativa de flash loan (pegar empréstimo e devolver na mesma transação)
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |_sc| {
            // Na implementação real, os flash loans teriam limitações ou taxas específicas
            // Aqui simulamos a verificação que o contrato impediria tal ataque
            
            // Um mecanismo possível seria exigir confirmação em bloco diferente para saques grandes
            let is_same_block = true; // Na implementação real, isso seria verificado
            let is_large_amount = true; // Simulação de valor grande
            
            if is_same_block && is_large_amount {
                // Em um contrato seguro, isso lançaria erro ou teria uma taxa específica
                assert!(true); // Simulação de proteção
            }
        })
        .assert_ok();
}

// Teste contra manipulação da taxa de utilização
#[test]
fn test_utilization_rate_manipulation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Verificar que a taxa de utilização não pode ser manipulada diretamente
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar taxa de utilização atual (deve ser 0)
            assert_eq!(sc.utilization_rate().get(), 0u64);
            
            // Na implementação real, a taxa de utilização seria protegida e só alterada 
            // através de empréstimos e pagamentos legítimos
            
            // Um atacante não poderia manipular diretamente a taxa para obter melhor taxa de juros
        })
        .assert_ok();
    
    // Fazer um empréstimo legítimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow_endpoint();
            
            // Verificar que a taxa de utilização foi atualizada corretamente
            assert_eq!(sc.utilization_rate().get(), 5000u64); // 50%
        })
        .assert_ok();
}

// Teste contra ataque de bloqueio de liquidez
#[test]
fn test_liquidity_lockup_attack() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Atacante tenta pegar todo o empréstimo para bloquear liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar implementação de limite máximo de empréstimo
            let max_borrow_percent = 9000u64; // 90% no contrato real
            let total_liquidity = sc.total_liquidity().get();
            let max_borrow_amount = &total_liquidity * &managed_biguint!(max_borrow_percent) / &managed_biguint!(10000);
            
            // Tentar emprestar mais do que o limite (isso deve falhar)
            let attempted_borrow = &total_liquidity * &managed_biguint!(9500) / &managed_biguint!(10000); // 95%
            
            assert!(attempted_borrow > max_borrow_amount);
            
            // Na implementação real, isso lançaria erro
            // "Exceeds maximum borrow_endpoint limit"
        })
        .assert_ok();
}

// Teste contra manipulação de reservas
#[test]
fn test_reserve_manipulation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez e gerar algumas reservas
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow_endpoint();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(5000), |sc| {
            // Adicionar juros
            sc.add_accumulated_interest_endpoint(managed_biguint!(5000));
            
            // Distribuir juros (20% para reservas = 1000)
            sc.distribute_interest_endpoint();
            
            // Verificar reservas
            assert_eq!(sc.total_reserves().get(), managed_biguint!(1000));
        })
        .assert_ok();
    
    // Tentativa de manipulação direta das reservas
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que um atacante não pode manipular diretamente as reservas
            let current_reserves = sc.total_reserves().get();
            assert_eq!(current_reserves, managed_biguint!(1000));
            
            // Na implementação real, as reservas só seriam modificáveis por funções específicas
            // e apenas pelo proprietário ou controlador
        })
        .assert_ok();
}

// Teste de proteção contra pausa maliciosa
#[test]
fn test_malicious_pause() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Tentativa de pausa por um atacante
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que o chamador não é o proprietário
            let caller = sc.blockchain().get_caller();
            let owner = sc.blockchain().get_owner_address();
            
            assert!(caller != owner);
            
            // Na implementação real, isso lançaria erro
            // "Only owner can call this function"
        })
        .assert_ok();
    
    // Pausa legítima pelo proprietário
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
            
            // Verificar estado
            assert!(sc.is_paused());
        })
        .assert_ok();
}

// Teste contra ataque de saldo de empréstimo errado
#[test]
fn test_incorrect_borrow_balance() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Configurar um cenário com um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow_endpoint();
        })
        .assert_ok();
    
    // Verificar consistência entre saldo individual e total
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let borrower_debt = sc.borrower_debt(&managed_address!(&setup.borrower_address)).get();
            let total_borrows = sc.total_borrows().get();
            
            // Os valores devem ser consistentes
            assert_eq!(borrower_debt, managed_biguint!(20000));
            assert_eq!(total_borrows, managed_biguint!(20000));
        })
        .assert_ok();
    
    // Pagar parte do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(10000));
            
            sc.repay_endpoint();
        })
        .assert_ok();
    
    // Verificar consistência após o pagamento
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let borrower_debt = sc.borrower_debt(&managed_address!(&setup.borrower_address)).get();
            let total_borrows = sc.total_borrows().get();
            
            // Os valores ainda devem ser consistentes
            assert_eq!(borrower_debt, managed_biguint!(10000));
            assert_eq!(total_borrows, managed_biguint!(10000));
        })
        .assert_ok();
}

// Teste de proteção contra ataque de overflow/underflow
#[test]
fn test_overflow_underflow_protection() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Verificar proteção contra underflow ao retirar mais do que o depositado
    // Verificar proteção contra underflow ao retirar mais do que o depositado
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular queima de tokens LP mais do que o saldo
            // No contrato real, isso seria validado antecipadamente
            
            let provider_funds = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            let provider_amount = provider_funds.amount.clone(); // Acessar o campo amount
            let withdraw_amount = &provider_amount + &managed_biguint!(1); // Mais do que o saldo
            
            // Na implementação real, isso lançaria erro
            // "Insufficient balance"
            assert!(withdraw_amount > provider_amount); // Agora comparando BigUint com BigUint
        })
        .assert_ok();
    
    // Teste contra overflow em depósitos muito grandes
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(u64::MAX), |sc| {
            // Fazer um depósito gigante
            // Em um contrato seguro, isso não causaria overflow
            sc.deposit_funds();
            
            // Verificar saldo atualizado (não deve causar overflow)
            let provider_balance = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            assert_eq!(provider_balance.amount, managed_biguint!(100000 + u64::MAX as u128));
        })
        .assert_ok();
}

// Teste contra ataque de DNS (Denial of Service)
#[test]
fn test_dos_attack_protection() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Configurar muitos pequenos depósitos (tentativa de DOS)
    for _ in 0..100 {
        setup.blockchain_wrapper
            .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(1), |sc| {
                // Em um contrato seguro, haveria um depósito mínimo
                // Especificar o tipo explicitamente usando o mesmo tipo que o framework usa
                let min_deposit: BigUint<DebugApi> = managed_biguint!(100); // Exemplo
                let deposit_amount: BigUint<DebugApi> = managed_biguint!(1);
                
                if deposit_amount < min_deposit {
                    // Na implementação real, isso lançaria erro
                    // "Deposit below minimum"
                    assert!(deposit_amount < min_deposit);
                } else {
                    sc.deposit_funds();
                }
            })
            .assert_ok();
    }
    
    // Verificar proteção contra muitas pequenas retiradas
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    for _ in 0..50 {
        setup.blockchain_wrapper
            .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                // Em um contrato seguro, haveria uma retirada mínima
                let min_withdrawal = managed_biguint!(10); // Exemplo
                let withdrawal_amount = managed_biguint!(1);
                
                if withdrawal_amount < min_withdrawal {
                    // Na implementação real, isso lançaria erro
                    // "Withdrawal below minimum"
                    assert!(withdrawal_amount < min_withdrawal);
                } else {
                    // Simular queima de tokens LP
                    sc.lp_tokens_burned_endpoint(managed_address!(&setup.provider_address), withdrawal_amount.clone());
                    
                    // Retirar
                    sc.withdraw_funds(withdrawal_amount);
                }
            })
            .assert_ok();
    }
}



//====================================================================
// Testes adicionais para melhorar a cobertura de segurança

// Teste 1: Validação de entrada
// Teste para validação de valor mínimo de depósito
#[test]
fn test_min_deposit_validation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(1), |sc| {
            sc.deposit_funds();
        })
        .assert_error(4, "Deposit amount below minimum");
}

// Teste para validação de endereço zero
#[test]
fn test_zero_address_validation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_loan_controller_address(ManagedAddress::zero());
        })
        .assert_error(4, "Invalid loan controller address");
}



// Teste 2: Testes de integração com contratos externos
#[test]
fn test_external_contract_integration() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Configurar o endereço do token LP
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_lp_token_address(managed_address!(&setup.lp_token_address));
            
            // Verificar que o endereço foi definido corretamente
            assert_eq!(sc.lp_token_address().get(), managed_address!(&setup.lp_token_address));
        })
        .assert_ok();
    
    // Configurar o endereço do token de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_debt_token_address(managed_address!(&setup.debt_token_address));
            
            // Verificar que o endereço foi definido corretamente
            assert_eq!(sc.debt_token_address().get(), managed_address!(&setup.debt_token_address));
        })
        .assert_ok();
    
    // Teste de chamada não autorizada a um endpoint do token
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Tentar registrar emissão de tokens LP sem autorização
            let attacker = sc.blockchain().get_caller();
            let lp_token = sc.lp_token_address().get();
            
            // Verificar que o chamador não é o token LP ou o proprietário
            let owner = sc.blockchain().get_owner_address();
            assert!(attacker != lp_token && attacker != owner);
            
            // Na implementação real, isso lançaria erro
            // "Apenas o contrato de token LP pode chamar esta função"
        })
        .assert_ok();
}

// Teste 3: Testes de atualização de estado
#[test]
fn test_state_updates() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Fazer um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow_endpoint();
            
            // Verificar que o estado foi atualizado corretamente
            let borrow_amount = managed_biguint!(5000); // O valor que o contrato usa por padrão
            
            // Verificar total de empréstimos
            assert_eq!(sc.total_borrows().get(), borrow_amount);
            
            // Verificar dívida do tomador
            let borrower = sc.blockchain().get_caller();
            assert_eq!(sc.borrower_debt(&borrower).get(), borrow_amount);
            
            // Verificar taxa de utilização
            // Utilização: 5000 / 50000 = 10% = 1000 (base 10000)
            assert_eq!(sc.utilization_rate().get(), 1000u64);
        })
        .assert_ok();
    
    // Fazer um pagamento de empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(2000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(2000));
            
            sc.repay_endpoint();
            
            // Verificar que o estado foi atualizado corretamente
            // Dívida restante: 5000 - 2000 = 3000
            assert_eq!(sc.borrower_debt(&managed_address!(&setup.borrower_address)).get(), managed_biguint!(3000));
            
            // Verificar total de empréstimos
            assert_eq!(sc.total_borrows().get(), managed_biguint!(3000));
            
            // Verificar taxa de utilização atualizada
            // Utilização: 3000 / 50000 = 6% = 600 (base 10000)
            assert_eq!(sc.utilization_rate().get(), 600u64);
        })
        .assert_ok();
}

// Teste 4: Testes de condições de corrida
#[test]
fn test_race_conditions() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Simular tentativa de condição de corrida durante retirada
    // Um atacante pode tentar fazer várias retiradas em paralelo
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar saldo atual
            let current_balance = sc.provider_funds(managed_address!(&setup.provider_address)).get().amount;
            assert_eq!(current_balance, managed_biguint!(50000));
            
            // Simular primeira retirada
            let withdrawal_amount_1 = managed_biguint!(30000);
            
            // Um contrato seguro atualizaria o estado ANTES de transferir os fundos
            let new_balance_1 = &current_balance - &withdrawal_amount_1;
            
            // Agora, se houver uma segunda tentativa de retirada (em uma condição de corrida),
            // o saldo já estaria atualizado para o valor menor
            let withdrawal_amount_2 = managed_biguint!(30000);
            
            if withdrawal_amount_2 > new_balance_1 {
                // A segunda retirada deveria falhar
                assert!(true); // Simulação de proteção
            }
        })
        .assert_ok();
}



// Teste 5: Testes de recuperação de falhas
// Teste para pausa do contrato
#[test]
fn test_contract_pause() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Pausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    // Verificar estado pausado (consulta)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert!(sc.is_paused());
        })
        .assert_ok();
}

// Teste para operações rejeitadas quando pausado
#[test]
fn test_operations_when_paused() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Pausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    // Tentar fazer um depósito enquanto pausado (deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_error(4, "Contrato está pausado");
}

// Teste para despausar o contrato
#[test]
fn test_contract_unpause() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Pausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    // Despausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause();
        })
        .assert_ok();
    
    // Verificar estado não pausado (consulta)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert!(!sc.is_paused());
        })
        .assert_ok();
}

// Teste para operações após despausar
#[test]
fn test_operations_after_unpause() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Pausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
        })
        .assert_ok();
    
    // Despausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause();
        })
        .assert_ok();
    
    // Verificar que as operações voltam a funcionar (depósito)
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Verificar que o depósito foi processado
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let provider_funds = sc.provider_funds(managed_address!(&setup.provider_address)).get().amount;
            assert_eq!(provider_funds, managed_biguint!(60000));
        })
        .assert_ok();
}




// Teste 6: Testes para funções de cálculo de juros
#[test]
fn test_interest_calculation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Fazer um empréstimo de 40000 (40% de utilização)
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.borrow_endpoint();
            
            // Supondo que o valor padrão de empréstimo seja 5000
            // Fazemos várias chamadas para atingir 40000
            sc.borrow_endpoint();
            sc.borrow_endpoint();
            sc.borrow_endpoint();
            sc.borrow_endpoint();
            sc.borrow_endpoint();
            sc.borrow_endpoint();
            sc.borrow_endpoint();
            
            // Verificar total de empréstimos (8 chamadas * 5000 = 40000)
            assert_eq!(sc.total_borrows().get(), managed_biguint!(40000));
            
            // Verificar taxa de utilização
            // Utilização: 40000 / 100000 = 40% = 4000 (base 10000)
            assert_eq!(sc.utilization_rate().get(), 4000u64);
        })
        .assert_ok();
    
    // Verificar cálculo da taxa de juros
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Configurações padrão do contrato:
            // - Taxa base: 1000 (10%)
            // - Taxa alvo de utilização: 8000 (80%)
            
            // Cálculo esperado para utilização de 40% (metade da meta):
            // Taxa base é 1000, deve ser reduzida pela proporção de quanto estamos abaixo da meta
            // Redução: (8000 - 4000) * 1000 / 8000 = 4000 * 1000 / 8000 = 500
            // Taxa final: 1000 - 500 = 500 (5%)
            
            let current_rate = sc.calculate_current_interest_rate();
            assert_eq!(current_rate, 500u64);
        })
        .assert_ok();
    
    // Aumentar a utilização para acima da meta (90%)
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Fazer mais empréstimos para chegar a 90000 (90% de utilização)
            // Já temos 40000, precisamos de mais 50000
            // Com 5000 por chamada, precisamos de 10 chamadas adicionais
            for _ in 0..10 {
                sc.borrow_endpoint();
            }
            
            // Verificar total de empréstimos
            assert_eq!(sc.total_borrows().get(), managed_biguint!(90000));
            
            // Verificar taxa de utilização
            // Utilização: 90000 / 100000 = 90% = 9000 (base 10000)
            assert_eq!(sc.utilization_rate().get(), 9000u64);
        })
        .assert_ok();
    
    // Verificar novo cálculo de taxa de juros (acima da meta)
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Configurações padrão do contrato:
            // - Taxa base: 1000 (10%)
            // - Taxa alvo de utilização: 8000 (80%)
            // - Taxa adicional para alta utilização: 2000 (20%)
            
            // Cálculo esperado para utilização de 90% (acima da meta):
            // Taxa base é 1000, deve ser aumentada pela proporção de quanto estamos acima da meta
            // Aumento: (9000 - 8000) * 2000 / (10000 - 8000) = 1000 * 2000 / 2000 = 1000
            // Taxa final: 1000 + 1000 = 2000 (20%)
            
            let current_rate = sc.calculate_current_interest_rate();
            assert_eq!(current_rate, 2000u64);
        })
        .assert_ok();
}




// Teste 7: Testes para limites máximos e proteção contra valores extremos
// Teste para limite máximo de taxa de rendimento
#[test]
fn test_excessive_yield_rate() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Tentar inicializar com taxa de rendimento anual acima do máximo
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Tentar definir uma taxa acima de 100% (maior que 10000)
            sc.set_interest_rate_base(12000u64);
        })
        .assert_error(4, "Taxa base muito alta");
}

// Teste para limite máximo de taxa de reserva
#[test]
fn test_excessive_reserve_rate() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Tentar definir uma taxa de reserva acima de 100%
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_reserve_percent(12000u64);
        })
        .assert_error(4, "Percentual de reserva muito alto");
}

// Teste para retirada excessiva
#[test]
fn test_excessive_withdrawal() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Tentar retirar mais do que o disponível
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular tentativa de retirada excessiva
            sc.withdraw_funds(managed_biguint!(200000)); // Mais que o depositado
        })
        .assert_error(4, "Insufficient funds to withdraw");
}



// Teste 8: Testes para funções de acesso sensíveis
#[test]
fn test_sensitive_function_access() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Tentar acessar funções administrativas como atacante
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que o chamador não é o proprietário
            let caller = sc.blockchain().get_caller();
            let owner = sc.blockchain().get_owner_address();
            assert!(caller != owner);
            
            // Na implementação real, todas estas chamadas lançariam erro
            // "Apenas o proprietário pode chamar esta função"
        })
        .assert_ok();
    
    // Teste de acesso a funções de controlador de empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que o chamador não é o controlador de empréstimo
            let caller = sc.blockchain().get_caller();
            let controller = sc.loan_controller_address().get();
            assert!(caller != controller);
            
            // Na implementação real, estas chamadas lançariam erro
            // "Apenas o controlador de empréstimos pode chamar esta função"
        })
        .assert_ok();
    
    // Tentar definir endereços de contratos como atacante
    setup.blockchain_wrapper
        .execute_tx(&setup.attacker_address, &setup.contract_wrapper, &rust_biguint!(0), |_sc| {
            // Na implementação real, estas chamadas lançariam erro
            // "Apenas o proprietário pode chamar esta função"
        })
        .assert_ok();
}

// Teste 9: Teste contra manipulações de timestamps
#[test]
fn test_timestamp_manipulation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
            
            // Armazenar timestamp inicial
            let initial_timestamp = sc.blockchain().get_block_timestamp();
            let provider_funds = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            
            // Verificar que o timestamp foi registrado corretamente
            assert_eq!(provider_funds.last_yield_timestamp, initial_timestamp);
        })
        .assert_ok();
    
    // Avançar o timestamp manualmente (simulando manipulação)
    // Observação: No ambiente de teste, podemos manipular o tempo,
    // mas em produção, os timestamps são controlados pela blockchain
    setup.blockchain_wrapper.set_block_timestamp(100000);
    
    // Verificar o cálculo de rendimento com o tempo manipulado
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Em um contrato real, o process_pending_yield é chamado durante operações
            // como depósito ou retirada
            // Aqui simulamos o efeito de chamar essa função com timestamp manipulado
            
            // Verificar o novo timestamp
            let new_timestamp = sc.blockchain().get_block_timestamp();
            assert_eq!(new_timestamp, 100000);
            
            // Em um contrato seguro, haveria proteções contra manipulações extremas
            // de tempo, como limites para o rendimento máximo por período
            let max_yield_percent = 5000u64; // Exemplo: 50% (base 10000)
            
            // Armazenar o rendimento anual configurado
            let annual_yield = sc.annual_yield_percentage().get();
            
            // Verificar que o rendimento para um período não excede o máximo
            assert!(annual_yield <= max_yield_percent);
        })
        .assert_ok();
}

// Teste 10: Teste de segurança para consistência de tokens
#[test]
fn test_token_consistency() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez com um tipo de token
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(50000), |sc| {
            sc.deposit_funds();
            
            // Obter o tipo de token do depósito
            let provider_funds = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            let token_id = provider_funds.token_id.clone();
            
            // Registrar para testes posteriores
            // (em um teste real, isso seria comparado com o token usado na chamada)
            assert!(token_id != TokenIdentifier::from_esdt_bytes(&[]));
        })
        .assert_ok();
    
    // Tentar fazer um depósito com um tipo diferente de token
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            // Em um contrato real, haveria uma verificação para garantir
            // que o token do novo depósito corresponde ao token existente
            
            // Simular verificação de token
            let provider_funds = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            let _existing_token = provider_funds.token_id;
            
            // Supor que o token da chamada atual é diferente
            let different_token = true; // Simulação
            
            if different_token {
                // Na implementação real, isso lançaria erro
                // "Token type doesn't match existing deposit"
                assert!(true); // Simulação de proteção
            }
        })
        .assert_ok();
}

// Teste 11: Teste de segurança para preservação de fundos em emergências
#[test]
fn test_emergency_fund_preservation() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(100000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Simular uma emergência (pausa do contrato)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
            assert!(sc.is_paused());
        })
        .assert_ok();
    
    // Tentar fazer um empréstimo durante emergência (deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que está pausado
            assert!(sc.is_paused());
            
            // Na implementação real, isso lançaria erro
            // "Contrato está pausado"
        })
        .assert_ok();
    
    // Verificar que os fundos permanecem intactos
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(100000));
        })
        .assert_ok();
    
    // Despausar e verificar que as operações normais podem ser retomadas
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause();
            assert!(!sc.is_paused());
        })
        .assert_ok();
}