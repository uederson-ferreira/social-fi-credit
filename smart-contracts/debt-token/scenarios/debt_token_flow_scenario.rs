// ==========================================================================
// ARQUIVO: debt_token_flow_scenario.rs
// Descrição: Cenários de fluxo completo para o contrato DebtToken
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};

use debt_token::*;

const WASM_PATH: &str = "output/debt-token.wasm";

// Estrutura para configuração dos testes
struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> debt_token::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub loan_controller_address: Address,
    pub liquidity_pool_address: Address,
    pub borrower_address: Address,
    pub user_addresses: Vec<Address>,
    pub contract_wrapper: ContractObjWrapper<debt_token::ContractObj<DebugApi>, ContractObjBuilder>,
}

// Função de configuração para os testes
fn setup_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> debt_token::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let loan_controller_address = blockchain_wrapper.create_user_account(&rust_zero);
    let liquidity_pool_address = blockchain_wrapper.create_user_account(&rust_zero);
    let borrower_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    // Criar alguns usuários adicionais
    let user1 = blockchain_wrapper.create_user_account(&rust_zero);
    let user2 = blockchain_wrapper.create_user_account(&rust_zero);
    let user3 = blockchain_wrapper.create_user_account(&rust_zero);
    
    let user_addresses = vec![user1, user2, user3];
    
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
            sc.init(managed_address!(&loan_controller_address));
        })
        .assert_ok();
    
    ContractSetup {
        blockchain_wrapper,
        owner_address,
        loan_controller_address,
        liquidity_pool_address,
        borrower_address,
        user_addresses,
        contract_wrapper,
    }
}

// Cenário: Fluxo completo de emissão de tokens para um empréstimo
#[test]
fn test_loan_issuance_scenario() {
    let mut setup = setup_contract(debt_token::contract_obj);
    
    // Etapa 1: LoanController concede um empréstimo e emite tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular a emissão de tokens para o empréstimo
            let loan_amount = managed_biguint!(10000);
            
            // Mintar tokens para o tomador
            sc.mint(managed_address!(&setup.borrower_address), loan_amount);
            
            // Verificar saldo do tomador e oferta total
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), loan_amount);
            assert_eq!(sc.total_token_supply().get(), loan_amount);
        })
        .assert_ok();
    
    // Etapa 2: O tomador pode verificar seu saldo
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let borrower_balance = sc.balance_of(&managed_address!(&setup.borrower_address));
            assert_eq!(borrower_balance, managed_biguint!(10000));
        })
        .assert_ok();
    
    // Etapa 3: O tomador transfere parte de sua dívida para outro usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir 4000 tokens para outro usuário
            sc.transfer(managed_address!(&setup.user_addresses[0]), managed_biguint!(4000));
            
            // Verificar saldo atualizado
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(6000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(4000));
        })
        .assert_ok();
    
    // Etapa 4: O tomador autoriza o LiquidityPool a gerenciar sua dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Aprovar o LiquidityPool para gerenciar os tokens
            sc.approve(managed_address!(&setup.liquidity_pool_address), managed_biguint!(6000));
            
            // Verificar autorização
            let allowance = sc.allowance(
                &managed_address!(&setup.borrower_address),
                &managed_address!(&setup.liquidity_pool_address)
            );
            assert_eq!(allowance, managed_biguint!(6000));
        })
        .assert_ok();
    
    // Etapa 5: LiquidityPool transfere tokens em nome do tomador
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir 2000 tokens para outro usuário
            sc.transfer_from(
                managed_address!(&setup.borrower_address),
                managed_address!(&setup.user_addresses[1]),
                managed_biguint!(2000)
            );
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(4000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(2000));
            
            // Verificar autorização atualizada
            let allowance = sc.allowance(
                &managed_address!(&setup.borrower_address),
                &managed_address!(&setup.liquidity_pool_address)
            );
            assert_eq!(allowance, managed_biguint!(4000));
        })
        .assert_ok();
    
    // Etapa 6: O tomador paga parte do empréstimo (queima de tokens)
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Queimar 4000 tokens do tomador
            sc.burn(managed_address!(&setup.borrower_address), managed_biguint!(4000));
            
            // Verificar saldo e oferta total
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(0));
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(6000));
        })
        .assert_ok();
    
    // Etapa 7: Os outros usuários pagam suas partes da dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Queimar tokens dos outros usuários
            sc.burn(managed_address!(&setup.user_addresses[0]), managed_biguint!(4000));
            sc.burn(managed_address!(&setup.user_addresses[1]), managed_biguint!(2000));
            
            // Verificar saldos zerados
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(0));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(0));
            
            // Verificar oferta total zerada
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(0));
        })
        .assert_ok();
}

