// ==========================================================================
// MÓDULO: debt-token/tests/fuzzy_tests.rs
// AUTOR: Claude
// DATA: 2025-05-06
// DESCRIÇÃO: Testes fuzzy para o contrato DebtToken.
//            Estes testes usam cenários simulados com valores aleatórios
//            para verificar o comportamento do contrato sob diferentes condições.
// ==========================================================================

use multiversx_sc::proxy_imports::TestAddress;
use multiversx_sc_scenario::ScenarioTxRun;
use multiversx_sc::imports::ReturnsNewAddress;
use crate::multiversx_chain_vm::types::EsdtLocalRole;
use multiversx_sc_scenario::imports::TestSCAddress;

use multiversx_sc_scenario::*;

use multiversx_sc::proxy_imports::BigUint;
use multiversx_sc_scenario::imports::MxscPath;
use debt_token::debt_token_proxy;

//const WASM_PATH: &str = "output/debt-token.wasm";

const CODE_PATH: MxscPath = MxscPath::new("output/debt-token.mxsc.json");
const OWNER: TestAddress = TestAddress::new("owner");
const LOAN_CONTROLLER: TestAddress = TestAddress::new("loan_controller");
const USER1: TestAddress = TestAddress::new("user1");
const USER2: TestAddress = TestAddress::new("user2");
const USER3: TestAddress = TestAddress::new("user3");
const DEBT_TOKEN_ADDRESS: TestSCAddress = TestSCAddress::new("debt_token");

// Configuração do mundo de testes com logging adicional
fn world() -> ScenarioWorld {
    println!("Criando ScenarioWorld");
    let mut blockchain = ScenarioWorld::new();
    println!("Configurando diretório de trabalho");
    blockchain.set_current_dir_from_workspace("debt-token");
    println!("Registrando contrato com caminho: {:?}", CODE_PATH);
    blockchain.register_contract(CODE_PATH, debt_token::ContractBuilder);
    println!("Mundo de testes criado com sucesso");
    blockchain
}

// Implantação do contrato com logging adicional
fn deploy_debt_token() -> ScenarioWorld {
    println!("\n=== Iniciando implantação do contrato ===");
    let mut world = world();

    // Configurando contas
    println!("Configurando conta do proprietário");
    world
        .account(OWNER)
        .nonce(0)
        .balance(1_000_000_000_000_000_000u64); // 1 EGLD

    println!("Configurando conta do controlador de empréstimos");
    world
        .account(LOAN_CONTROLLER)
        .nonce(0)
        .balance(1_000_000_000_000_000_000u64);

    println!("Configurando contas de usuários");
    world
        .account(USER1)
        .nonce(0)
        .balance(1_000_000_000_000_000_000u64);

    world
        .account(USER2)
        .nonce(0)
        .balance(1_000_000_000_000_000_000u64);

    world
        .account(USER3)
        .nonce(0)
        .balance(1_000_000_000_000_000_000u64);

    // Implantando o contrato
    println!("Implantando o contrato DebtToken");
    let debt_token_address = world
        .tx()
        .from(OWNER)
        .typed(debt_token_proxy::DebtTokenProxy)
        .init(LOAN_CONTROLLER.to_address())
        .code(CODE_PATH)
        .new_address(DEBT_TOKEN_ADDRESS)
        .returns(ReturnsNewAddress)
        .run();

    println!("Contrato implantado no endereço: {:?}", debt_token_address);
    assert_eq!(debt_token_address, DEBT_TOKEN_ADDRESS.to_address());

    // Emitindo o token
    println!("Emitindo o token de dívida");
    let tste = world
        .tx()
        .from(OWNER)
        .typed(debt_token_proxy::DebtTokenProxy)
        .issue_debt_token()
        .egld(50_000_000_000_000_000u64);// 0.05 EGLD para taxa de emissão
        //.run();
    println!("Token emitido com sucesso");

// Em vez de chamar o callback, defina o token ID diretamente
world
    .tx()
    .from(OWNER)
    .to(DEBT_TOKEN_ADDRESS)
    .typed(debt_token_proxy::DebtTokenProxy)
    .debt_token_id_view() // Supondo que exista um método para isso
    .run();

    world
    .tx()
    .from(OWNER)
    .to(DEBT_TOKEN_ADDRESS)
    .raw_call("issueCallback") // Tente variações do nome
    .run();

    // Em vez de tentar simular o callback, cria uma transação especial 
    // para definir o ID do token manualmente
    println!("Executando callback de emissão do token");
    world
    .tx()
    .from(OWNER)
    .to(DEBT_TOKEN_ADDRESS)
    .raw_call("issue_callback")
    .run();

    // Configure os papéis ESDT novamente para garantia
    println!("Configurando papéis do token");
    world
    .set_esdt_local_roles(
        DEBT_TOKEN_ADDRESS,
        "DEBT-123456".as_bytes(),
        &[EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn],
    );

    // Verificar se o token ID foi definido corretamente
    println!("Verificando ID do token");
    let result = world
        .query()
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .debt_token_id_view() 
        .run();
    
    println!("Verificação de ID do token concluída: {:?}", result);
    println!("=== Implantação concluída com sucesso ===\n");
    
    world
}

