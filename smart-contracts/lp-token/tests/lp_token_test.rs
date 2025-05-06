use multiversx_sc_scenario::rust_biguint;
use multiversx_sc_scenario::imports::ManagedAddress;
use multiversx_sc::types::{Address, BigUint, ManagedBuffer};
use multiversx_sc_scenario::{DebugApi, testing_framework::*,};

use lp_token::*;

const WASM_PATH: &'static str = "../output/lp_token.wasm";

struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> lp_token::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub user_address: Address,
    pub contract_wrapper: ContractObjWrapper<lp_token::ContractObj<DebugApi>, ContractObjBuilder>,
}

fn setup_contract<ContractObjBuilder>(
    cf_builder: ContractObjBuilder,
) -> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> lp_token::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    let user_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    let contract_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        cf_builder,
        WASM_PATH,
    );

    // Inicializa o contrato com valores padrão
    blockchain_wrapper.execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
        // Nome: "Test Token", Ticker: "TEST", 18 casas decimais, supply inicial: 1.000.000 tokens
        let initial_supply = BigUint::from(1_000_000u64);
        let token_name = ManagedBuffer::from("Test Token");
        let token_ticker = ManagedBuffer::from("TEST");
        let token_decimals = 18u8;

        sc.init(
            initial_supply,
            token_name,
            token_ticker,
            token_decimals,
        )
    }).assert_ok();

    ContractSetup {
        blockchain_wrapper,
        owner_address,
        user_address,
        contract_wrapper,
    }
}

#[test]
fn test_init() {
    let mut setup = setup_contract(lp_token::contract_obj);
    
    // Testa os valores iniciais do token
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        assert_eq!(
            ManagedBuffer::from("Test Token"),
            sc.get_name()
        );
        assert_eq!(
            ManagedBuffer::from("TEST"),
            sc.get_ticker()
        );
        assert_eq!(
            18u8,
            sc.get_decimals()
        );
        assert_eq!(
            BigUint::from(1_000_000u64),
            sc.total_supply()
        );
        
        // Verifica que o supply inicial foi atribuído ao criador do contrato
        assert_eq!(
            BigUint::from(1_000_000u64),
            sc.balance_of(&ManagedAddress::from_address(&setup.owner_address))
        );
        
        // Verifica que o contrato não está pausado inicialmente
        assert_eq!(false, sc.is_paused());
        
        // Verifica que a taxa inicial é 0%
        assert_eq!(0u64, sc.get_fee_percentage());
    });
}

#[test]
fn test_transfer() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();
    
    // Owner transfere 1000 tokens para o usuário
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(1000u64);
        
        sc.transfer(&user_managed_addr, &amount);
    }).assert_ok();
    
    // Verifica se os saldos foram atualizados corretamente
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let owner_managed_addr = ManagedAddress::from_address(&owner_addr);
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        assert_eq!(
            BigUint::from(999_000u64),  // 1.000.000 - 1.000
            sc.balance_of(&owner_managed_addr)
        );
        assert_eq!(
            BigUint::from(1000u64),
            sc.balance_of(&user_managed_addr)
        );
    });
}

#[test]
fn test_transfer_with_fee() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();
    
    // Configura uma taxa de 5% (500 basis points)
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        sc.set_fee_percentage(500u64); // 5%
    }).assert_ok();
    
    // Owner transfere 1000 tokens para o usuário (com taxa de 5%)
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(1000u64);
        
        sc.transfer(&user_managed_addr, &amount);
    }).assert_ok();
    
    // Verifica se os saldos foram atualizados corretamente considerando a taxa
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let owner_managed_addr = ManagedAddress::from_address(&owner_addr);
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        // Owner deve ter: 1.000.000 - 1.000 + 50 (taxa) = 999.050
        assert_eq!(
            BigUint::from(999_050u64),
            sc.balance_of(&owner_managed_addr)
        );
        
        // Usuário deve ter: 1.000 - 50 (taxa) = 950
        assert_eq!(
            BigUint::from(950u64),
            sc.balance_of(&user_managed_addr)
        );
    });
}

