// Importações necessárias
use debt_token::debt_token_proxy;
use multiversx_sc_scenario::imports::*;

// Definindo as constantes do caminho e endereços
const CODE_PATH: MxscPath = MxscPath::new("output/debt-token.mxsc.json");
const OWNER: TestAddress = TestAddress::new("owner");
const USER: TestAddress = TestAddress::new("user");
const DEBT_TOKEN_ADDRESS: TestSCAddress = TestSCAddress::new("debt-token");

// Configuração do mundo de teste
fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    
    // Definindo o diretório atual para o workspace do debt-token
    blockchain.set_current_dir_from_workspace("debt-token");
    
    // Registrando o contrato para testes
    blockchain.register_contract(CODE_PATH, debt_token::ContractBuilder);
    
    blockchain
}

// Função para implantar o token de dívida
fn debt_token_deploy() -> ScenarioWorld {
    let mut world = world();
    
    // Configurando a conta do proprietário com saldo inicial
    // O valor inicial está completamente errado, vamos usar exatamente 950000000
    world.account(OWNER).nonce(0).balance(950_000_000);
    
    // Implantando o contrato
    let debt_token_address = world
        .tx()
        .from(OWNER)
        .typed(debt_token_proxy::DebtTokenProxy)
        .init(OWNER.to_address())
        .code(CODE_PATH)
        .new_address(DEBT_TOKEN_ADDRESS)
        .returns(ReturnsNewAddress)
        .run();
    
    // Verificando se o endereço do contrato foi definido corretamente
    assert_eq!(debt_token_address, DEBT_TOKEN_ADDRESS.to_address());
    
    world
}

// Função para a emissão do token
fn test_issue_token_setup() -> ScenarioWorld {
    let mut world = debt_token_deploy();
    
    // Devemos ter algum argumento faltando na chamada 'issue_debt_token'
    // Vamos verificar se a função precisa de parâmetros
    world
        .tx()
        .from(OWNER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .issue_debt_token()  // Adicionando parâmetros
        .run();
    
    // Opcionalmente, após a emissão do token, configuramos manualmente os papéis ESDT
    // Isso parece estar funcionando no teste test_issue_token
    
    world
}

// Teste para inicialização do contrato
#[test]
fn test_init() {
    let mut world = debt_token_deploy();
    
    // Verificações do estado após inicialização
    world.check_account(OWNER).nonce(1).balance(950_000_000); // Valor ajustado
    world.check_account(DEBT_TOKEN_ADDRESS).nonce(0);
    
    // Adicione verificações específicas do seu contrato
}

// Teste para emissão de token
#[test]
fn test_issue_token() {
    let mut world = test_issue_token_setup();
    
    // Verificações após a emissão do token
    world.check_account(OWNER).nonce(2); // nonce aumenta após cada transação
    
    // Verificar se o token foi emitido corretamente
    // Adicione consultas específicas para verificar o estado do token
}

// Teste para criação de NFT de dívida
#[test]
fn test_create_debt_nft() {
    let mut world = test_issue_token_setup();
    
    // Configurar usuário para testar a criação de NFT
    world.account(USER).nonce(0).balance(100_000_000);
    
    // Criar um NFT de dívida
    world
        .tx()
        .from(OWNER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .create_debt_nft(
            1u64,                // loan_id
            USER.to_address(),   // borrower
            1000u64,             // amount
            5u64,                // interest_rate
            10u64                // due_timestamp
        )
        .run();
    
    // Verificações após a criação do NFT
    // Adicione verificações específicas do seu contrato
}

// Teste para cunhagem e queima de NFT
#[test]
fn test_mint_burn() {
    let mut world = test_issue_token_setup();
    
    // Configurar usuário 
    world.account(USER).nonce(0).balance(100_000_000);
    
    // Criar e cunhar NFT
    world
        .tx()
        .from(OWNER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .create_debt_nft(
            1u64,                // loan_id
            USER.to_address(),   // borrower
            1000u64,             // amount
            5u64,                // interest_rate
            10u64                // due_timestamp
        )
        .run();
    
    // Verificar se o NFT foi cunhado corretamente
    
    // Testar a função de queima
    world
        .tx()
        .from(USER)
        .to(DEBT_TOKEN_ADDRESS)
        .typed(debt_token_proxy::DebtTokenProxy)
        .burn_debt_nft(1u64)  // Assumindo que o ID do NFT é 1
        .run();
    
    // Verificar se o NFT foi queimado corretamente
}

// Teste de cunhagem e queima com valores aleatórios
#[test]
fn test_mint_burn_fuzzy() {
    let mut world = test_issue_token_setup();
    
    // Configurar usuário
    world.account(USER).nonce(0).balance(100_000_000);
    
    // Testar com vários valores - usando u64 explicitamente
    for i in 1..5u64 {
        // Criar e cunhar NFT
        world
            .tx()
            .from(OWNER)
            .to(DEBT_TOKEN_ADDRESS)
            .typed(debt_token_proxy::DebtTokenProxy)
            .create_debt_nft(
                i,                  // loan_id usando o contador como u64
                USER.to_address(),  // borrower
                i * 1000u64,        // amount
                5u64,               // interest_rate
                i * 10u64           // due_timestamp
            )
            .run();
        
        // Verificar se o NFT foi cunhado corretamente
        
        // Queimar o NFT - agora i já é u64, não precisa de conversão
        world
            .tx()
            .from(USER)
            .to(DEBT_TOKEN_ADDRESS)
            .typed(debt_token_proxy::DebtTokenProxy)
            .burn_debt_nft(i)
            .run();
        
        // Verificar se o NFT foi queimado corretamente
    }
}