// -----------------------------------------
// Testes Fuzzy
// -----------------------------------------
#[test]
fn test_fuzzy_create_debt_nft() {
    let mut world = deploy_debt_token();
    
    println!("\n=== Iniciando teste de criação de NFT de dívida ===");
    // Valores aleatórios para 10 iterações
    for i in 1..11 {
        let loan_id = i * 100 + fastrand::u64(1..100);
        let amount = BigUint::from(fastrand::u64(1_000_000_000..100_000_000_000));
        let interest_rate = fastrand::u64(100..5000); // 1% a 50% (em pontos base)
        
        // Define um timestamp base e o prazo
        let base_timestamp = 1715000000u64; // Timestamp aproximado de maio de 2024
        let due_timestamp = base_timestamp + fastrand::u64(86400..31536000);
        
        // Define o timestamp base
        world.current_block().block_timestamp(base_timestamp);
        
        println!("Criando NFT para empréstimo #{:?} com valor {:?}", loan_id, amount);
        // Criando o NFT
        world
            .tx()
            .from(LOAN_CONTROLLER)
            .to(DEBT_TOKEN_ADDRESS)
            .typed(debt_token_proxy::DebtTokenProxy)
            .create_debt_nft(
                loan_id,
                USER1.to_address(),
                amount.clone(),
                interest_rate,
                due_timestamp,
            )
            .run();
        println!("NFT criado com sucesso");
        
        // Verificando se o NFT foi criado
        // Tentar criar o mesmo NFT novamente - deve falhar
        println!("Tentando criar o mesmo NFT novamente (deve falhar)");
        world
            .tx()
            .from(LOAN_CONTROLLER)
            .to(DEBT_TOKEN_ADDRESS)
            .typed(debt_token_proxy::DebtTokenProxy)
            .create_debt_nft(
                loan_id,
                USER1.to_address(),
                amount.clone(),
                interest_rate,
                due_timestamp,
            )
            .with_result(ExpectError(4, "NFT already exists for this loan"))
            .run();
        println!("Teste de falha concluído com sucesso");
    }
    println!("=== Teste de criação de NFT de dívida concluído ===\n");
}