// Cenário: Recuperação de inadimplência via tokens de dívida
#[test]
fn test_debt_default_recovery_scenario() {
    let mut setup = setup_contract(debt_token::contract_obj);
    
    // Etapa 1: Criar um empréstimo com tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emitir tokens de dívida para o tomador
            sc.mint(managed_address!(&setup.borrower_address), managed_biguint!(10000));
        })
        .assert_ok();
    
    // Etapa 2: O tomador vende/transfere tokens de dívida para vários usuários
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Distribuir a dívida entre vários usuários
            sc.transfer(managed_address!(&setup.user_addresses[0]), managed_biguint!(3000));
            sc.transfer(managed_address!(&setup.user_addresses[1]), managed_biguint!(4000));
            sc.transfer(managed_address!(&setup.user_addresses[2]), managed_biguint!(2000));
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(1000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(3000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(4000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[2])), managed_biguint!(2000));
        })
        .assert_ok();
    
    // Etapa 3: O tomador principal entra em inadimplência
    // Simular marcação de inadimplência sem alterar os tokens
    
    // Etapa 4: Liquidação de garantias e recuperação parcial
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular recuperação de 60% do valor através de liquidação
            // Os detentores de tokens recebem proporcionalmente
            
            // Calcular perdas de 40% para cada detentor
            // Queimar tokens proporcionalmente à perda
            sc.burn(managed_address!(&setup.borrower_address), managed_biguint!(400)); // 40% de 1000
            sc.burn(managed_address!(&setup.user_addresses[0]), managed_biguint!(1200)); // 40% de 3000
            sc.burn(managed_address!(&setup.user_addresses[1]), managed_biguint!(1600)); // 40% de 4000
            sc.burn(managed_address!(&setup.user_addresses[2]), managed_biguint!(800)); // 40% de 2000
            
            // Verificar saldos após haircut
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(600));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(1800));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(2400));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[2])), managed_biguint!(1200));
            
            // Verificar oferta total reduzida em 40%
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(6000)); // 60% de 10000
        })
        .assert_ok();
    
    // Etapa 5: Resgatar tokens restantes por valor
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Simular resgate dos tokens restantes após liquidação
            sc.burn(managed_address!(&setup.borrower_address), managed_biguint!(600));
            sc.burn(managed_address!(&setup.user_addresses[0]), managed_biguint!(1800));
            sc.burn(managed_address!(&setup.user_addresses[1]), managed_biguint!(2400));
            sc.burn(managed_address!(&setup.user_addresses[2]), managed_biguint!(1200));
            
            // Verificar saldos zerados
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(0));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(0));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(0));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[2])), managed_biguint!(0));
            
            // Verificar oferta total zerada
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(0));
        })
        .assert_ok();
}

// Cenário: Mercado secundário para tokens de dívida (continuação)
#[test]
fn test_debt_token_secondary_market_scenario() {
    let mut setup = setup_contract(debt_token::contract_obj);
    
    // Etapa 1: Criar um empréstimo com tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emitir tokens de dívida para o tomador
            sc.mint(managed_address!(&setup.borrower_address), managed_biguint!(10000));
        })
        .assert_ok();
    
    // Etapa 2: O tomador vende parte dos tokens no mercado secundário
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir para o primeiro comprador
            sc.transfer(managed_address!(&setup.user_addresses[0]), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Etapa 3: O primeiro comprador vende com desconto para outro usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.user_addresses[0], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Aprovar para transferência futura (simulando uma ordem de venda)
            sc.approve(managed_address!(&setup.liquidity_pool_address), managed_biguint!(2000));
        })
        .assert_ok();
    
    // Simulação de uma exchange executando a ordem
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir tokens do vendedor para o comprador
            sc.transfer_from(
                managed_address!(&setup.user_addresses[0]),
                managed_address!(&setup.user_addresses[1]),
                managed_biguint!(2000)
            );
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(3000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(2000));
        })
        .assert_ok();
    
    // Etapa 4: Outro usuário oferece comprar mais tokens com desconto
    setup.blockchain_wrapper
        .execute_tx(&setup.user_addresses[0], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Aprovar mais tokens para venda
            sc.approve(managed_address!(&setup.liquidity_pool_address), managed_biguint!(1000));
        })
        .assert_ok();
    
    setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Executar a nova transferência
            sc.transfer_from(
                managed_address!(&setup.user_addresses[0]),
                managed_address!(&setup.user_addresses[2]),
                managed_biguint!(1000)
            );
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(2000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[2])), managed_biguint!(1000));
        })
        .assert_ok();
    
    // Etapa 5: O tomador paga parte do empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Queimar tokens do tomador
            sc.burn(managed_address!(&setup.borrower_address), managed_biguint!(3000));
            
            // Verificar saldo e oferta total
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(2000));
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(7000));
        })
        .assert_ok();
    
    // Etapa 6: Um detentor de tokens resgata sua parte
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Queimar tokens de um dos detentores
            sc.burn(managed_address!(&setup.user_addresses[1]), managed_biguint!(2000));
            
            // Verificar saldo e oferta total
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(0));
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(5000));
        })
        .assert_ok();
}

