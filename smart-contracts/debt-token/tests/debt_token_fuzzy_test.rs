// ==========================================================================
// ARQUIVO: debt_token_fuzzy_test.rs
// Descrição: Testes fuzzy com entradas aleatórias para o contrato DebtToken
// ==========================================================================

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    DebugApi,
};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

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
    pub users: Vec<Address>,
    pub contract_wrapper: ContractObjWrapper<debt_token::ContractObj<DebugApi>, ContractObjBuilder>,
}

// Função de configuração para os testes
fn setup_fuzzy_contract<ContractObjBuilder>(
    builder: ContractObjBuilder,
    num_users: usize,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> debt_token::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let loan_controller_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    // Criar vários usuários para testes
    let mut users = Vec::with_capacity(num_users);
    for _ in 0..num_users {
        let user_address = blockchain_wrapper.create_user_account(&rust_zero);
        users.push(user_address);
    }
    
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
        users,
        contract_wrapper,
    }
}

// Função para gerar um endereço aleatório
fn generate_random_address(rng: &mut StdRng) -> Address {
    let mut address_bytes = [0u8; 32];
    rng.fill(&mut address_bytes);
    Address::from_slice(&address_bytes)
}

// Teste fuzzy para operações de mintagem e queima
#[test]
fn test_mint_burn_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(debt_token::contract_obj, 10);
    
    // Realizar várias operações aleatórias de mint e burn
    for _ in 0..100 {
        let user_idx = rng.gen_range(0..setup.users.len());
        let user_address = &setup.users[user_idx];
        
        // Gerar valor aleatório
        let amount = rng.gen_range(100..10000u64);
        
        // Decidir entre mint e burn
        let is_mint = rng.gen_bool(0.7); // 70% chance de mint
        
        if is_mint {
            // Mintar tokens
            setup.blockchain_wrapper
                .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    sc.mint(managed_address!(user_address), managed_biguint!(amount));
                })
                .assert_ok();
        } else {
            // Queimar tokens (apenas se o usuário tiver saldo suficiente)
            setup.blockchain_wrapper
                .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    let current_balance = sc.balance_of(&managed_address!(user_address));
                    
                    // Só queimar se tiver saldo suficiente
                    if current_balance >= managed_biguint!(amount) {
                        sc.burn(managed_address!(user_address), managed_biguint!(amount));
                    }
                })
                .assert_ok();
        }
    }
    
    // Verificar consistência da oferta total
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let total_supply = sc.total_token_supply().get();
            
            // Calcular a soma dos saldos de todos os usuários
            let mut sum_balances = managed_biguint!(0);
            for user in &setup.users {
                let balance = sc.balance_of(&managed_address!(user));
                sum_balances += balance;
            }
            
            // Verificar que a oferta total é igual à soma dos saldos
            assert_eq!(total_supply, sum_balances);
        })
        .assert_ok();
}

// Teste fuzzy para transferências entre usuários (continuação)
#[test]
fn test_transfers_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(debt_token::contract_obj, 10);
    
    // Primeiro, mintar tokens para todos os usuários
    for (i, user) in setup.users.iter().enumerate() {
        let amount = (i as u64 + 1) * 1000; // Diferentes quantidades para diferentes usuários
        
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.mint(managed_address!(user), managed_biguint!(amount));
            })
            .assert_ok();
    }
    
    // Guardar o total de oferta para verificação final
    let initial_total_supply = setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            sc.total_token_supply().get()
        })
        .unwrap_or_default();
    
    // Realizar várias transferências aleatórias
    for _ in 0..200 {
        let sender_idx = rng.gen_range(0..setup.users.len());
        let mut recipient_idx = rng.gen_range(0..setup.users.len());
        
        // Garantir que remetente e destinatário são diferentes
        while recipient_idx == sender_idx {
            recipient_idx = rng.gen_range(0..setup.users.len());
        }
        
        let sender = &setup.users[sender_idx];
        let recipient = &setup.users[recipient_idx];
        
        // Obter saldo do remetente
        let sender_balance = setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                sc.balance_of(&managed_address!(sender))
            })
            .unwrap_or_default();
        
        if sender_balance > BigUint::zero() {
            // Gerar valor aleatório para transferência (até o saldo disponível)
            let max_amount = sender_balance.to_u64().unwrap_or(0);
            if max_amount > 0 {
                let amount = rng.gen_range(1..=max_amount);
                
                // Executar a transferência
                setup.blockchain_wrapper
                    .execute_tx(sender, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        sc.transfer(managed_address!(recipient), managed_biguint!(amount));
                    })
                    .assert_ok();
            }
        }
    }
    
    // Verificar que a oferta total permanece inalterada
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let final_total_supply = sc.total_token_supply().get();
            assert_eq!(final_total_supply, initial_total_supply);
        })
        .assert_ok();
}