#[test]
fn test_approve_and_transfer_from() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();
    
    // Owner aprova o usuário a gastar 500 tokens
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(500u64);
        
        sc.approve(&user_managed_addr, &amount);
    }).assert_ok();
    
    // Verifica se a allowance foi configurada corretamente
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let owner_managed_addr = ManagedAddress::from_address(&owner_addr);
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        assert_eq!(
            BigUint::from(500u64),
            sc.allowance(&owner_managed_addr, &user_managed_addr)
        );
    });
    
    // Usuário transfere 300 tokens do owner para si mesmo usando transferFrom
    setup.blockchain_wrapper.execute_tx(&user_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let owner_managed_addr = ManagedAddress::from_address(&owner_addr);
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(300u64);
        
        sc.transfer_from(&owner_managed_addr, &user_managed_addr, &amount);
    }).assert_ok();
    
    // Verifica os saldos e allowance após transferFrom
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let owner_managed_addr = ManagedAddress::from_address(&owner_addr);
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        // Owner deve ter 999.700 tokens
        assert_eq!(
            BigUint::from(999_700u64),
            sc.balance_of(&owner_managed_addr)
        );
        
        // Usuário deve ter 300 tokens
        assert_eq!(
            BigUint::from(300u64),
            sc.balance_of(&user_managed_addr)
        );
        
        // Allowance restante deve ser 200 (500 - 300)
        assert_eq!(
            BigUint::from(200u64),
            sc.allowance(&owner_managed_addr, &user_managed_addr)
        );
    });
}

#[test]
fn test_mint_and_burn() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();
    
    // Owner cria 5000 tokens para o usuário
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(5000u64);
        
        sc.mint_endpoint(&user_managed_addr, &amount);
    }).assert_ok();
    
    // Verifica se o mint funcionou corretamente
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        assert_eq!(
            BigUint::from(5000u64),
            sc.balance_of(&user_managed_addr)
        );
        
        // Supply total deve ser 1.005.000
        assert_eq!(
            BigUint::from(1_005_000u64),
            sc.total_supply()
        );
    });
    
    // Owner queima 2000 tokens do usuário
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(2000u64);
        
        sc.burn_endpoint(&user_managed_addr, &amount);
    }).assert_ok();
    
    // Verifica se o burn funcionou corretamente
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        assert_eq!(
            BigUint::from(3000u64),  // 5000 - 2000
            sc.balance_of(&user_managed_addr)
        );
        
        // Supply total deve ser 1.003.000
        assert_eq!(
            BigUint::from(1_003_000u64),
            sc.total_supply()
        );
    });
}




#[test]
fn test_public_mint() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let user_addr = setup.user_address.clone();
    
    // Primeiro, obter uma transação bem-sucedida para comparação
    let ok_result = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        // Não fazer nada, apenas uma consulta
        let _ = sc.get_fee_percentage();
    });
    
    // Usuário solicita o public_mint
    setup.blockchain_wrapper.execute_tx(&user_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        sc.public_mint();
    }).assert_ok(); // Este método assert_ok() parece funcionar
    
    // Verifica se o usuário recebeu 10 tokens
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        assert_eq!(
            BigUint::from(10u64),
            sc.balance_of(&user_managed_addr)
        );
    });
    
    // Tenta solicitar novamente e deve falhar
    let result = setup.blockchain_wrapper.execute_tx(&user_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        sc.public_mint();
    });
    
    // Comparar com o status de sucesso em vez de usar o valor inteiro 0
    assert_ne!(
        result.result_status, 
        ok_result.result_status, 
        "Usuário não deveria conseguir fazer public_mint duas vezes"
    );
}




#[test]
fn test_pause_functionality() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();
    
    // Primeiro, obter uma transação bem-sucedida para comparação
    let ok_result = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        // Não fazer nada, apenas uma consulta
        let _ = sc.is_paused();
    });
    
    // Owner pausa o contrato
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        sc.pause();
    }).assert_ok();
    
    // Verifica se o contrato está pausado
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        assert_eq!(true, sc.is_paused());
    });
    
    // Tenta fazer uma transferência enquanto pausado - deve falhar
    let result = setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(100u64);
        
        sc.transfer(&user_managed_addr, &amount);
    });
    
    // Comparar com o status de sucesso em vez de usar o valor inteiro 0
    assert_ne!(
        result.result_status, 
        ok_result.result_status, 
        "Transferência enquanto pausado deveria falhar"
    );
    
    // Owner despausa o contrato
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        sc.unpause();
    }).assert_ok();
    
    // Tenta transferir novamente - agora deve funcionar
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(100u64);
        
        sc.transfer(&user_managed_addr, &amount);
    }).assert_ok();
}




