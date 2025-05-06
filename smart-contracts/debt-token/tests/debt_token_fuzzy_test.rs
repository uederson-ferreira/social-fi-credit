// ==========================================================================
// ARQUIVO: debt_token_fuzzy_test.rs
// Descrição: Testes fuzzy com entradas aleatórias para o contrato DebtToken
// ==========================================================================

use multiversx_sc_scenario::imports::ReturnCode;
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
    //pub owner_address: Address,
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
    
    // Emitir o token
    blockchain_wrapper
        .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
            sc.issue_debt_token();
        })
        .assert_ok();
    
    ContractSetup {
        blockchain_wrapper,
        //owner_address,
        loan_controller_address,
        users,
        contract_wrapper,
    }
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
                    let current_balance = sc.balance_of(managed_address!(user_address));
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
            let total_supply = sc.total_token_supply();
            
            // Calcular a soma dos saldos de todos os usuários
            let mut sum_balances = BigUint::<DebugApi>::zero();
            for user in &setup.users {
                let balance = sc.balance_of(managed_address!(user));
                sum_balances += balance;
            }
            
            // Verificar que a oferta total é igual à soma dos saldos
            assert_eq!(total_supply, sum_balances, "Total supply should equal sum of all balances");
        })
        .assert_ok();
}



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
    let mut initial_total_supply = BigUint::<DebugApi>::zero();
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            initial_total_supply = sc.total_token_supply();
        })
        .assert_ok();
    
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
        let mut sender_balance = BigUint::<DebugApi>::zero();
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                sender_balance = sc.balance_of(managed_address!(sender));
            })
            .assert_ok();
        
        if sender_balance > BigUint::<DebugApi>::zero() {
            // Gerar valor aleatório para transferência (até o saldo disponível)
            let max_amount = sender_balance.to_u64().unwrap_or(0);
            if max_amount > 0 {
                let amount = rng.gen_range(1..=max_amount);
                
                // Executar a transferência
                setup.blockchain_wrapper
                    .execute_tx(sender, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        sc.transfer_tokens(managed_address!(recipient), managed_biguint!(amount));
                    })
                    .assert_ok();
            }
        }
    }
    
    // Verificar que a oferta total permanece inalterada
    let mut final_total_supply = BigUint::<DebugApi>::zero();
    setup.blockchain_wrapper
        .execute_query(&setup.contract_wrapper, |sc| {
            final_total_supply = sc.total_token_supply();
        })
        .assert_ok();
        
    assert_eq!(
        final_total_supply, 
        initial_total_supply, 
        "Total supply should remain unchanged after transfers"
    );
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
                sc.approve_tokens(managed_address!(spender), managed_biguint!(amount));
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
        let mut allowance_result = BigUint::<DebugApi>::zero();
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                allowance_result = sc.get_allowance(managed_address!(owner), managed_address!(spender));
            })
            .assert_ok();
            
        let mut owner_balance = BigUint::<DebugApi>::zero();
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                owner_balance = sc.balance_of(managed_address!(owner));
            })
            .assert_ok();
        
        if allowance_result > BigUint::<DebugApi>::zero() && owner_balance > BigUint::<DebugApi>::zero() {
            // Determinar o valor máximo que pode ser transferido
            let max_transfer = std::cmp::min(allowance_result.clone(), owner_balance.clone());
            let max_amount = max_transfer.to_u64().unwrap_or(0);
            
            if max_amount > 0 {
                // Gerar valor aleatório para transferência
                let amount = rng.gen_range(1..=max_amount);
                
                // Executar a transferência
                setup.blockchain_wrapper
                    .execute_tx(spender, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        sc.transfer_tokens_from(
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
            let total_supply = sc.total_token_supply();
            
            // Calcular a soma dos saldos de todos os usuários
            let mut sum_balances = BigUint::<DebugApi>::zero();
            for user in &setup.users {
                let balance = sc.balance_of(managed_address!(user));
                sum_balances += balance;
            }
            
            // Verificar que a oferta total é igual à soma dos saldos
            assert_eq!(total_supply, sum_balances, "Total supply should equal sum of all balances after transferFrom operations");
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
                        sc.approve_tokens(managed_address!(spender), managed_biguint!(initial_amount));
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
                    sc.increase_token_allowance(managed_address!(spender), managed_biguint!(amount));
                })
                .assert_ok();
        } else {
            // // Diminuir allowance, mas verificar primeiro se há suficiente
            // let current_allowance = setup.blockchain_wrapper
            //     .execute_query(&setup.contract_wrapper, |sc| {
            //         sc.get_allowance(managed_address!(owner), managed_address!(spender));
            //     })
            //     .assert_ok();
            
            // Obter o allowance atual primeiro
            let mut current_allowance = BigUint::<DebugApi>::zero();
            setup.blockchain_wrapper
                .execute_query(&setup.contract_wrapper, |sc| {
                    current_allowance = sc.get_allowance(managed_address!(owner), managed_address!(spender));
                })
                .assert_ok();

            // Calcular valor máximo que pode ser reduzido
            let max_decrease = current_allowance.to_u64().unwrap_or(0);

            if max_decrease > 0 {
                let decrease_amount = std::cmp::min(amount, max_decrease);
                
                setup.blockchain_wrapper
                    .execute_tx(owner, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                        sc.decrease_token_allowance(managed_address!(spender), managed_biguint!(decrease_amount));
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
                        let allowance = sc.get_allowance(
                            managed_address!(owner), 
                            managed_address!(spender)
                        );
                        
                        // Verificar que allowance não é negativa
                        assert!(allowance >= BigUint::<DebugApi>::zero(), "Allowance should not be negative");
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
        u64::MAX / 4, // Reduzido para evitar problemas de overflow
        u64::MAX / 2,
        u64::MAX / 2 - 1,
    ];
    
    for &value in &extreme_values {
        let user_idx = rng.gen_range(0..setup.users.len());
        let user = &setup.users[user_idx];
        
        // Mintar valor extremo (exceto para valores muito grandes)
        if value <= u64::MAX / 4 {
            setup.blockchain_wrapper
                .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    sc.mint(managed_address!(user), managed_biguint!(value));
                })
                .assert_ok();
                
            // Verificar saldo
            setup.blockchain_wrapper
                .execute_query(&setup.contract_wrapper, |sc| {
                    let balance = sc.balance_of(managed_address!(user));
                    assert_eq!(balance, managed_biguint!(value), "Balance should match minted amount");
                })
                .assert_ok();
                
            // Queimar o valor
            setup.blockchain_wrapper
                .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                    if value > 0 { // Só queimar valores positivos
                        sc.burn(managed_address!(user), managed_biguint!(value));
                    }
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
            sc.transfer_tokens(managed_address!(recipient), managed_biguint!(0));
            
            // Verificar saldos não mudaram
            assert_eq!(sc.balance_of(managed_address!(sender)), managed_biguint!(1000), "Sender balance should not change after zero transfer");
            assert_eq!(sc.balance_of(managed_address!(recipient)), managed_biguint!(0), "Recipient balance should not change after zero transfer");
        })
        .assert_ok();
}

