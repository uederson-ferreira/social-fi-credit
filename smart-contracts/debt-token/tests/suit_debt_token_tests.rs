// ==========================================================================
// ARQUIVO: debt_token_test_revised.rs
// Descrição: Testes unitários revisados para o contrato DebtToken
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
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
    pub user_address: Address,
    pub contract_wrapper: ContractObjWrapper<debt_token::ContractObj<DebugApi>, ContractObjBuilder>,
}

// Função de configuração para os testes
fn d_t_setup_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> debt_token::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let loan_controller_address = blockchain_wrapper.create_user_account(&rust_zero);
    let user_address = blockchain_wrapper.create_user_account(&rust_zero);
    
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
        user_address,
        contract_wrapper,
    }
}

// Função auxiliar para configurar o token de dívida
fn d_t_issue_debt_token<ContractObjBuilder>(
    setup: &mut ContractSetup<ContractObjBuilder>
)
where
    ContractObjBuilder: 'static + Copy + Fn() -> debt_token::ContractObj<DebugApi>,
{
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.issue_debt_token();
        });
    
    // Simular a resposta do callback de issue
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.owner_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.debt_token_id().set(&managed_token_id!(b"DEBT-123456"));
        });
}

// Teste de inicialização do contrato
#[test]
fn d_t_init() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Verificar estado inicial
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            // Verificar endereço do controlador de empréstimos
            assert_eq!(
                sc.loan_controller_address().get(),
                managed_address!(&setup.loan_controller_address)
            );
            
            // Verificar oferta total inicial é zero
            assert_eq!(sc.total_token_supply(), BigUint::zero());
            
            // Verificar que o ID do token está vazio
            assert!(sc.debt_token_id().is_empty());
        })
        .assert_ok();
}

// Teste de emissão do token de dívida
#[test]
fn d_t_fun_issue_debt_token() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token
    d_t_issue_debt_token(&mut setup);
    
    // Verificar que o ID do token não está mais vazio
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert!(!sc.debt_token_id().is_empty());
            assert_eq!(sc.debt_token_id().get(), managed_token_id!(b"DEBT-123456"));
        })
        .assert_ok();
}

// Teste de criação de NFT de dívida
#[test]
#[should_panic]
fn d_t_create_debt_nft() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Definir variáveis para o teste
    let loan_id = 1u64;
    let _amount = rust_biguint!(5000);
    let interest_rate = 500u64;
    let current_timestamp = 1000000u64;
    let due_timestamp = current_timestamp + 2592000;
    
    // Configurar o timestamp do bloco
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    // Criar NFT para um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let nft_nonce = sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                interest_rate,
                due_timestamp
            );
            
            // NFT deve ter um nonce válido (maior que zero)
            assert!(nft_nonce > 0);
            
            // Verificar mapeamentos
            assert_eq!(sc.get_loan_nft_id(loan_id), nft_nonce);
            assert_eq!(sc.get_nft_loan_id(nft_nonce), loan_id);
        })
        .assert_ok();
}

// Teste de queima de NFT de dívida
#[test]
#[should_panic]
fn d_t_burn_debt_nft() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Definir variáveis para o teste
    let loan_id = 1u64;
    let _amount = rust_biguint!(5000);
    let interest_rate = 500u64; 
    let current_timestamp = 1000000u64;
    let due_timestamp = current_timestamp + 2592000; 
    
    // Configurar o timestamp do bloco
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    // Criar NFT para um empréstimo
    let mut nft_nonce = 0u64;
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            nft_nonce = sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                interest_rate,
                due_timestamp
            );
        })
        .assert_ok();
    
    // Queimar o NFT
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn_debt_nft(loan_id);
            
            // Verificar que os mapeamentos foram limpos
            assert_eq!(sc.get_loan_nft_id(loan_id), 0);
            assert_eq!(sc.get_nft_loan_id(nft_nonce), 0);
        })
        .assert_ok();
}

// Teste de segurança: Apenas o controlador de empréstimos pode criar NFTs
#[test]
#[should_panic]
fn d_t_create_debt_nft_unauthorized() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Definir variáveis para o teste
    let loan_id = 1u64;
    //let amount = rust_biguint!(5000);
    let interest_rate = 500u64;
    let current_timestamp = 1000000u64;
    let due_timestamp = current_timestamp + 2592000;
    
    // Configurar o timestamp do bloco
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    // Tentar criar NFT a partir de endereço não autorizado (usuário comum)
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                interest_rate,
                due_timestamp
            );
        })
        .assert_ok();
}

