// // tests/testesimples.rs
// use multiversx_sc_scenario::{
//     rust_biguint, managed_address, managed_biguint,
//     testing_framework::BlockchainStateWrapper,
// };
// use debt_token::*;

// const WASM_PATH: &str = "output/debt-token.wasm";

// #[test]
// fn test_simple_assertion() {
//     assert_eq!(2 + 2, 4);
// }

// #[test]
// fn test_basic_deployment() {
//     let rust_zero = rust_biguint!(0u64);
//     let mut blockchain_wrapper = BlockchainStateWrapper::new();
//     let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
//     let loan_controller_address = blockchain_wrapper.create_user_account(&rust_zero);
    
//     // Deploy do contrato
//     let contract_wrapper = blockchain_wrapper.create_sc_account(
//         &rust_zero,
//         Some(&owner_address),
//         debt_token::contract_obj,
//         WASM_PATH,
//     );
    
//     // Inicialização do contrato
//     blockchain_wrapper
//         .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
//             sc.init(managed_address!(&loan_controller_address));
//         })
//         .assert_ok();
    
//     // Verificar se o controlador de empréstimos foi configurado corretamente
//     blockchain_wrapper
//         .execute_query(&contract_wrapper, |sc| {
//             assert_eq!(
//                 sc.loan_controller_address().get(),
//                 managed_address!(&loan_controller_address)
//             );
//         })
//         .assert_ok();
// }


// #[test]
// fn test_fuzzy_create_debt_nft() {
//     // Definir um valor inicial de EGLD para o proprietário
//     let initial_owner_balance = rust_biguint!(1_000_000_000_000_000_000u64); // 1 EGLD
//     let rust_zero = rust_biguint!(0u64);
    
//     let mut blockchain_wrapper = BlockchainStateWrapper::new();
    
//     // Criar contas com saldo
//     let owner_address = blockchain_wrapper.create_user_account(&initial_owner_balance);
//     let loan_controller_address = blockchain_wrapper.create_user_account(&rust_zero);
//     let borrower_address = blockchain_wrapper.create_user_account(&rust_zero);
    
//     // Deploy do contrato
//     let contract_wrapper = blockchain_wrapper.create_sc_account(
//         &rust_zero,
//         Some(&owner_address),
//         debt_token::contract_obj,
//         WASM_PATH,
//     );
    
//     // Inicialização do contrato
//     blockchain_wrapper
//         .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
//             sc.init(managed_address!(&loan_controller_address));
//         })
//         .assert_ok();
    
//     // Emitir o token de dívida com taxa em EGLD
//     let token_issue_cost = rust_biguint!(50_000_000_000_000_000u64); // 0.05 EGLD
//     blockchain_wrapper
//         .execute_tx(&owner_address, &contract_wrapper, &token_issue_cost, |sc| {
//             sc.issue_debt_token();
//         })
//         .assert_ok();
    
//     // Simular a definição do ID do token após a emissão
//     blockchain_wrapper
//         .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
//             // Verificar se o token foi emitido
//             let token_issued = !sc.debt_token_id().is_empty();
//             println!("Token emitido: {}", token_issued);
//         })
//         .assert_ok();
// }