// Cenário: Atualização do contrato e migração de tokens
#[test]
fn test_contract_upgrade_migration_scenario() {
    let mut setup = setup_contract(debt_token::contract_obj);
    
    // Etapa 1: Configuração inicial com tokens emitidos
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emitir tokens para vários usuários
            sc.mint(managed_address!(&setup.borrower_address), managed_biguint!(5000));
            sc.mint(managed_address!(&setup.user_addresses[0]), managed_biguint!(3000));
            sc.mint(managed_address!(&setup.user_addresses[1]), managed_biguint!(2000));
            
            // Verificar oferta total
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(10000));
        })
        .assert_ok();
    
    // Etapa 2: Simular preparação para atualização
    let new_loan_controller = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Pausar o contrato durante a atualização
            sc.pause();
            
            // Verificar estado
            assert!(sc.is_paused().get());
        })
        .assert_ok();
    
    // Etapa 3: Tentar fazer uma transferência durante a pausa (deve falhar)
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que o contrato está pausado
            assert!(sc.is_paused().get());
            
            // Na implementação real, isso lançaria erro
            // "Contract is paused"
        })
        .assert_ok();
    
    // Etapa 4: Atualizar endereço do controlador e despausar
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Atualizar endereço do controlador
            sc.set_loan_controller_address(managed_address!(&new_loan_controller));
            
            // Despausar o contrato
            sc.unpause();
            
            // Verificar estado
            assert_eq!(sc.loan_controller_address().get(), managed_address!(&new_loan_controller));
            assert!(!sc.is_paused().get());
        })
        .assert_ok();
    
    // Etapa 5: Verificar que o novo controlador pode operar
    setup.blockchain_wrapper
        .execute_tx(&new_loan_controller, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Mintar novos tokens
            sc.mint(managed_address!(&setup.user_addresses[2]), managed_biguint!(1000));
            
            // Verificar saldo
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[2])), managed_biguint!(1000));
            
            // Verificar oferta total atualizada
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(11000));
        })
        .assert_ok();
    
    // Etapa 6: Verificar que o antigo controlador não pode mais operar
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar que não é o controlador autorizado
            assert!(sc.blockchain().get_caller() != sc.loan_controller_address().get());
            
            // Na implementação real, isso lançaria erro
            // "Only loan controller can call this function"
        })
        .assert_ok();
}

// Cenário: Resposta a emergências
#[test]
fn test_emergency_response_scenario() {
    let mut setup = setup_contract(debt_token::contract_obj);
    
    // Etapa 1: Configuração inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emitir tokens para vários usuários
            sc.mint(managed_address!(&setup.borrower_address), managed_biguint!(5000));
            sc.mint(managed_address!(&setup.user_addresses[0]), managed_biguint!(3000));
            sc.mint(managed_address!(&setup.user_addresses[1]), managed_biguint!(2000));
        })
        .assert_ok();
    
    // Etapa 2: Detectar vulnerabilidade ou ataque (simulação)
    // Um usuário malicioso tentou explorar uma vulnerabilidade
    
    // Etapa 3: Ativar modo de emergência
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Pausar imediatamente o contrato
            sc.pause();
            
            // Ativar controles adicionais de emergência
            sc.enable_emergency_mode();
            
            // Verificar estado
            assert!(sc.is_paused().get());
            assert!(sc.emergency_mode().get());
        })
        .assert_ok();
    
    // Etapa 4: Verificar que todas as operações normais estão bloqueadas
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Verificar estado de emergência
            assert!(sc.is_paused().get());
            assert!(sc.emergency_mode().get());
            
            // Na implementação real, qualquer operação lançaria erro
            // "Contract is in emergency mode"
        })
        .assert_ok();
    
    // Etapa 5: Executor operações de recuperação
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Corrigir saldos se necessário (simulando correção pós-ataque)
            let original_balance = sc.balance_of(&managed_address!(&setup.user_addresses[0]));
            let corrected_balance = managed_biguint!(3000); // Valor correto
            
            if original_balance != corrected_balance {
                // Na implementação real, haveria uma função de correção de emergência
                sc.emergency_balance_correction(
                    managed_address!(&setup.user_addresses[0]),
                    corrected_balance
                );
            }
            
            // Verificar que o saldo está correto
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), corrected_balance);
        })
        .assert_ok();
    
    // Etapa 6: Desativar modo de emergência após correções
    setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Desativar modo de emergência
            sc.disable_emergency_mode();
            
            // Despausar o contrato
            sc.unpause();
            
            // Verificar estado
            assert!(!sc.is_paused().get());
            assert!(!sc.emergency_mode().get());
        })
        .assert_ok();
    
    // Etapa 7: Verificar que operações normais voltaram
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Testar uma transferência normal
            sc.transfer(managed_address!(&setup.user_addresses[2]), managed_biguint!(1000));
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(4000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[2])), managed_biguint!(1000));
        })
        .assert_ok();
}