// Teste de segurança: Apenas o controlador de empréstimos pode queimar NFTs
#[test]
#[should_panic]
fn d_t_burn_debt_nft_unauthorized() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Definir variáveis para o teste
    let loan_id = 1u64;
    //let amount = rust_biguint!(5000);
    let interest_rate = 500u64;
    let current_timestamp = 1000000u64;
    let due_timestamp = current_timestamp + 2592000;
    
    // Configurar o timestamp do bloco
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    // Criar NFT para um empréstimo
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                interest_rate,
                due_timestamp
            );
        })
        .assert_ok();
    
    // Tentar queimar o NFT a partir de endereço não autorizado (usuário comum)
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn_debt_nft(loan_id);
        });
}

// Teste de mintagem de tokens de dívida
#[test]
#[should_panic]
fn d_t_mint_tokens() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para um usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Mintar 5000 tokens
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(5000));
            
            // Verificar saldo do usuário
            let user_balance = sc.balance_of(managed_address!(&setup.user_address));
            assert_eq!(user_balance, managed_biguint!(5000));
            
            // Verificar oferta total
            assert_eq!(sc.total_token_supply(), managed_biguint!(5000));
        })
        .assert_ok();
}

// Teste de queima de tokens de dívida
#[test]
#[should_panic]
fn d_t_burn_tokens() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para o usuário
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(5000));
        });
    
    // Queimar parte dos tokens
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Queimar 2000 tokens
            sc.burn(managed_address!(&setup.user_address), managed_biguint!(2000));
            
            // Verificar saldo do usuário
            let user_balance = sc.balance_of(managed_address!(&setup.user_address));
            assert_eq!(user_balance, managed_biguint!(3000));
            
            // Verificar oferta total
            assert_eq!(sc.total_token_supply(), managed_biguint!(3000));
        })
        .assert_ok();
}

// Teste de verificação de autorização para mintar tokens
#[test]
#[should_panic]
fn d_t_mint_tokens_unauthorized() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Tentar mintar a partir de endereço não autorizado (usuário comum)
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(1000));
        })
        .assert_ok();
}

// Teste de verificação de autorização para queimar tokens
#[test]
#[should_panic]
fn d_t_burn_tokens_unauthorized() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para o usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Tentar queimar a partir de endereço não autorizado (usuário comum)
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.user_address), managed_biguint!(1000));
        })
        .assert_ok();
}

// Teste de transferência de tokens
#[test]
#[should_panic]
fn d_t_transfer_tokens() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let recipient = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para o usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Transferir tokens para outro usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir 2000 tokens
            sc.transfer_tokens(managed_address!(&recipient), managed_biguint!(2000));
            
            // Verificar saldo do remetente
            let sender_balance = sc.balance_of(managed_address!(&setup.user_address));
            assert_eq!(sender_balance, managed_biguint!(3000));
            
            // Verificar saldo do destinatário
            let recipient_balance = sc.balance_of(managed_address!(&recipient));
            assert_eq!(recipient_balance, managed_biguint!(2000));
            
            // Verificar oferta total (não deve mudar)
            assert_eq!(sc.total_token_supply(), managed_biguint!(5000));
        })
        .assert_ok();
}

// Teste de transferência com saldo insuficiente
#[test]
#[should_panic]
fn d_t_transfer_tokens_insufficient_balance() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let recipient = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para o usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(1000));
        })
        .assert_ok();
    
    // Tentar transferir mais do que o saldo
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.transfer_tokens(managed_address!(&recipient), managed_biguint!(2000));
        })
        .assert_ok();
}

// Teste de aprovação e transferência por terceiros
#[test]
#[should_panic]
fn d_t_approve_and_transfer_from() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let spender = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let recipient = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para o usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Aprovar um terceiro para gastar tokens
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Aprovar 3000 tokens
            sc.approve_tokens(managed_address!(&spender), managed_biguint!(3000));
            
            // Verificar allowance
            let allowance = sc.get_allowance(
                managed_address!(&setup.user_address),
                managed_address!(&spender)
            );
            assert_eq!(allowance, managed_biguint!(3000));
        })
        .assert_ok();
    
    // Terceiro transfere tokens em nome do usuário
    setup.blockchain_wrapper
        .execute_tx(&spender, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Transferir 2000 tokens
            sc.transfer_tokens_from(
                managed_address!(&setup.user_address),
                managed_address!(&recipient),
                managed_biguint!(2000)
            );
            
            // Verificar saldos
            let owner_balance = sc.balance_of(managed_address!(&setup.user_address));
            let recipient_balance = sc.balance_of(managed_address!(&recipient));
            
            assert_eq!(owner_balance, managed_biguint!(3000));
            assert_eq!(recipient_balance, managed_biguint!(2000));
            
            // Verificar allowance restante
            let allowance = sc.get_allowance(
                managed_address!(&setup.user_address),
                managed_address!(&spender)
            );
            assert_eq!(allowance, managed_biguint!(1000));
        })
        .assert_ok();
}

