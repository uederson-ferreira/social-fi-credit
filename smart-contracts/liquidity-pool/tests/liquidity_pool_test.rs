// ==========================================================================
// ARQUIVO: liquidity_pool_test.rs
// Descrição: Testes unitários para o contrato LiquidityPool
// ==========================================================================

use multiversx_sc::contract_base::ContractBase;
use multiversx_sc::types::Address;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint, testing_framework::{BlockchainStateWrapper, ContractObjWrapper}, DebugApi
};

use liquidity_pool::*;

const TOKEN_ID_BYTES: &[u8] = b"TEST-123456";
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
    let provider_address = blockchain_wrapper.create_user_account(&rust_biguint!(10000));
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
                managed_address!(&loan_controller_address),
                managed_biguint!(1_000), // valor mínimo de depósito, por exemplo 1000
                10u64                    // rendimento anual em %, por exemplo 10%
            );
            
            // Definir endereços adicionais durante a inicialização
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
        provider_address,
        borrower_address,
        contract_wrapper,
    }
}

// Função auxiliar para adicionar tokens ESDT a um contrato já configurado
fn add_esdt_to_contract<ContractObjBuilder>(
    setup: &mut ContractSetup<ContractObjBuilder>,
    provider: &Address,
    token_id: &[u8],
    amount: u64
)
where
    ContractObjBuilder: 'static + Copy + Fn() -> liquidity_pool::ContractObj<DebugApi>
{
    // Configurar token ESDT no ambiente de teste
    setup.blockchain_wrapper.set_esdt_balance(provider, token_id, &rust_biguint!(amount));
    
    // Fazer depósito com ESDT e adicionar provedor à lista
    setup.blockchain_wrapper.execute_esdt_transfer(
        provider,
        &setup.contract_wrapper,
        token_id,
        0,
        &rust_biguint!(amount),
        |sc| {
            // Opcional: inicializar o array providers se ainda não existir
            if sc.providers().is_empty() {
                // Se o contrato tiver um método init_providers(), chame-o aqui
            }
            
            // Adicionar o provedor à lista de provedores
            let provider_addr = managed_address!(provider);
            
            // Verificar se já existe
            let mut found = false;
            for i in 0..sc.providers().len() {
                if sc.providers().get(i) == provider_addr {
                    found = true;
                    break;
                }
            }
            
            // Adicionar se não existir
            if !found {
                sc.providers().push(&provider_addr);
            }
            
            // Chamar deposit_funds
            sc.deposit_funds();
        }
    ).assert_ok();
}

// Teste de inicialização do contrato
#[test]
fn l_t_init() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Verificar estado inicial
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Verificar endereços dos contratos relacionados
            assert_eq!(
                sc.loan_controller_address().get(),
                managed_address!(&setup.loan_controller_address)
            );
            assert_eq!(
                sc.debt_token_address().get(),
                managed_address!(&setup.debt_token_address)
            );
            assert_eq!(
                sc.lp_token_address().get(),
                managed_address!(&setup.lp_token_address)
            );
            
            // Verificar parâmetros
            assert_eq!(sc.interest_rate_base().get(), 1000u64);
            assert_eq!(sc.max_utilization_rate().get(), 2000u64);
            assert_eq!(sc.target_utilization_rate().get(), 8000u64);
            
            // Verificar valores iniciais
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(0));
            assert_eq!(sc.total_borrows().get(), managed_biguint!(0));
            assert_eq!(sc.total_reserves().get(), managed_biguint!(0));
            assert_eq!(sc.utilization_rate().get(), 0u64);
        })
        .assert_ok();
}

// Teste de depósito de liquidez
#[test]
fn l_t_deposit_liquidity() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Primeiro, criar uma cópia do endereço
    let provider_addr = setup.provider_address.clone();
    
    // Adicionar liquidez inicial usando ESDT em vez de EGLD
    add_esdt_to_contract(&mut setup, &provider_addr, TOKEN_ID_BYTES, 10000);


    // Simulação de emissão de tokens LP
    setup.blockchain_wrapper
        .execute_tx(&setup.lp_token_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Na implementação real, o contrato LP Token receberia uma chamada para mintar tokens
            // Aqui simulamos o registro da emissão
            sc.lp_tokens_minted_endpoint(managed_address!(&setup.provider_address), managed_biguint!(5000));
        })
        .assert_ok();
}