#[test]
fn test_burn_own() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();
    
    // Primeiro transfere alguns tokens para o usuário
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(500u64);
        
        sc.transfer(&user_managed_addr, &amount);
    }).assert_ok();
    
    // Usuário queima 200 dos seus próprios tokens
    setup.blockchain_wrapper.execute_tx(&user_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let amount = BigUint::from(200u64);
        
        sc.burn_own(&amount);
    }).assert_ok();
    
    // Verifica se o saldo foi atualizado e o supply reduziu
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        assert_eq!(
            BigUint::from(300u64),  // 500 - 200
            sc.balance_of(&user_managed_addr)
        );
        
        // Supply total deve ser 999.800 (1.000.000 - 200)
        assert_eq!(
            BigUint::from(999_800u64),
            sc.total_supply()
        );
    });
}


// Este código faz o seguinte:
// Primeiro, executa uma transação simples que sabemos que terá sucesso (apenas consultando um valor) e armazena seu status
// Para cada teste de operação inválida:
// Executa a transação que esperamos que falhe
// Imprime o status para depuração
// Compara o status com o status de sucesso guardado anteriormente
// Assegura que são diferentes, o que significa que a transação falhou como esperado
#[test]
fn test_invalid_operations() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();

    // Primeiro, obter uma transação bem-sucedida para comparação
    let ok_result = setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        // Não fazer nada, apenas consultar algum valor
        let _ = sc.get_fee_percentage();
    });
    
    println!("Status de uma transação bem-sucedida: {:?}", ok_result.result_status);

    // 1. Teste: Usuário tenta transferir mais do que possui
    let result = setup.blockchain_wrapper.execute_tx(&user_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let owner_managed_addr = ManagedAddress::from_address(&owner_addr);
        let amount = BigUint::from(1000u64); // Usuário não tem tokens inicialmente
        sc.transfer(&owner_managed_addr, &amount);
    });

    // Imprimir o status para debug
    println!("Status da transação 1: {:?}", result.result_status);
    
    // Comparar diretamente os dois status
    assert!(
        result.result_status != ok_result.result_status,
        "Falha esperada: transferência sem saldo deveria ter falhado."
    );

    // 2. Teste: Usuário tenta definir a taxa (apenas owner pode)
    let result = setup.blockchain_wrapper.execute_tx(&user_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        sc.set_fee_percentage(200u64);
    });

    // Imprimir o status para debug
    println!("Status da transação 2: {:?}", result.result_status);
    
    // Comparar diretamente com o status de sucesso
    assert!(
        result.result_status != ok_result.result_status,
        "Usuário não deveria conseguir definir a taxa"
    );

    // 3. Teste: Owner tenta definir taxa acima do limite (10000 basis points)
    let result = setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        sc.set_fee_percentage(15000u64); // 150% - acima do limite
    });

    // Imprimir o status para debug
    println!("Status da transação 3: {:?}", result.result_status);
    
    // Comparar diretamente com o status de sucesso
    assert!(
        result.result_status != ok_result.result_status,
        "Definir taxa acima de 100% deveria falhar"
    );
}


// Função approve precisa ser implementada no contrato principal antes de executar este teste
// Caso contrário, adicione a implementação aqui ou comente este teste
#[test]
fn test_approve() {
    let mut setup = setup_contract(lp_token::contract_obj);
    let owner_addr = setup.owner_address.clone();
    let user_addr = setup.user_address.clone();
    
    // Implementation of approve if not in main contract
    // Adicione ao trait Erc20Token:
    /*
    #[endpoint]
    fn approve(&self, spender: &ManagedAddress, amount: &BigUint) {
        self.require_not_paused();
        
        let caller = self.blockchain().get_caller();
        self.allowances(&caller, spender).set(amount);
        
        self.approve_event(&caller, spender, amount);
    }
    */

    
    // Owner aprova o usuário a gastar 1000 tokens
    setup.blockchain_wrapper.execute_tx(&owner_addr, &setup.contract_wrapper, &rust_biguint!(0), |sc| {
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        let amount = BigUint::from(1000u64);
        
        sc.approve(&user_managed_addr, &amount);
    }).assert_ok();
    
    // Verifica se a allowance foi configurada corretamente
    let _ = setup.blockchain_wrapper.execute_query(&setup.contract_wrapper, |sc| {
        let owner_managed_addr = ManagedAddress::from_address(&owner_addr);
        let user_managed_addr = ManagedAddress::from_address(&user_addr);
        
        assert_eq!(
            BigUint::from(1000u64),
            sc.allowance(&owner_managed_addr, &user_managed_addr)
        );
    });
}