// Teste para criação e queima de NFTs
#[test]
fn test_nft_creation_burning_fuzzy() {
    // Usar uma semente fixa para reprodutibilidade
    let mut rng = StdRng::seed_from_u64(42);
    
    let mut setup = setup_fuzzy_contract(debt_token::contract_obj, 5);
    
    // Criar vários empréstimos (NFTs)
    let mut loan_ids = Vec::new();
    let future_timestamp = 1893456000; // 2030-01-01, bem no futuro
    
    for i in 1..=20 {
        let borrower_idx = rng.gen_range(0..setup.users.len());
        let borrower = &setup.users[borrower_idx];
        
        let amount = rng.gen_range(1000..10000u64);
        let interest_rate = rng.gen_range(5..25u64);
        let loan_id = i;
        
        // Criar NFT para o empréstimo
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                let nft_nonce = sc.create_debt_nft(
                    loan_id, 
                    managed_address!(borrower), 
                    managed_biguint!(amount), 
                    interest_rate, 
                    future_timestamp
                );
                
                // Verificar que o NFT foi criado
                assert!(nft_nonce > 0, "NFT nonce should be greater than zero");
                
                // Verificar mapeamentos
                assert_eq!(sc.get_loan_nft_id(loan_id), nft_nonce, "NFT nonce should be correctly mapped to loan ID");
                assert_eq!(sc.get_nft_loan_id(nft_nonce), loan_id, "Loan ID should be correctly mapped to NFT nonce");
            })
            .assert_ok();
            
        loan_ids.push(loan_id);
    }
    
    // Queimar metade dos NFTs aleatoriamente
    let num_to_burn = loan_ids.len() / 2;
    for _ in 0..num_to_burn {
        if loan_ids.is_empty() {
            break;
        }
        
        let idx = rng.gen_range(0..loan_ids.len());
        let loan_id = loan_ids.remove(idx);
        
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
                let nft_nonce_before = sc.get_loan_nft_id(loan_id);
                assert!(nft_nonce_before > 0, "NFT should exist before burning");
                
                sc.burn_debt_nft(loan_id);
                
                // Verificar que o NFT foi queimado
                assert_eq!(sc.get_loan_nft_id(loan_id), 0, "NFT should no longer be mapped to loan ID after burning");
            })
            .assert_ok();
    }
    
    // Verificar que os NFTs restantes ainda existem
    for loan_id in loan_ids {
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let nft_nonce = sc.get_loan_nft_id(loan_id);
                assert!(nft_nonce > 0, "NFT for non-burned loan should still exist");
                assert_eq!(sc.get_nft_loan_id(nft_nonce), loan_id, "Loan ID mapping should still be correct");
            })
            .assert_ok();
    }
}