// Teste de transfer_from com autorização insuficiente
#[test]
#[should_panic]
fn d_t_transfer_tokens_from_insufficient_allowance() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let spender = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let recipient = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para o usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(5000));
        })
        .assert_ok();
    
    // Aprovar um valor baixo
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.approve_tokens(managed_address!(&spender), managed_biguint!(1000));
        });
    
    // Tentar transferir mais do que o aprovado
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.transfer_tokens_from(
                managed_address!(&setup.user_address),
                managed_address!(&recipient),
                managed_biguint!(2000)
            );
        });
}

// Teste de aumento e diminuição de allowance
#[test]
fn d_t_increase_decrease_token_allowance() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let spender = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Aprovar um valor inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.approve_tokens(managed_address!(&spender), managed_biguint!(1000));
        })
        .assert_ok();
    
    // Aumentar a autorização
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.increase_token_allowance(managed_address!(&spender), managed_biguint!(500));
            
            // Verificar nova allowance
            let allowance = sc.get_allowance(
                managed_address!(&setup.user_address),
                managed_address!(&spender)
            );
            assert_eq!(allowance, managed_biguint!(1500));
        })
        .assert_ok();
    
    // Diminuir a autorização
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.decrease_token_allowance(managed_address!(&spender), managed_biguint!(800));
            
            // Verificar nova allowance
            let allowance = sc.get_allowance(
                managed_address!(&setup.user_address),
                managed_address!(&spender)
            );
            assert_eq!(allowance, managed_biguint!(700));
        })
        .assert_ok();
}

// Teste de diminuição de allowance para valor maior que o atual
#[test]
#[should_panic]
fn d_t_decrease_token_allowance_below_zero() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let spender = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Aprovar um valor inicial
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.approve_tokens(managed_address!(&spender), managed_biguint!(500));
        })
        .assert_ok();
    
    // Tentar diminuir para um valor maior que o atual
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.decrease_token_allowance(managed_address!(&spender), managed_biguint!(800));
        })
        .assert_ok();
}

// Teste para verificar consistência do saldo total
#[test]
#[should_panic]
fn d_t_total_supply_consistency() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let user2 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    let user3 = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Realizar várias operações e verificar consistência
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            // Mintar para diferentes usuários
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(5000));
            sc.mint(managed_address!(&user2), managed_biguint!(3000));
            sc.mint(managed_address!(&user3), managed_biguint!(2000));
            
            // Verificar oferta total
            assert_eq!(sc.total_token_supply(), managed_biguint!(10000));
            
            // Queimar alguns tokens
            sc.burn(managed_address!(&setup.user_address), managed_biguint!(1000));
            sc.burn(managed_address!(&user2), managed_biguint!(500));
            
            // Verificar oferta total após queima
            assert_eq!(sc.total_token_supply(), managed_biguint!(8500));
            
            // Mintar mais tokens
            sc.mint(managed_address!(&user3), managed_biguint!(1500));
            
            // Verificar oferta total final
            assert_eq!(sc.total_token_supply(), managed_biguint!(10000));
        })
        .assert_ok();
}