// Teste fuzzy para operações de approve e transferFrom
#[test]
fn test_approve_transfer_from_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(debt_token::contract_obj, 10);
    
    // Mintar tokens para todos os usuários
    for (i, user) in setup.users.iter().enumerate() {
        let amount = (i as u64 + 1) * 1000; // Diferentes quantidades para diferentes usuários
        
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.mint(managed_address!(user), managed_biguint!(amount));
            })
            .assert_ok();
    }
    
    // Realizar aprovações aleatórias
    for _ in 0..50 {
        let owner_idx = rng.gen_range(0..setup.users.len());
        let mut spender_idx = rng.gen_range(0..setup.users.len());
        
        // Garantir que owner e spender são diferentes
        while spender_idx == owner_idx {
            spender_idx = rng.gen_range(0..setup.users.len());
        }
        
        let owner = &setup.users[owner_idx];
        let spender = &setup.users[spender_idx];
        
        // Gerar valor aleatório para aprovação
        let amount = rng.gen_range(100..5000u64);
        
        // Executar a aprovação
        setup.blockchain_wrapper
            .execute_tx(owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                sc.approve(managed_address!(spender), managed_biguint!(amount));
            })
            .assert_ok();
    }
    
    // Realizar transferências usando as aprovações
    for _ in 0..100 {
        let owner_idx = rng.gen_range(0..setup.users.len());
        let spender_idx = rng.gen_range(0..setup.users.len());
        let mut recipient_idx = rng.gen_range(0..setup.users.len());
        
        // Garantir que recipient é diferente de owner e spender
        while recipient_idx == owner_idx || recipient_idx == spender_idx {
            recipient_idx = rng.gen_range(0..setup.users.len());
        }
        
        let owner = &setup.users[owner_idx];
        let spender = &setup.users[spender_idx];
        let recipient = &setup.users[recipient_idx];
        
        // Verificar allowance e saldo
        let (allowance, owner_balance) = setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let allowance = sc.allowance(
                    &managed_address!(owner),
                    &managed_address!(spender)
                );
                let owner_balance = sc.balance_of(&managed_address!(owner));
                (allowance, owner_balance)
            })
            .unwrap_or_default();
        
        if allowance > BigUint::zero() && owner_balance > BigUint::zero() {
            // Determinar o valor máximo que pode ser transferido
            let max_transfer = std::cmp::min(allowance.clone(), owner_balance.clone());
            let max_amount = max_transfer.to_u64().unwrap_or(0);
            
            if max_amount > 0 {
                // Gerar valor aleatório para transferência
                let amount = rng.gen_range(1..=max_amount);
                
                // Executar a transferência
                setup.blockchain_wrapper
                    .execute_tx(spender, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        sc.transfer_from(
                            managed_address!(owner),
                            managed_address!(recipient),
                            managed_biguint!(amount)
                        );
                    })
                    .assert_ok();
            }
        }
    }
    
    // Verificar consistência da oferta total
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            let total_supply = sc.total_token_supply().get();
            
            // Calcular a soma dos saldos de todos os usuários
            let mut sum_balances = managed_biguint!(0);
            for user in &setup.users {
                let balance = sc.balance_of(&managed_address!(user));
                sum_balances += balance;
            }
            
            // Verificar que a oferta total é igual à soma dos saldos
            assert_eq!(total_supply, sum_balances);
        })
        .assert_ok();
}