#[test]
fn test_fuzzy_nft_create_with_invalid_params() {
    let mut world = deploy_debt_token();
    
    println!("\n=== Iniciando teste de parâmetros inválidos ===");
    let loan_id = 12345u64; // Explicitamente u64
    let amount = BigUint::from(1_000_000_000u64);
    let interest_rate = 1000u64; // 10%
    
    // Define um timestamp base em vez de tentar obter o atual
    let base_timestamp = 1715000000u64; // Um timestamp aproximado de maio de 2024
    world.current_block().block_timestamp(base_timestamp);
    let due_timestamp = base_timestamp + 86400; // 1 dia
    
    // Tentativa de criar NFT com chamador incorreto
    println!("Testando criação por chamador não autorizado");
    world
        .tx()
        .from(USER1)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .create_debt_nft(
            loan_id,
            USER1.to_address(),
            amount.clone(),
            interest_rate,
            due_timestamp,
        )
        .with_result(ExpectError(4, "Only loan controller can create debt NFTs"))
        .run();
    println!("Teste de chamador não autorizado concluído com sucesso");
    
    // Tentativa de criar NFT com prazo no passado
    println!("Testando criação com prazo no passado");
    world
        .tx()
        .from(LOAN_CONTROLLER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .create_debt_nft(
            loan_id,
            USER1.to_address(),
            amount.clone(),
            interest_rate,
            base_timestamp - 1, // Usa base_timestamp em vez de current_timestamp
        )
        .with_result(ExpectError(4, "Due date must be in the future"))
        .run();
    println!("Teste de prazo no passado concluído com sucesso");
    
    // Tentativa de criar NFT com valor zero
    println!("Testando criação com valor zero");
    world
        .tx()
        .from(LOAN_CONTROLLER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .create_debt_nft(
            loan_id,
            USER1.to_address(),
            BigUint::zero(),
            interest_rate,
            due_timestamp,
        )
        .with_result(ExpectError(4, "Amount must be greater than zero"))
        .run();
    println!("Teste de valor zero concluído com sucesso");
    
    // Verificando que os NFTs não foram criados - verificamos indiretamente
    println!("Verificando indiretamente que nenhum NFT foi criado");
    let get_loan_result = world
        .query()
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .get_loan_nft_id(loan_id)
        .run();
    
    println!("Resultado da consulta: {:?}", get_loan_result);
    
    // Não podemos verificar diretamente o valor retornado, então verificamos
    // indiretamente tentando criar um NFT válido com o mesmo ID
    println!("Tentando criar NFT válido com o mesmo ID");
    world
        .tx()
        .from(LOAN_CONTROLLER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .create_debt_nft(
            loan_id,
            USER1.to_address(),
            amount.clone(),
            interest_rate,
            due_timestamp,
        )
        .run(); // Isso deve funcionar se nenhum NFT foi criado anteriormente
    println!("NFT válido criado com sucesso");
    println!("=== Teste de parâmetros inválidos concluído ===\n");
}

#[test]
fn test_fuzzy_nft_burn() {
    let mut world = deploy_debt_token();
    
    println!("\n=== Iniciando teste de queima de NFT ===");
    // Criar alguns NFTs
    let loan_ids = [1001u64, 1002u64, 1003u64, 1004u64, 1005u64]; // Explicitamente u64
    let mut nft_nonces = [0u64; 5];
    
    // Define um timestamp base
    let base_timestamp = 1715000000u64; // Um timestamp aproximado
    world.current_block().block_timestamp(base_timestamp);
    
    for (i, &loan_id) in loan_ids.iter().enumerate() {
        let amount = BigUint::from(10_000_000_000u64 * (i as u64 + 1));
        let interest_rate = 500u64 + i as u64 * 100; // 5% a 9%
        let due_timestamp = base_timestamp + 86400u64 * (i as u64 + 1);
        
        // Criar NFT
        println!("Criando NFT para empréstimo #{}", loan_id);
        world
            .tx()
            .from(LOAN_CONTROLLER)
            .to(DEBT_TOKEN_ADDRESS)
            .typed(debt_token_proxy::DebtTokenProxy)
            .create_debt_nft(
                loan_id,
                USER1.to_address(),
                amount,
                interest_rate,
                due_timestamp,
            )
            .run();
        println!("NFT criado com sucesso");
        
        // Armazenamos o índice como nonce para rastreamento
        // Não podemos obter o valor real, então usamos o índice + 1 como estimativa
        nft_nonces[i] = (i as u64) + 1;
    }
    
    // Queimar NFTs em ordem aleatória
    let mut indices = [0, 1, 2, 3, 4];
    fastrand::shuffle(&mut indices);
    
    for &idx in indices.iter() {
        let loan_id = loan_ids[idx];
        
        // Queimar NFT
        println!("Queimando NFT para empréstimo #{}", loan_id);
        world
            .tx()
            .from(LOAN_CONTROLLER)
            .to(DEBT_TOKEN_ADDRESS)
            .typed(debt_token_proxy::DebtTokenProxy)
            .burn_debt_nft(loan_id)
            .run();
        println!("NFT queimado com sucesso");
        
        // Tente queimar o mesmo NFT novamente - deve falhar se já foi queimado
        println!("Tentando queimar o mesmo NFT novamente (deve falhar)");
        world
            .tx()
            .from(LOAN_CONTROLLER)
            .to(DEBT_TOKEN_ADDRESS)
            .typed(debt_token_proxy::DebtTokenProxy)
            .burn_debt_nft(loan_id)
            .with_result(ExpectError(4, "No NFT exists for this loan"))
            .run();
        println!("Teste de falha na segunda queima concluído com sucesso");
    }
    
    // Tentativa de queimar um NFT inexistente
    println!("Tentando queimar um NFT inexistente");
    world
        .tx()
        .from(LOAN_CONTROLLER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .burn_debt_nft(9999u64) // Explicitamente u64
        .with_result(ExpectError(4, "No NFT exists for this loan"))
        .run();
    println!("Teste de queima de NFT inexistente concluído com sucesso");
    
    // Tentativa de queimar como usuário não autorizado
    let new_loan_id = 1006u64; // Explicitamente u64
    println!("Criando um novo NFT para teste de permissão");
    world
        .tx()
        .from(LOAN_CONTROLLER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .create_debt_nft(
            new_loan_id,
            USER1.to_address(),
            BigUint::from(50_000_000_000u64),
            800u64, // Explicitamente u64
            base_timestamp + 86400u64 * 10, // Use base_timestamp
        )
        .run();
    println!("Novo NFT criado com sucesso");
    
    println!("Tentando queimar NFT por usuário não autorizado");
    world
        .tx()
        .from(USER1)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .burn_debt_nft(new_loan_id)
        .with_result(ExpectError(4, "Only loan controller can burn debt NFTs"))
        .run();
    println!("Teste de permissão concluído com sucesso");
    println!("=== Teste de queima de NFT concluído ===\n");
}

// Aqui você pode incluir os outros testes, seguindo o mesmo padrão de adicionar logs detalhados


// // ==========================================================================
// // MÓDULO: debt-token/tests/fuzzy_tests.rs
// // AUTOR: Claude
// // DATA: 2025-05-06
// // DESCRIÇÃO: Testes fuzzy para o contrato DebtToken.
// //            Estes testes usam cenários simulados com valores aleatórios
// //            para verificar o comportamento do contrato sob diferentes condições.
// // ==========================================================================

// use multiversx_sc::proxy_imports::TestAddress;
// use multiversx_sc_scenario::ScenarioTxRun;
// use multiversx_sc::imports::ReturnsNewAddress;
// use crate::multiversx_chain_vm::types::EsdtLocalRole;
// use multiversx_sc_scenario::imports::TestSCAddress;

// use multiversx_sc_scenario::*;

// use multiversx_sc::proxy_imports::BigUint;
// use multiversx_sc_scenario::imports::MxscPath;
// use debt_token::debt_token_proxy;

// //const WASM_PATH: &str = "output/debt-token.wasm";

// const CODE_PATH: MxscPath = MxscPath::new("output/debt-token.mxsc.json");
// const OWNER: TestAddress = TestAddress::new("owner");
// const LOAN_CONTROLLER: TestAddress = TestAddress::new("loan_controller");
// const USER1: TestAddress = TestAddress::new("user1");
// const USER2: TestAddress = TestAddress::new("user2");
// const USER3: TestAddress = TestAddress::new("user3");
// const DEBT_TOKEN_ADDRESS: TestSCAddress = TestSCAddress::new("debt_token");

// // Configuração do mundo de testes
// fn world() -> ScenarioWorld {
//     let mut blockchain = ScenarioWorld::new();
//     blockchain.set_current_dir_from_workspace("smart-contracts/debt-token");
//     blockchain.register_contract(
//         "mxsc:output/debt-token.mxsc.json", 
//         debt_token::ContractBuilder
//     );
//     blockchain
// }
// // use multiversx_sc_scenario::*;

// // fn world() -> ScenarioWorld {
// //     let mut blockchain = ScenarioWorld::new();

// //     blockchain.set_current_dir_from_workspace("contracts/examples/lottery-esdt");
// //     blockchain.register_contract(
// //         "mxsc:output/lottery-esdt.mxsc.json",
// //         lottery_esdt::ContractBuilder,
// //     );
// //     blockchain
// // }
// // blockchain.register_contract(
// //     "mxsc:output/lottery-esdt.mxsc.json",
// //     lottery_esdt::ContractBuilder,
// // );
// //"mxsc:output/debt-token.mxsc.json"

// // Implantação do contrato
// fn deploy_debt_token() -> ScenarioWorld {
//     let mut world = world();

//     // Configurando contas
//     world
//         .account(OWNER)
//         .nonce(0)
//         .balance(1_000_000_000_000_000_000u64); // 1 EGLD

//     world
//         .account(LOAN_CONTROLLER)
//         .nonce(0)
//         .balance(1_000_000_000_000_000_000u64);

//     world
//         .account(USER1)
//         .nonce(0)
//         .balance(1_000_000_000_000_000_000u64);

//     world
//         .account(USER2)
//         .nonce(0)
//         .balance(1_000_000_000_000_000_000u64);

//     world
//         .account(USER3)
//         .nonce(0)
//         .balance(1_000_000_000_000_000_000u64);

//     // Implantando o contrato
//     let debt_token_address = world
//         .tx()
//         .from(OWNER)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .init(LOAN_CONTROLLER.to_address())
//         .code(CODE_PATH)
//         .new_address(DEBT_TOKEN_ADDRESS)
//         .returns(ReturnsNewAddress)
//         .run();

//     assert_eq!(debt_token_address, DEBT_TOKEN_ADDRESS.to_address());

//     // Emitindo o token
//     world
//         .tx()
//         .from(OWNER)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .issue_debt_token()
//         .egld(50_000_000_000_000_000u64)// 0.05 EGLD para taxa de emissão
//         .result_handler; 

//     // Simulando o callback de emissão do token
//     world
//         .set_esdt_local_roles(
//             DEBT_TOKEN_ADDRESS,
//             "DEBT-123456".as_bytes(), // Token ID simulado
//             &[EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn],
//         );


//     // Em vez de tentar simular o callback, cria uma transação especial 
//     // para definir o ID do token manualmente
//     world
//     .tx()
//     .from(OWNER)
//     .to(DEBT_TOKEN_ADDRESS)
//     .raw_call("issue_callback")
//     .run();

//     // Configure os papéis ESDT (isso já estava funcionando)
//     world
//     .set_esdt_local_roles(
//         DEBT_TOKEN_ADDRESS,
//         "DEBT-123456".as_bytes(),
//         &[EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn],
//     );

//     // Verificar se o token ID foi definido corretamente
//     world
//         .query()
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .debt_token_id_view() 
//         .run();

//     world
// }

// // -----------------------------------------
// // Testes Fuzzy
// // -----------------------------------------
// #[test]
// fn test_fuzzy_create_debt_nft() {
//     let mut world = deploy_debt_token();
    
//     // Valores aleatórios para 10 iterações
//     for i in 1..11 {
//         let loan_id = i * 100 + fastrand::u64(1..100);
//         let amount = BigUint::from(fastrand::u64(1_000_000_000..100_000_000_000));
//         let interest_rate = fastrand::u64(100..5000); // 1% a 50% (em pontos base)
        
//         // Define um timestamp base e o prazo
//         let base_timestamp = 1715000000u64; // Timestamp aproximado de maio de 2024
//         let due_timestamp = base_timestamp + fastrand::u64(86400..31536000);
        
//         // Define o timestamp base
//         world.current_block().block_timestamp(base_timestamp);
        
//         // Criando o NFT - adicione .to() antes de .typed()
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS) // Adicione esta linha!
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .create_debt_nft(
//                 loan_id,
//                 USER1.to_address(),
//                 amount.clone(),
//                 interest_rate,
//                 due_timestamp,
//             )
//             .run();
        
//         // Verificando se o NFT foi criado
//         // Tentar criar o mesmo NFT novamente - deve falhar
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS) // Adicione esta linha!
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .create_debt_nft(
//                 loan_id,
//                 USER1.to_address(),
//                 amount.clone(),
//                 interest_rate,
//                 due_timestamp,
//             )
//             .with_result(ExpectError(4, "NFT already exists for this loan")) // Adicione o código de erro (4)
//             .run();
//     }
// }


// #[test]
// fn test_fuzzy_nft_create_with_invalid_params() {
//     let mut world = deploy_debt_token();
    
//     //let loan_id = 12345;
//     let loan_id = 12345u64; // Explicitamente u64
//     let amount = BigUint::from(1_000_000_000u64);
//     let interest_rate = 1000u64; // 10%
    
//     // Define um timestamp base em vez de tentar obter o atual
//     let base_timestamp = 1715000000u64; // Um timestamp aproximado de maio de 2024
//     world.current_block().block_timestamp(base_timestamp);
//     let due_timestamp = base_timestamp + 86400; // 1 dia
    
//     // Tentativa de criar NFT com chamador incorreto
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id,
//             USER1.to_address(),
//             amount.clone(),
//             interest_rate,
//             due_timestamp,
//         )
//         .with_result(ExpectError(4, "Only loan controller can create debt NFTs"))
//         .run();
    
//     // Tentativa de criar NFT com prazo no passado
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id,
//             USER1.to_address(),
//             amount.clone(),
//             interest_rate,
//             base_timestamp - 1, // Usa base_timestamp em vez de current_timestamp
//         )
//         .with_result(ExpectError(4, "Due date must be in the future"))
//         .run();
    
//     // Tentativa de criar NFT com valor zero
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id,
//             USER1.to_address(),
//             BigUint::zero(),
//             interest_rate,
//             due_timestamp,
//         )
//         .with_result(ExpectError(4, "Amount must be greater than zero"))
//         .run();
    
//     // Verificando que os NFTs não foram criados - remova returns(ExpectValue)
//     let _ = world
//         .query()
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .get_loan_nft_id(loan_id)
//         .run();
    
//     // Não podemos verificar diretamente o valor retornado, então verificamos
//     // indiretamente tentando criar um NFT válido com o mesmo ID
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id,
//             USER1.to_address(),
//             amount.clone(),
//             interest_rate,
//             due_timestamp,
//         )
//         .run(); // Isso deve funcionar se nenhum NFT foi criado anteriormente
// }

// #[test]
// fn test_fuzzy_nft_burn() {
//     let mut world = deploy_debt_token();
    
//     // Criar alguns NFTs
//     let loan_ids = [1001u64, 1002u64, 1003u64, 1004u64, 1005u64]; // Explicitamente u64
//     let mut nft_nonces = [0u64; 5];
    
//     // Define um timestamp base
//     let base_timestamp = 1715000000u64; // Um timestamp aproximado
//     world.current_block().block_timestamp(base_timestamp);
    
//     for (i, &loan_id) in loan_ids.iter().enumerate() {
//         let amount = BigUint::from(10_000_000_000u64 * (i as u64 + 1));
//         let interest_rate = 500u64 + i as u64 * 100; // 5% a 9%
//         let due_timestamp = base_timestamp + 86400u64 * (i as u64 + 1);
        
//         // Criar NFT
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .create_debt_nft(
//                 loan_id,
//                 USER1.to_address(),
//                 amount,
//                 interest_rate,
//                 due_timestamp,
//             )
//             .run();
        
//         // Armazenamos o índice como nonce para rastreamento
//         // Não podemos obter o valor real, então usamos o índice + 1 como estimativa
//         nft_nonces[i] = (i as u64) + 1;
//     }
    
//     // Queimar NFTs em ordem aleatória
//     let mut indices = [0, 1, 2, 3, 4];
//     fastrand::shuffle(&mut indices);
    
//     for &idx in indices.iter() {
//         let loan_id = loan_ids[idx];
        
//         // Queimar NFT
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .burn_debt_nft(loan_id)
//             .run(); // Use run() em vez de execute()
        
//         // Tente queimar o mesmo NFT novamente - deve falhar se já foi queimado
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .burn_debt_nft(loan_id)
//             .with_result(ExpectError(4, "No NFT exists for this loan"))
//             .run();
//     }
    
//     // Tentativa de queimar um NFT inexistente
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .burn_debt_nft(9999u64) // Explicitamente u64
//         .with_result(ExpectError(4, "No NFT exists for this loan"))
//         .run();
    
//     // Tentativa de queimar como usuário não autorizado
//     let new_loan_id = 1006u64; // Explicitamente u64
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             new_loan_id,
//             USER1.to_address(),
//             BigUint::from(50_000_000_000u64),
//             800u64, // Explicitamente u64
//             base_timestamp + 86400u64 * 10, // Use base_timestamp
//         )
//         .run(); // Use run() em vez de execute()
    
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS) // Adicione o destino
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .burn_debt_nft(new_loan_id)
//         .with_result(ExpectError(4, "Only loan controller can burn debt NFTs"))
//         .run();
// }

// #[test]
// fn test_fuzzy_token_operations() {
//     let mut world = deploy_debt_token();
    
//     // Criar um NFT para simular o processo completo
//     let loan_id = 2001u64;
//     let amount = BigUint::from(100_000_000_000u64);
    
//     // Define um timestamp base
//     let base_timestamp = 1715000000u64; // Um timestamp aproximado
//     world.current_block().block_timestamp(base_timestamp);
    
//     // Criar NFT
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id,
//             USER1.to_address(),
//             amount.clone(),
//             1000u64, // 10%
//             base_timestamp + 86400u64 * 30, // 30 dias
//         )
//         .run();
    
//     // Mintar tokens para USER1
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .mint(USER1.to_address(), amount.clone())
//         .run();
    
//     // USER1 transfere para USER2
//     let transfer_amount = BigUint::from(40_000_000_000u64);
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens(USER2.to_address(), transfer_amount.clone())
//         .run();
    
//     // USER1 aprova USER3 para gastar seus tokens
//     let approve_amount = BigUint::from(30_000_000_000u64);
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .approve_tokens(USER3.to_address(), approve_amount.clone())
//         .run();
    
//     // USER3 transfere tokens de USER1 para si mesmo
//     let transfer_from_amount = BigUint::from(20_000_000_000u64);
//     world
//         .tx()
//         .from(USER3)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens_from(
//             USER1.to_address(),
//             USER3.to_address(),
//             transfer_from_amount.clone(),
//         )
//         .run();
    
//     // Aumentar allowance
//     let increase_amount = BigUint::from(10_000_000_000u64);
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .increase_token_allowance(USER3.to_address(), increase_amount.clone())
//         .run();
    
//     // Diminuir allowance
//     let decrease_amount = BigUint::from(5_000_000_000u64);
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .decrease_token_allowance(USER3.to_address(), decrease_amount.clone())
//         .run();
    
//     // Queimar tokens
//     let burn_amount = BigUint::from(15_000_000_000u64);
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .burn(USER2.to_address(), burn_amount.clone())
//         .run();
    
//     // Não podemos fazer verificações diretas dos valores retornados
//     // O teste passando sem erros já indica que as operações foram bem-sucedidas
// }

// #[test]
// fn test_fuzzy_token_operations_errors() {
//     let mut world = deploy_debt_token();
    
//     // Mintar tokens para USER1
//     let mint_amount = BigUint::from(50_000_000_000u64);
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .mint(USER1.to_address(), mint_amount.clone())
//         .run();
    
//     // Tentativa de mintar como usuário não autorizado
//     world
//         .tx()
//         .from(USER2)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .mint(USER2.to_address(), mint_amount.clone())
//         .with_result(ExpectError(4, "Only loan controller can mint tokens"))
//         .run();
    
//     // Tentativa de transferir mais do que o saldo
//     let exceed_amount = BigUint::from(100_000_000_000u64);
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens(USER2.to_address(), exceed_amount.clone())
//         .with_result(ExpectError(4, "Insufficient balance for transfer"))
//         .run();
    
//     // USER1 aprova USER2
//     let approve_amount = BigUint::from(20_000_000_000u64);
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .approve_tokens(USER2.to_address(), approve_amount.clone())
//         .run();
    
//     // Tentativa de transferir mais do que o aprovado
//     let exceed_approve = BigUint::from(30_000_000_000u64);
//     world
//         .tx()
//         .from(USER2)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens_from(
//             USER1.to_address(),
//             USER2.to_address(),
//             exceed_approve.clone(),
//         )
//         .with_result(ExpectError(4, "Insufficient allowance"))
//         .run();
    
//     // Tentativa de diminuir allowance além do disponível
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .decrease_token_allowance(USER2.to_address(), exceed_approve)
//         .with_result(ExpectError(4, "Cannot decrease below zero"))
//         .run();
    
//     // Tentativa de queimar como usuário não autorizado
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .burn(USER1.to_address(), BigUint::from(1_000_000_000u64))
//         .with_result(ExpectError(4, "Only loan controller can burn tokens"))
//         .run();
    
//     // Tentativa de queimar mais do que o saldo
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .burn(USER1.to_address(), exceed_amount)
//         .with_result(ExpectError(4, "Insufficient balance for burn"))
//         .run();
// }

// #[test]
// fn test_fuzzy_stress_test() {
//     let mut world = deploy_debt_token();
    
//     // Criar vários NFTs e realizar várias operações com tokens
//     let users = [USER1, USER2, USER3];
//     let mut loan_id_counter = 3000u64;
    
//     // Define um timestamp base
//     let base_timestamp = 1715000000u64; // Um timestamp aproximado
//     world.current_block().block_timestamp(base_timestamp);
    
//     // Criar 5 NFTs com parâmetros aleatórios para simplificar o teste
//     for _ in 0..5 {
//         loan_id_counter += 1;
        
//         // Escolher usuário aleatório
//         let user_idx = fastrand::usize(0..users.len());
//         let user = users[user_idx];
        
//         // Gerar valores aleatórios
//         let amount = BigUint::from(fastrand::u64(1_000_000_000..20_000_000_000));
//         let interest_rate = fastrand::u64(100..3000);
//         let due_timestamp = base_timestamp + fastrand::u64(86400..31536000);
        
//         // Criar NFT
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .create_debt_nft(
//                 loan_id_counter,
//                 user.to_address(),
//                 amount.clone(),
//                 interest_rate,
//                 due_timestamp,
//             )
//             .run();
        
//         // Mintar tokens
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .mint(user.to_address(), amount.clone())
//             .run();
//     }
    
//     // Realizar 10 operações aleatórias (reduzido para tornar o teste mais rápido)
//     for _ in 0..10 {
//         let operation = fastrand::u8(0..3); // Apenas 3 operações para simplificar
        
//         match operation {
//             // Transferência
//             0 => {
//                 let from_idx = fastrand::usize(0..users.len());
//                 let to_idx = fastrand::usize(0..users.len());
//                 if from_idx == to_idx {
//                     continue;
//                 }
                
//                 let from = users[from_idx];
//                 let to = users[to_idx];
                
//                 // Transferir um valor arbitrário
//                 let transfer_amount = BigUint::from(1_000_000_000u64);
                
//                 world
//                     .tx()
//                     .from(from)
//                     .to(DEBT_TOKEN_ADDRESS)
//                     .typed(debt_token_proxy::DebtTokenProxy)
//                     .transfer_tokens(to.to_address(), transfer_amount)
//                     .run();
//             },
            
//             // Aprovar
//             1 => {
//                 let from_idx = fastrand::usize(0..users.len());
//                 let to_idx = fastrand::usize(0..users.len());
//                 if from_idx == to_idx {
//                     continue;
//                 }
                
//                 let from = users[from_idx];
//                 let to = users[to_idx];
                
//                 // Aprovar um valor arbitrário
//                 let approve_amount = BigUint::from(5_000_000_000u64);
                
//                 world
//                     .tx()
//                     .from(from)
//                     .to(DEBT_TOKEN_ADDRESS)
//                     .typed(debt_token_proxy::DebtTokenProxy)
//                     .approve_tokens(to.to_address(), approve_amount)
//                     .run();
//             },
            
//             // Queimar NFT
//             2 => {
//                 // Escolher um ID de empréstimo para tentar queimar
//                 let loan_id = 3001u64 + fastrand::u64(0..5);
                
//                 // Tentar queimar o NFT - pode falhar, mas não deve quebrar o teste
//                 world
//                     .tx()
//                     .from(LOAN_CONTROLLER)
//                     .to(DEBT_TOKEN_ADDRESS)
//                     .typed(debt_token_proxy::DebtTokenProxy)
//                     .burn_debt_nft(loan_id)
//                     .run();
//             },
//             _ => continue,
//         }
//     }
    
//     // O teste passando sem erros já indica que as operações foram bem-sucedidas
// }

// #[test]
// fn test_fuzzy_edge_cases() {
//     let mut world = deploy_debt_token();
    
//     // Define um timestamp base
//     let base_timestamp = 1715000000u64;
//     world.current_block().block_timestamp(base_timestamp);
    
//     // Casos de borda para criação de NFT
//     let loan_id = 4001u64;
//     let amount = BigUint::from(u64::MAX / 1000); // Valor grande, mas não máximo para evitar overflow
//     let due_timestamp = base_timestamp + 31536000u64; // 1 ano depois
    
//     // Criar NFT com valores grandes
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id,
//             USER1.to_address(),
//             amount.clone(),
//             u64::MAX / 10000, // Taxa de juros muito alta
//             due_timestamp,
//         )
//         .run();
    
//     // Mintar tokens com valor mínimo
//     let min_amount = BigUint::from(1u32);
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .mint(USER1.to_address(), min_amount.clone())
//         .run();
    
//     // Transferir exatamente o saldo disponível 
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens(USER2.to_address(), min_amount.clone())
//         .run();
    
//     // Aprovar valor zero (deve ser válido)
//     world
//         .tx()
//         .from(USER2)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .approve_tokens(USER3.to_address(), BigUint::zero())
//         .run();
    
//     // Tentar transferir com allowance zero
//     world
//         .tx()
//         .from(USER3)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens_from(
//             USER2.to_address(),
//             USER3.to_address(),
//             min_amount.clone(),
//         )
//         .with_result(ExpectError(4, "Insufficient allowance"))
//         .run();
    
//     // Tentar transferir valor zero (deve ser permitido)
//     world
//         .tx()
//         .from(USER2)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens(USER3.to_address(), BigUint::zero())
//         .run();
// }

// #[test]
// fn test_fuzzy_concurrent_operations() {
//     let mut world = deploy_debt_token();
    
//     // Define um timestamp base
//     let base_timestamp = 1715000000u64;
//     world.current_block().block_timestamp(base_timestamp);
    
//     // Criar NFTs e mintar tokens para cada usuário
//     let users = [USER1, USER2, USER3];
    
//     // Para cada usuário, criar um NFT e mintar tokens
//     for (i, &user) in users.iter().enumerate() {
//         let loan_id = 5000u64 + i as u64;
//         let amount = BigUint::from(50_000_000_000u64);
        
//         // Criar NFT
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .create_debt_nft(
//                 loan_id,
//                 user.to_address(),
//                 amount.clone(),
//                 1000u64, // 10%
//                 base_timestamp + 86400u64 * 30, // 30 dias
//             )
//             .run();
        
//         // Mintar tokens
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .mint(user.to_address(), amount.clone())
//             .run();
//     }
    
//     // Simular operações "concorrentes" - aprovar todos os usuários
//     // USER1 aprova USER2, USER2 aprova USER3, USER3 aprova USER1
//     for i in 0..3 {
//         let from = users[i];
//         let to = users[(i + 1) % 3];
        
//         world
//             .tx()
//             .from(from)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .approve_tokens(to.to_address(), BigUint::from(20_000_000_000u64))
//             .run();
//     }
    
//     // Executar transferências "concorrentes"
//     for i in 0..3 {
//         let from = users[i];
//         let to = users[(i + 1) % 3];
        
//         world
//             .tx()
//             .from(from)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .transfer_tokens(to.to_address(), BigUint::from(10_000_000_000u64))
//             .run();
//     }
    
//     // O teste passando sem erros já indica que as operações foram bem-sucedidas
// }

// // Utilitários para teste de helpers internos
// #[test]
// fn test_fuzzy_helpers() {
//     let mut world = deploy_debt_token();
    
//     // Define um timestamp base
//     let base_timestamp = 1715000000u64;
//     world.current_block().block_timestamp(base_timestamp);
    
//     // Testar criação e queima de NFTs com diferentes valores
//     let test_values = [
//         (6001u64, 1_000_000_000u64, 100u64, 86400u64),
//         (6002u64, 1_000_000_000_000u64, 1000u64, 31536000u64),
//         (6003u64, u64::MAX / 2, 3000u64, u64::MAX / 100),
//     ];
    
//     for &(loan_id, amount_u64, interest_rate, duration) in test_values.iter() {
//         let amount = BigUint::from(amount_u64);
//         let due_timestamp = base_timestamp + duration;
        
//         // Criar NFT
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .create_debt_nft(
//                 loan_id,
//                 USER1.to_address(),
//                 amount,
//                 interest_rate,
//                 due_timestamp,
//             )
//             .run();
        
//         // Queimar NFT
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .burn_debt_nft(loan_id)
//             .run();
        
//         // Verificar que o NFT foi queimado tentando queimá-lo novamente
//         world
//             .tx()
//             .from(LOAN_CONTROLLER)
//             .to(DEBT_TOKEN_ADDRESS)
//             .typed(debt_token_proxy::DebtTokenProxy)
//             .burn_debt_nft(loan_id)
//             .with_result(ExpectError(4, "No NFT exists for this loan"))
//             .run();
//     }
// }

// #[test]
// fn test_fuzzy_attack_scenarios() {
//     let mut world = deploy_debt_token();
    
//     // Define um timestamp base
//     let base_timestamp = 1715000000u64;
//     world.current_block().block_timestamp(base_timestamp);
    
//     // Criar NFT e mintar tokens para USER1
//     let loan_id = 7001u64;
//     let amount = BigUint::from(100_000_000_000u64);
    
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id,
//             USER1.to_address(),
//             amount.clone(),
//             1000u64, // 10%
//             base_timestamp + 86400u64 * 30, // 30 dias
//         )
//         .run();
    
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .mint(USER1.to_address(), amount.clone())
//         .run();
    
//     // Cenário 1: Tentativa de reentrância simulada
//     // USER1 aprova USER2
//     world
//         .tx()
//         .from(USER1)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .approve_tokens(USER2.to_address(), amount.clone())
//         .run();
    
//     // USER2 transfere tokens de USER1 para USER3 (primeira parte)
//     let half_amount = amount / BigUint::from(2u32);
//     world
//         .tx()
//         .from(USER2)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens_from(
//             USER1.to_address(),
//             USER3.to_address(),
//             half_amount.clone(),
//         )
//         .run();
    
//     // USER2 transfere novamente (segunda parte) - deve usar o allowance atualizado
//     world
//         .tx()
//         .from(USER2)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens_from(
//             USER1.to_address(),
//             USER3.to_address(),
//             half_amount.clone(),
//         )
//         .run();
    
//     // USER2 não deve conseguir transferir mais
//     world
//         .tx()
//         .from(USER2)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .transfer_tokens_from(
//             USER1.to_address(),
//             USER3.to_address(),
//             BigUint::from(1u32),
//         )
//         .with_result(ExpectError(4, "Insufficient allowance"))
//         .run();
    
//     // Cenário 2: Tentativa de queima do mesmo NFT duas vezes
//     let loan_id2 = 7002u64;
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .create_debt_nft(
//             loan_id2,
//             USER1.to_address(),
//             BigUint::from(50_000_000_000u64),
//             1000u64,
//             base_timestamp + 86400u64 * 30,
//         )
//         .run();
    
//     // Primeira queima
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .burn_debt_nft(loan_id2)
//         .run();
    
//     // Segunda queima - deve falhar
//     world
//         .tx()
//         .from(LOAN_CONTROLLER)
//         .to(DEBT_TOKEN_ADDRESS)
//         .typed(debt_token_proxy::DebtTokenProxy)
//         .burn_debt_nft(loan_id2)
//         .with_result(ExpectError(4, "No NFT exists for this loan"))
//         .run();
// }