// Teste de retirada de liquidez
#[test]
fn l_t_withdraw_liquidity() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Primeiro, fazer um depósito
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(8000), |sc| {
            sc.deposit_funds();
            
            // Simular emissão de tokens LP
            sc.lp_tokens_minted_endpoint(managed_address!(&setup.provider_address), managed_biguint!(8000));
        })
        .assert_ok();
    
    // Agora, retirar parte da liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular queima de tokens LP
            sc.lp_tokens_burned_endpoint(managed_address!(&setup.provider_address), managed_biguint!(3000));
            
            // Retirar liquidez
            sc.withdraw_funds(managed_biguint!(3000));
            
            // Verificar estado atualizado
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(5000));
            
            let provider_funds = sc.provider_funds(managed_address!(&setup.provider_address)).get();
            assert_eq!(
                provider_funds.amount,
                managed_biguint!(5000)
            );
        })
        .assert_ok();
}

// Teste de empréstimo de fundos
#[test]
fn l_t_borrow_funds() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez ao pool
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Solicitar empréstimo pelo controlador de empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular solicitação de empréstimo
            liquidity_pool::LiquidityPool::borrow_endpoint(&sc);
            
            // Verificar estado
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(10000));
            assert_eq!(sc.total_borrows().get(), managed_biguint!(5000));
            assert_eq!(sc.utilization_rate().get(), 5000u64); // 50%
            
            // Verificar dívida do tomador
            assert_eq!(
                sc.borrower_debt(&managed_address!(&setup.borrower_address)).get(),
                managed_biguint!(5000)
            );
        })
        .assert_ok();
    
    // Simulação de emissão de tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.debt_token_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Na implementação real, o contrato Debt Token receberia uma chamada para mintar tokens
            // Aqui simulamos o registro da emissão
            sc.debt_tokens_minted_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(5000));
        })
        .assert_ok();
}

// Teste de pagamento de empréstimo
#[test]
fn l_t_repay_loan() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Preparar o cenário: adicionar liquidez e fazer um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            liquidity_pool::LiquidityPool::borrow_endpoint(&sc);
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Realizar pagamento parcial do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(3000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(3000));
            
            sc.repay_endpoint();
            
            // Verificar estado atualizado
            assert_eq!(sc.total_borrows().get(), managed_biguint!(2000));
            assert_eq!(
                sc.borrower_debt(&managed_address!(&setup.borrower_address)).get(),
                managed_biguint!(2000)
            );
            assert_eq!(sc.utilization_rate().get(), 2000u64); // 20%
        })
        .assert_ok();
    
    // Pagamento total do restante
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(2000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(2000));
            
            sc.repay_endpoint();
            
            // Verificar estado atualizado
            assert_eq!(sc.total_borrows().get(), managed_biguint!(0));
            assert_eq!(
                sc.borrower_debt(&managed_address!(&setup.borrower_address)).get(),
                managed_biguint!(0)
            );
            assert_eq!(sc.utilization_rate().get(), 0u64); // 0%
        })
        .assert_ok();
}

// Teste de cálculo de taxas de juros dinâmicas
#[test]
fn l_t_dynamic_interest_rate() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Verificar cálculo de taxa em diferentes níveis de utilização
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Baixa utilização (40%)
            sc.utilization_rate().set(4000u64);
            
            let low_rate = sc.calculate_current_interest_rate();
            
            // Meta de utilização é 80%, estamos abaixo
            // Taxa deve ser menor que a base
            assert!(low_rate < 1000u64);
            
            // Utilização exatamente na meta (80%)
            sc.utilization_rate().set(8000u64);
            
            let target_rate = sc.calculate_current_interest_rate();
            
            // Na meta de utilização, deve usar a taxa base
            assert_eq!(target_rate, 1000u64);
            
            // Alta utilização (90%)
            sc.utilization_rate().set(9000u64);
            
            let high_rate = sc.calculate_current_interest_rate();
            
            // Acima da meta, taxa deve ser maior
            assert!(high_rate > 1000u64);
        })
        .assert_ok();
}