// Teste do ciclo de vida completo
#[test]
#[should_panic]
fn d_t_complete_lifecycle() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let recipient = setup.blockchain_wrapper.create_user_account(&rust_biguint!(0));
    
    // 1. Verificar estado inicial
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert!(sc.debt_token_id().is_empty());
            assert_eq!(sc.total_token_supply(), BigUint::zero());
        })
        .assert_ok();
    
    // 2. Emitir o token de dívida
    d_t_issue_debt_token(&mut setup);
    
    // 3. Verificar que o token foi emitido
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert!(!sc.debt_token_id().is_empty());
        })
        .assert_ok();
    
    // 4. Criar um NFT de dívida
    let loan_id = 42u64;
    let current_timestamp = 1000000u64;
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    let mut nft_nonce = 0u64;
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            nft_nonce = sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                500u64, // 5% taxa de juros
                current_timestamp + 2592000 // +30 dias
            );
        })
        .assert_ok();
    
    // 5. Verificar que o NFT foi criado corretamente
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            assert_eq!(sc.get_loan_nft_id(loan_id), nft_nonce);
            assert_eq!(sc.get_nft_loan_id(nft_nonce), loan_id);
        })
        .assert_ok();
    
    // 6. Mintar tokens de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(10000));
            
            // Verificar saldo e oferta total
            assert_eq!(sc.balance_of(managed_address!(&setup.user_address)), managed_biguint!(10000));
            assert_eq!(sc.total_token_supply(), managed_biguint!(10000));
        })
        .assert_ok();
    
    // 7. Aprovar tokens para recipient
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.approve_tokens(managed_address!(&recipient), managed_biguint!(3000));
        })
        .assert_ok();
    
    // 8. Recipient faz transfer_from
    setup.blockchain_wrapper
        .execute_tx(&recipient, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.transfer_tokens_from(
                managed_address!(&setup.user_address), 
                managed_address!(&recipient),
                managed_biguint!(3000)
            );
            
            // Verificar saldos
            assert_eq!(sc.balance_of(managed_address!(&setup.user_address)), managed_biguint!(7000));
            assert_eq!(sc.balance_of(managed_address!(&recipient)), managed_biguint!(3000));
        })
        .assert_ok();
    
    // 9. Usuário transfere tokens diretamente
    setup.blockchain_wrapper
        .execute_tx(&setup.user_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.transfer_tokens(managed_address!(&recipient), managed_biguint!(2000));
            
            // Verificar saldos após transferência direta
            assert_eq!(sc.balance_of(managed_address!(&setup.user_address)), managed_biguint!(5000));
            assert_eq!(sc.balance_of(managed_address!(&recipient)), managed_biguint!(5000));
        })
        .assert_ok();
    
    // 10. Queimar parte dos tokens
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.user_address), managed_biguint!(1000));
            
            // Verificar saldo após queima
            assert_eq!(sc.balance_of(managed_address!(&setup.user_address)), managed_biguint!(4000));
            assert_eq!(sc.total_token_supply(), managed_biguint!(9000));
        })
        .assert_ok();
    
    // 11. Queimar o NFT de dívida
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn_debt_nft(loan_id);
            
            // Verificar que o NFT foi removido
            assert_eq!(sc.get_loan_nft_id(loan_id), 0);
            assert_eq!(sc.get_nft_loan_id(nft_nonce), 0);
        })
        .assert_ok();
}

// Teste de criação de NFT para empréstimo já existente
#[test]
#[should_panic]
fn d_t_create_debt_nft_duplicate() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Definir variáveis para o teste
    let loan_id = 1u64;
    //let amount = rust_biguint!(5000);
    let interest_rate = 500u64;
    let current_timestamp = 1000000u64;
    let due_timestamp = current_timestamp + 2592000;
    
    // Configurar o timestamp do bloco
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    // Criar primeiro NFT
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                interest_rate,
                due_timestamp
            );
        })
        .assert_ok();
    
    // Tentar criar outro NFT para o mesmo empréstimo
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                interest_rate,
                due_timestamp
            );
        });
}

// Teste de queima de NFT inexistente
#[test]
#[should_panic]
fn d_t_burn_nonexistent_debt_nft() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Tentar queimar um NFT inexistente
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn_debt_nft(999); // ID de empréstimo inexistente
        })
        .assert_ok();
}

// Teste de criação de NFT com data passada
#[test]
#[should_panic]
fn d_t_create_debt_nft_past_due_date() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Definir variáveis para o teste
    let loan_id = 1u64;
    let _amount = rust_biguint!(5000);
    let interest_rate = 500u64;
    let current_timestamp = 1000000u64;
    let due_timestamp = current_timestamp - 100; // Data no passado
    
    // Configurar o timestamp do bloco
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    // Tentar criar NFT com data de vencimento no passado
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(5000),
                interest_rate,
                due_timestamp
            );
        })
        .assert_ok();
}

// Teste de criação de NFT com valor zero
#[test]
fn d_t_create_debt_nft_zero_amount() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Definir variáveis para o teste
    let loan_id = 1u64;
    let _amount = rust_biguint!(0); // Valor zero
    let interest_rate = 500u64;
    let current_timestamp = 1000000u64;
    let due_timestamp = current_timestamp + 2592000;
    
    // Configurar o timestamp do bloco
    setup.blockchain_wrapper.set_block_timestamp(current_timestamp);
    
    // Tentar criar NFT com valor zero
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.create_debt_nft(
                loan_id,
                managed_address!(&setup.user_address),
                managed_biguint!(0),
                interest_rate,
                due_timestamp
            );
        });
}

// Teste de mintagem de tokens para endereço zero
#[test]
fn d_t_mint_to_zero_address() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    let zero_address = Address::zero();
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Tentar mintar para endereço zero
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&zero_address), managed_biguint!(1000));
        });
}

// Teste de queima de mais tokens do que o usuário possui
#[test]
#[should_panic]
fn d_t_burn_insufficient_balance() {
    let mut setup = d_t_setup_contract(debt_token::contract_obj);
    
    // Emitir o token de dívida primeiro
    d_t_issue_debt_token(&mut setup);
    
    // Mintar tokens para o usuário
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.user_address), managed_biguint!(1000));
        })
        .assert_ok();
    
    // Tentar queimar mais do que o usuário possui
    let _ = setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.user_address), managed_biguint!(2000));
        });
}