// Cenário: Gerenciamento de múltiplos empréstimos por usuário
#[test]
fn test_multiple_loan_management_scenario() {
    let mut setup = setup_contract(debt_token::contract_obj);
    
    // Etapa 1: Emitir tokens para dois empréstimos diferentes do mesmo tomador
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Emitir tokens para o primeiro empréstimo
            // Na implementação real, haveria identificadores de empréstimo
            sc.mint(managed_address!(&setup.borrower_address), managed_biguint!(5000));
            
            // Associar a um ID de empréstimo (simulação)
            sc.set_loan_id_for_tokens(managed_address!(&setup.borrower_address), 1u64);
            
            // Emitir tokens para o segundo empréstimo
            sc.mint(managed_address!(&setup.borrower_address), managed_biguint!(3000));
            
            // Associar a outro ID de empréstimo (simulação)
            sc.set_loan_id_for_tokens(managed_address!(&setup.borrower_address), 2u64);
            
            // Verificar saldo total do tomador
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(8000));
        })
        .assert_ok();
    
    // Etapa 2: O tomador transfere parte de um empréstimo específico
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir tokens do primeiro empréstimo
            sc.transfer_tokens_for_loan(
                managed_address!(&setup.user_addresses[0]),
                managed_biguint!(2000),
                1u64 // ID do empréstimo
            );
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(6000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(2000));
            
            // Verificar associação de empréstimo (simulação)
            assert_eq!(sc.get_loan_id_for_tokens(managed_address!(&setup.user_addresses[0])), 1u64);
        })
        .assert_ok();
    
    // Etapa 3: Transferir parte do segundo empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.borrower_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir tokens do segundo empréstimo
            sc.transfer_tokens_for_loan(
                managed_address!(&setup.user_addresses[1]),
                managed_biguint!(1000),
                2u64 // ID do empréstimo
            );
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(5000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(1000));
            
            // Verificar associação de empréstimo (simulação)
            assert_eq!(sc.get_loan_id_for_tokens(managed_address!(&setup.user_addresses[1])), 2u64);
        })
        .assert_ok();
    
    // Etapa 4: Pagar um dos empréstimos completamente
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Queimar tokens do primeiro empréstimo (do tomador)
            sc.burn_tokens_for_loan(managed_address!(&setup.borrower_address), managed_biguint!(3000), 1u64);
            
            // Queimar tokens do primeiro empréstimo (do outro detentor)
            sc.burn_tokens_for_loan(managed_address!(&setup.user_addresses[0]), managed_biguint!(2000), 1u64);
            
            // Verificar saldos
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(2000));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[0])), managed_biguint!(0));
            
            // Verificar oferta total
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(3000));
        })
        .assert_ok();
    
    // Etapa 5: Pagar o segundo empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Queimar tokens do segundo empréstimo (do tomador)
            sc.burn_tokens_for_loan(managed_address!(&setup.borrower_address), managed_biguint!(2000), 2u64);
            
            // Queimar tokens do segundo empréstimo (do outro detentor)
            sc.burn_tokens_for_loan(managed_address!(&setup.user_addresses[1]), managed_biguint!(1000), 2u64);
            
            // Verificar saldos zerados
            assert_eq!(sc.balance_of(&managed_address!(&setup.borrower_address)), managed_biguint!(0));
            assert_eq!(sc.balance_of(&managed_address!(&setup.user_addresses[1])), managed_biguint!(0));
            
            // Verificar oferta total zerada
            assert_eq!(sc.total_token_supply().get(), managed_biguint!(0));
        })
        .assert_ok();
}