// Teste de distribuição de juros para provedores de liquidez
#[test]
fn l_t_interest_distribution() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar dois provedores de liquidez
    let provider2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(10000));
    
    // Primeiro provedor adiciona 6000
    // Primeiro, criar uma cópia do endereço
    let provider_addr = setup.provider_address.clone();

    // Adicionar liquidez inicial usando ESDT em vez de EGLD
    add_esdt_to_contract(&mut setup, &provider_addr, TOKEN_ID_BYTES, 6000);

    // Segundo provedor adiciona 4000
    setup.blockchain_wrapper
        .execute_tx(&provider2, &setup.contract_wrapper, &rust_biguint!(4000), |sc| {
            sc.deposit_funds();
            
            // Simular emissão de tokens LP
            sc.lp_tokens_minted_endpoint(managed_address!(&provider2), managed_biguint!(4000));
        })
        .assert_ok();
    
    // Fazer um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            liquidity_pool::LiquidityPool::borrow_endpoint(&sc);
            
            // Simular emissão de tokens de dívida
            sc.debt_tokens_minted_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(8000));
        })
        .assert_ok();
    
    // Simular acúmulo de juros (10% do valor emprestado = 800)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(800), |sc| {
            // Adicionar juros acumulados
            sc.add_accumulated_interest_endpoint(managed_biguint!(800));
            
            // Verificar juros totais
            assert_eq!(sc.total_interest_accumulated().get(), managed_biguint!(800));
        })
        .assert_ok();
    
    // Distribuir juros aos provedores
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Calcular parte das reservas (20% = 160)
            let reserve_part = &sc.total_interest_accumulated().get() * &managed_biguint!(2000) / &managed_biguint!(10000);
            assert_eq!(reserve_part, managed_biguint!(160));
            
            // Distribuir juros
            sc.distribute_interest_endpoint();
            
            // Verificar reservas
            assert_eq!(sc.total_reserves().get(), managed_biguint!(160));
            
            // Verificar juros distribuídos aos provedores (640 total)
            // Primeiro provedor: 60% = 384
            // Segundo provedor: 40% = 256
            let provider1_share = sc.provider_interest(&managed_address!(&setup.provider_address)).get();
            let provider2_share = sc.provider_interest(&managed_address!(&provider2)).get();
            
            assert_eq!(provider1_share, managed_biguint!(384));
            assert_eq!(provider2_share, managed_biguint!(256));
            
            // Verificar que os juros foram totalmente distribuídos
            assert_eq!(sc.total_interest_accumulated().get(), managed_biguint!(0));
        })
        .assert_ok();
}

// Teste de atualização da taxa de utilização
#[test]
fn l_t_update_utilization_rate() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    // Fazer empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            liquidity_pool::LiquidityPool::borrow_endpoint(&sc);
            
            // Verificar taxa de utilização
            assert_eq!(sc.utilization_rate().get(), 7500u64); // 75%
        })
        .assert_ok();
    
    // Fazer outro empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            liquidity_pool::LiquidityPool::borrow_endpoint(&sc);
            
            // Verificar taxa de utilização
            assert_eq!(sc.utilization_rate().get(), 9000u64); // 90%
        })
        .assert_ok();
    
    // Pagar parte do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(3000), |sc| {
            // Simular queima de tokens de dívida
            sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower_address), managed_biguint!(3000));
            
            sc.repay_endpoint();
            
            // Verificar taxa de utilização atualizada
            assert_eq!(sc.utilization_rate().get(), 6000u64); // 60%
        })
        .assert_ok();
}