// Teste fuzzy para operações de aumento e diminuição de allowance
#[test]
fn test_increase_decrease_allowance_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(debt_token::contract_obj, 10);
    
    // Configurar aprovações iniciais
    for owner_idx in 0..setup.users.len() {
        for spender_idx in 0..setup.users.len() {
            if owner_idx != spender_idx {
                let owner = &setup.users[owner_idx];
                let spender = &setup.users[spender_idx];
                
                // Aprovação inicial
                let initial_amount = rng.gen_range(1000..5000u64);
                
                setup.blockchain_wrapper
                    .execute_tx(owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        sc.approve(managed_address!(spender), managed_biguint!(initial_amount));
                    })
                    .assert_ok();
            }
        }
    }
    
    // Realizar várias operações de aumento e diminuição
    for _ in 0..200 {
        let owner_idx = rng.gen_range(0..setup.users.len());
        let mut spender_idx = rng.gen_range(0..setup.users.len());
        
        // Garantir que owner e spender são diferentes
        while spender_idx == owner_idx {
            spender_idx = rng.gen_range(0..setup.users.len());
        }
        
        let owner = &setup.users[owner_idx];
        let spender = &setup.users[spender_idx];
        
        // Gerar valor aleatório para ajuste
        let amount = rng.gen_range(100..1000u64);
        
        // Decidir entre aumento e diminuição
        let is_increase = rng.gen_bool(0.5);
        
        if is_increase {
            // Aumentar allowance
            setup.blockchain_wrapper
                .execute_tx(owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    sc.increase_allowance(managed_address!(spender), managed_biguint!(amount));
                })
                .assert_ok();
        } else {
            // Diminuir allowance, mas verificar primeiro se há suficiente
            let current_allowance = setup.blockchain_wrapper
                .execute_query(&setup.contract_wrapper, |sc| {
                    sc.allowance(&managed_address!(owner), &managed_address!(spender))
                })
                .unwrap_or_default();
            
            // Calcular valor máximo que pode ser reduzido
            let max_decrease = current_allowance.to_u64().unwrap_or(0);
            
            if max_decrease > 0 {
                let decrease_amount = std::cmp::min(amount, max_decrease);
                
                setup.blockchain_wrapper
                    .execute_tx(owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        sc.decrease_allowance(managed_address!(spender), managed_biguint!(decrease_amount));
                    })
                    .assert_ok();
            }
        }
    }
    
    // Verificar que todas as allowances são válidas (não negativas)
    for owner_idx in 0..setup.users.len() {
        for spender_idx in 0..setup.users.len() {
            if owner_idx != spender_idx {
                let owner = &setup.users[owner_idx];
                let spender = &setup.users[spender_idx];
                
                setup.blockchain_wrapper
                    .execute_query(&setup.contract_wrapper, |sc| {
                        let allowance = sc.allowance(
                            &managed_address!(owner),
                            &managed_address!(spender)
                        );
                        
                        // Verificar que allowance não é negativa
                        assert!(allowance >= BigUint::zero());
                    })
                    .assert_ok();
            }
        }
    }
}

// Teste fuzzy para valores extremos
#[test]
fn test_extreme_values_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(debt_token::contract_obj, 5);
    
    // Testar valores extremos
    let extreme_values = [
        0u64,
        1u64,
        u64::MAX,
        u64::MAX / 2,
        u64::MAX - 1,
    ];
    
    for &value in &extreme_values {
        let user_idx = rng.gen_range(0..setup.users.len());
        let user = &setup.users[user_idx];
        
        // Mintar valor extremo (exceto para u64::MAX que seria muito grande)
        if value != u64::MAX {
            setup.blockchain_wrapper
                .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    sc.mint(managed_address!(user), managed_biguint!(value));
                })
                .assert_ok();
                
            // Verificar saldo
            setup.blockchain_wrapper
                .execute_query(&setup.contract_wrapper, |sc| {
                    let balance = sc.balance_of(&managed_address!(user));
                    assert_eq!(balance, managed_biguint!(value));
                })
                .assert_ok();
                
            // Queimar o valor
            setup.blockchain_wrapper
                .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    sc.burn(managed_address!(user), managed_biguint!(value));
                })
                .assert_ok();
        }
    }
    
    // Teste de transferência com valor zero
    let sender_idx = rng.gen_range(0..setup.users.len());
    let mut recipient_idx = rng.gen_range(0..setup.users.len());
    
    while recipient_idx == sender_idx {
        recipient_idx = rng.gen_range(0..setup.users.len());
    }
    
    let sender = &setup.users[sender_idx];
    let recipient = &setup.users[recipient_idx];
    
    // Mintar alguns tokens para o remetente
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(sender), managed_biguint!(1000));
        })
        .assert_ok();
    
    // Transferir valor zero
    setup.blockchain_wrapper
        .execute_tx(sender, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.transfer(managed_address!(recipient), managed_biguint!(0));
            
            // Verificar saldos não mudaram
            assert_eq!(sc.balance_of(&managed_address!(sender)), managed_biguint!(1000));
            assert_eq!(sc.balance_of(&managed_address!(recipient)), managed_biguint!(0));
        })
        .assert_ok();
}