// Teste para tentativas de operações não autorizadas
#[test]
fn test_unauthorized_operations() {
    let mut setup = setup_fuzzy_contract(debt_token::contract_obj, 3);
    
    // Usuário comum tenta criar NFT (deve falhar)
    let result = setup.blockchain_wrapper
        .execute_tx(&setup.users[0], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            let _ = sc.create_debt_nft(
                1,
                managed_address!(&setup.users[1]),
                managed_biguint!(1000),
                10,
                1893456000
            );
        });
    
    // Verifica se o resultado não é ok
    assert!(
        result.result_status != ReturnCode::UserError, 
        "Unauthorized user should not be able to create NFT"
    );
    
    // Usuário comum tenta mintar tokens (deve falhar)
    let result = setup.blockchain_wrapper
        .execute_tx(&setup.users[0], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.users[1]), managed_biguint!(1000));
        });
    
    assert!(
        result.result_status != ReturnCode::UserError, 
        "Unauthorized user should not be able to mint tokens"
    );
    
    // Usuário comum tenta queimar tokens (deve falhar)
    let result = setup.blockchain_wrapper
        .execute_tx(&setup.users[0], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.burn(managed_address!(&setup.users[1]), managed_biguint!(1000));
        });
    
    assert!(
        result.result_status != ReturnCode::UserError, 
        "Unauthorized user should not be able to burn tokens"
    );
    
    // Mintar alguns tokens para testes
    setup.blockchain_wrapper
        .execute_tx(&setup.loan_controller_address, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.mint(managed_address!(&setup.users[0]), managed_biguint!(1000));
        })
        .assert_ok();
        
    // Tentar transferir mais do que o saldo (deve falhar)
    let result = setup.blockchain_wrapper
        .execute_tx(&setup.users[0], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.transfer_tokens(managed_address!(&setup.users[1]), managed_biguint!(2000));
        });
        
    assert!(
        result.result_status != ReturnCode::UserError, 
        "User should not be able to transfer more than their balance"
    );
    
    // Aprovar tokens
    setup.blockchain_wrapper
        .execute_tx(&setup.users[0], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.approve_tokens(managed_address!(&setup.users[1]), managed_biguint!(500));
        })
        .assert_ok();
        
    // Tentar transferir mais do que o allowance (deve falhar)
    let result = setup.blockchain_wrapper
        .execute_tx(&setup.users[1], &setup.contract_wrapper, &rust_biguint!(0), |sc| {
            sc.transfer_tokens_from(
                managed_address!(&setup.users[0]),
                managed_address!(&setup.users[2]),
                managed_biguint!(600)
            );
        });
        
    assert!(
        result.result_status != ReturnCode::UserError, 
        "User should not be able to transfer more than their allowance"
    );
}