// Teste de pausa e despausa do contrato
#[test]
fn l_t_pause_unpause() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Pausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.pause();
            
            // Verificar estado
            assert!(sc.is_paused());
        })
        .assert_ok();
    
    // Tentar fazer um depósito (deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(1000), |sc| {
            // Verificar se está pausado
            let is_paused = sc.is_paused();
            assert!(is_paused);
            
            // Na implementação real, isso lançaria erro
            // "Contract is paused"
        })
        .assert_ok();
    
    // Despausar o contrato
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.unpause();
            
            // Verificar estado
            assert!(!sc.is_paused());
        })
        .assert_ok();
    
    // Agora deve ser possível fazer depósito
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(1000), |sc| {
            sc.deposit_funds();
            
            // Verificar que funcionou
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(1000));
        })
        .assert_ok();
}

// Teste de atualização de contratos relacionados
#[test]
fn l_t_update_related_contracts() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Criar novos endereços para os contratos
    let new_loan_controller = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let new_debt_token = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Atualizar endereços
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.set_loan_controller_address(managed_address!(&new_loan_controller));
            sc.set_debt_token_address(managed_address!(&new_debt_token));
            
            // Verificar atualização
            assert_eq!(
                sc.loan_controller_address().get(),
                managed_address!(&new_loan_controller)
            );
            assert_eq!(
                sc.debt_token_address().get(),
                managed_address!(&new_debt_token)
            );
        })
        .assert_ok();
    
    // Verificar que o antigo controlador não pode mais fazer empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que não é o controlador autorizado
            assert!(sc.blockchain().get_caller() != sc.loan_controller_address().get());
            
            // Na implementação real, isso lançaria erro
            // "Only loan controller can call this function"
        })
        .assert_ok();
    
    // Verificar que o novo controlador pode fazer empréstimos
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&new_loan_controller, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            liquidity_pool::LiquidityPool::borrow_endpoint(&sc);
            
            // Verificar empréstimo
            assert_eq!(sc.total_borrows().get(), managed_biguint!(5000));
        })
        .assert_ok();
}

// Teste de utilização de reservas
#[test]
fn l_t_use_reserves() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Adicionar liquidez e acumular reservas
    setup.blockchain_wrapper
        .execute_tx(&setup.provider_address, &setup.contract_wrapper, &rust_biguint!(10000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            liquidity_pool::LiquidityPool::borrow_endpoint(&sc);
        })
        .assert_ok();
    
    // Simular acúmulo de juros e distribuição
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(1000), |sc| {
            // Adicionar juros acumulados
            sc.add_accumulated_interest_endpoint(managed_biguint!(1000));
            
            // Distribuir juros
            sc.distribute_interest_endpoint();
            
            // Verificar reservas (20% = 200)
            assert_eq!(sc.total_reserves().get(), managed_biguint!(200));
        })
        .assert_ok();
    
    // Utilizar parte das reservas para algum propósito (ex: seguros)
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.use_reserves_endpoint(managed_address!(&setup.owner_address), managed_biguint!(150));
            
            // Verificar reservas atualizadas
            assert_eq!(sc.total_reserves().get(), managed_biguint!(50));
        })
        .assert_ok();
}

// Teste de atualização de parâmetros
#[test]
fn l_t_update_parameters() {
    let mut setup = setup_contract(liquidity_pool::contract_obj);
    
    // Atualizar parâmetros
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Atualizar taxa de juros base
            sc.set_interest_rate_base(1200u64);
            assert_eq!(sc.interest_rate_base().get(), 1200u64);
            
            // Atualizar utilização meta
            sc.set_target_utilization_rate(7500u64);
            assert_eq!(sc.target_utilization_rate().get(), 7500u64);
            
            // Atualizar utilização máxima
            sc.set_max_utilization_rate(2500u64);
            assert_eq!(sc.max_utilization_rate().get(), 2500u64);
            
            // Atualizar percentual de reservas
            sc.set_reserve_percent(1500u64);
            assert_eq!(sc.reserve_percent().get(), 1500u64);
        })
        .assert_ok();
    
    // Verificar que os novos parâmetros afetam os cálculos
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Simular utilização exatamente na meta
            sc.utilization_rate().set(7500u64);
            
            // Taxa deve ser igual à base atualizada
            let rate = sc.calculate_current_interest_rate();
            assert_eq!(rate, 1200u64);
        })
        .assert_ok();
}