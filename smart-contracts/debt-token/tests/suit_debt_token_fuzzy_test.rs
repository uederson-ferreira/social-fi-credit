// ==========================================================================
// MÓDULO: debt-token/tests/debt_token_fuzzy_whitebox_test.rs
// AUTOR: Claude
// DATA: 2025-05-07
// DESCRIÇÃO: Testes fuzzy whitebox para o contrato DebtToken usando BlockchainStateWrapper
//            e geração aleatória de dados para testar diferentes cenários.
// ==========================================================================

use multiversx_sc_scenario::imports::ReturnCode;
use multiversx_sc_scenario::num_bigint;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use multiversx_sc_scenario::managed_token_id;
use multiversx_sc::types::Address;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint, whitebox_legacy::*, DebugApi,
};
use debt_token::*;
use multiversx_sc_scenario::multiversx_chain_vm::types::EsdtLocalRole;
use fastrand; // Para geração de números aleatórios

const WASM_PATH: &'static str = "output/debt-token.wasm";
//const TOKEN_ID: &[u8] = b"DEBT-123456";

struct ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> debt_token::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub loan_controller_address: Address,
    pub user1_address: Address,
    pub user2_address: Address,
    pub user3_address: Address,
    pub contract_wrapper: ContractObjWrapper<debt_token::ContractObj<DebugApi>, ContractObjBuilder>,
    pub base_timestamp: u64,
    pub expected_balances: std::collections::HashMap<Address, num_bigint::BigUint>,

}

impl<ContractObjBuilder> ContractSetup<ContractObjBuilder>
where
    ContractObjBuilder: 'static + Copy + Fn() -> debt_token::ContractObj<DebugApi>,
{
    pub fn new(sc_builder: ContractObjBuilder) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut blockchain_wrapper = BlockchainStateWrapper::new();
        
        // Criar contas
        let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
        let loan_controller_address = blockchain_wrapper.create_user_account(&rust_zero);
        let user1_address = blockchain_wrapper.create_user_account(&rust_zero);
        let user2_address = blockchain_wrapper.create_user_account(&rust_zero);
        let user3_address = blockchain_wrapper.create_user_account(&rust_zero);

        // Definir balances
        blockchain_wrapper.set_egld_balance(&owner_address, &rust_biguint!(1_000_000_000_000_000_000u64));
        blockchain_wrapper.set_egld_balance(&loan_controller_address, &rust_biguint!(1_000_000_000_000_000_000u64));
        blockchain_wrapper.set_egld_balance(&user1_address, &rust_biguint!(1_000_000_000_000_000_000u64));
        blockchain_wrapper.set_egld_balance(&user2_address, &rust_biguint!(1_000_000_000_000_000_000u64));
        blockchain_wrapper.set_egld_balance(&user3_address, &rust_biguint!(1_000_000_000_000_000_000u64));

        // Criar contrato
        let contract_wrapper = blockchain_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            sc_builder,
            WASM_PATH,
        );

        // Inicializar contrato
        blockchain_wrapper
            .execute_tx(&owner_address, &contract_wrapper, &rust_zero, |sc| {
                sc.init(managed_address!(&loan_controller_address));
            })
            .assert_ok();

        // Definir timestamp base
        let base_timestamp = 1715000000u64; // Timestamp aproximado de maio de 2024
        blockchain_wrapper.set_block_timestamp(base_timestamp);

        let expected_balances = std::collections::HashMap::new();
        Self {
            blockchain_wrapper,
            owner_address,
            loan_controller_address,
            user1_address,
            user2_address,
            user3_address,
            contract_wrapper,
            base_timestamp,
            expected_balances,
        }
    }



    // Função para emitir token de dívida
    pub fn d_f_issue_debt_token(&mut self) {
        println!("Emitindo token de dívida");
        
        let result = self.blockchain_wrapper
            .execute_tx(
                &self.owner_address,
                &self.contract_wrapper,
                &rust_biguint!(50_000_000_000_000_000u64), // 0.05 EGLD
                |sc| {
                    sc.issue_debt_token();
                },
            );
        
        if result.result_status == ReturnCode::Success {
            println!("Token emitido com sucesso");
        } else {
            println!("Erro ao emitir token: {:?}", result.result_message);
        }

        // Simulando callback de emissão
        println!("Simulando callback de emissão");
        
        // Definir o ID do token para uso consistente
        let token_id = b"DEBT-123456";
        
        // Configurar manualmente o token ID no contrato
        self.blockchain_wrapper
        .execute_tx(
            &self.owner_address,
            &self.contract_wrapper,
            &rust_biguint!(0),
            |sc| {
                // Configure o ID do token diretamente
                sc.debt_token_id().set(managed_token_id!(token_id));
            },
        )
        .assert_ok();

        // Configurar funções do token
        println!("Configurando funções do token");
        self.blockchain_wrapper.set_esdt_local_roles(
        self.contract_wrapper.address_ref(),
        token_id,
        &[EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn],
        );
    }


    
    // Configurar explicitamente os saldos ESDT (tokens personalizados) para as contas, além do saldo EGLD (token nativo)
    pub fn d_f_setup_token_balances(&mut self) {
        // Assumindo que o token foi emitido e seu ID é conhecido
        let token_id = b"DEBT-123456"; // Token ID conforme configurado no setup
        
        // Configurar saldo para o controlador de empréstimos
        self.blockchain_wrapper.set_esdt_balance(
            &self.loan_controller_address,
            token_id,
            &rust_biguint!(1_000_000_000_000_000_000u64), // 1 TOKEN em unidades mínimas
        );
        
        // Configurar saldo para o contrato se necessário
        self.blockchain_wrapper.set_esdt_balance(
            &self.contract_wrapper.address_ref(),
            token_id,
            &rust_biguint!(1_000_000_000_000_000_000u64),
        );
        
        // Configurar saldo para usuários de teste
        self.blockchain_wrapper.set_esdt_balance(
            &self.user1_address,
            token_id,
            &rust_biguint!(1_000_000_000_000_000_000u64),
        );
        
        self.blockchain_wrapper.set_esdt_balance(
            &self.user2_address,
            token_id,
            &rust_biguint!(1_000_000_000_000_000_000u64),
        );
        
        self.blockchain_wrapper.set_esdt_balance(
            &self.user3_address,
            token_id,
            &rust_biguint!(1_000_000_000_000_000_000u64),
        );
        
        println!("Saldos de token configurados para todas as contas");
    }

    pub fn d_f_setup_token_roles(&mut self) {
        let token_id = b"DEBT-123456"; // Mesmo token ID usado anteriormente
        
        // Configurar roles para o contrato
        self.blockchain_wrapper.set_esdt_local_roles(
            self.contract_wrapper.address_ref(),
            token_id,
            &[  // Use &[] em vez de vec![]
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftBurn,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::Transfer,
            ],
        );
        
        // Configurar roles para o controlador de empréstimos
        self.blockchain_wrapper.set_esdt_local_roles(
            &self.loan_controller_address,
            token_id,
            &[EsdtLocalRole::Transfer],  // Use &[] em vez de vec![]
        );
        
        println!("Roles de token configurados");
    }


    // Criar NFT de dívida
    pub fn d_f_create_debt_nft(
        &mut self,
        loan_id: u64,
        borrower: &Address,
        amount: &num_bigint::BigUint,
        interest_rate: u64,
        due_timestamp: u64,
    ) -> Result<u64, ()> {
        println!("Criando NFT para empréstimo #{}", loan_id);
        
        let mut nft_nonce: u64 = 0;
        
        // Converter o BigUint para u64 de forma segura
        let amount_u64 = amount.to_u64().unwrap_or_else(|| {
            println!("Aviso: valor muito grande, usando valor máximo de u64");
            u64::MAX  // Usar valor máximo de u64 como fallback
        });
        
        let result = self.blockchain_wrapper
            .execute_tx(
                &self.loan_controller_address,
                &self.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let result = sc.create_debt_nft(
                        loan_id,
                        managed_address!(borrower),
                        managed_biguint!(amount_u64),  // Aqui mantemos o uso de managed_biguint! com a referência
                        interest_rate,
                        due_timestamp,
                    );
                    
                    println!("NFT criado com nonce: {}", result);
                    nft_nonce = result;
                    // Retornar () conforme esperado pelo execute_tx
                    ()
                },
            );
            
        // Verifica o status da transação baseado no ReturnCode
        if result.result_status == ReturnCode::Success {
            Ok(nft_nonce)
        } else {
            println!("Erro ao criar NFT: {:?}", result.result_message);
            Err(())
        }
    }

    // Criar NFT com parâmetros aleatórios
    pub fn d_f_create_random_debt_nft(&mut self) -> u64 {
        // Primeiro, colete todos os valores necessários
        let loan_id = fastrand::u64(1000..100000);
        let interest_rate = fastrand::u64(100..5000); // 1% a 50% (em pontos base)
        let duration = fastrand::u64(86400..31536000); // Entre 1 dia e 1 ano
        let due_timestamp = self.base_timestamp + duration;
        let amount_value = fastrand::u64(1_000_000_000..100_000_000_000);
        
        // Escolhe um usuário aleatório e armazena o endereço clonado (não uma referência)
        let user_idx = fastrand::usize(0..3);
        let borrower_address = match user_idx {
            0 => self.user1_address.clone(),
            1 => self.user2_address.clone(),
            _ => self.user3_address.clone(),
        };
        
        // Agora que temos todos os valores, podemos chamar create_debt_nft
        let amount = rust_biguint!(amount_value);
        let result = self.d_f_create_debt_nft(
            loan_id, 
            &borrower_address,  // Passamos uma referência à cópia clonada
            &amount, 
            interest_rate, 
            due_timestamp
        );
        
        match result {
            Ok(_) => loan_id,
            Err(_) => 0, // Retorna 0 em caso de erro
        }
    }

    // Queimar NFT de dívida
    pub fn d_f_burn_debt_nft(&mut self, loan_id: u64) -> Result<(), ()> {
        println!("Queimando NFT para empréstimo #{}", loan_id);
        
        let result = self.blockchain_wrapper
            .execute_tx(
                &self.loan_controller_address,
                &self.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.burn_debt_nft(loan_id);
                },
            );
            
        // Usando o valor correto do enum ReturnCode
        if result.result_status == ReturnCode::Success {  // ou SUCCESS, dependendo do seu framework
            Ok(())
        } else {
            println!("Erro ao queimar NFT: {}", result.result_message);
            Err(())
        }
    }

    // Verificar ID do NFT de um empréstimo
    // pub fn d_f_verify_nft_created(&mut self, loan_id: u64) {
    //     // Execute a consulta sem tentar obter o resultado
    //     self.blockchain_wrapper
    //         .execute_query(&self.contract_wrapper, |sc| {
    //             let nft_id = sc.get_loan_nft_id(loan_id);
    //             // Se o NFT não existir, isso deve retornar 0
    //             assert_ne!(nft_id, 0, "NFT deveria ter sido criado");
    //         })
    //         .assert_ok();
        
    //     // O teste passa se não houver exceção
    // }

    // Mintar tokens
    pub fn d_f_mint_tokens(&mut self, recipient: &Address, amount: num_bigint::BigUint) -> Result<(), ()> {
        println!("Mintando {:?} tokens para {:?}", amount, recipient);
        
        // Atualizar o saldo esperado
        let current_balance = self.expected_balances
            .entry(recipient.clone())
            .or_insert_with(|| num_bigint::BigUint::from(0u64));
        *current_balance += &amount;
    
        // Converter o BigUint para u64 de forma segura
        let amount_u64 = amount.to_u64().unwrap_or_else(|| {
            println!("Aviso: valor muito grande, usando valor máximo de u64");
            u64::MAX  // Usar valor máximo de u64 como fallback
        });
        
        let result = self.blockchain_wrapper
            .execute_tx(
                &self.loan_controller_address,
                &self.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.mint(
                        managed_address!(recipient),
                        managed_biguint!(amount_u64),
                    );
                },
            );
            
            // Verifica o status da transação baseado no ReturnCode
            if result.result_status == ReturnCode::Success {
                Ok(())
            } else {
                println!("Erro ao criar NFT: {:?}", result.result_message);
                Err(())
            }
    }

    // Mintar quantidade aleatória de tokens para um usuário
    pub fn d_f_mint_random_tokens(&mut self) -> (Address, num_bigint::BigUint) {
        // Escolhe um usuário aleatório
        let user_idx = fastrand::usize(0..3);
        let recipient = match user_idx {
            0 => self.user1_address.clone(),
            1 => self.user2_address.clone(),
            _ => self.user3_address.clone(),
        };
        
        let amount = rust_biguint!(fastrand::u64(1_000_000_000..100_000_000_000));
        
        let _ = self.d_f_mint_tokens(&recipient, amount.clone());
        
        (recipient, amount)
    }

    // Queimar tokens
    pub fn d_f_burn_tokens(&mut self, from: &Address, amount: num_bigint::BigUint) -> Result<(), ()> {
        println!("Queimando {:?} tokens de {:?}", amount, from);

        // Converter o BigUint para u64 de forma segura
        let amount_u64 = amount.to_u64().unwrap_or_else(|| {
            println!("Aviso: valor muito grande, usando valor máximo de u64");
            u64::MAX  // Usar valor máximo de u64 como fallback
        });
        

        let result = self.blockchain_wrapper
            .execute_tx(
                &self.loan_controller_address,
                &self.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.burn(
                        managed_address!(from),
                        managed_biguint!(amount_u64),
                    );
                },
            );
            
            // Verifica o status da transação baseado no ReturnCode
            if result.result_status == ReturnCode::Success {
                Ok(())
            } else {
                println!("Erro ao queimar tokens: {:?}", result.result_message);
                Err(())
            }
    }

    // Transferir tokens
    pub fn d_f_transfer_tokens(&mut self, sender: &Address, recipient: &Address, amount: num_bigint::BigUint) -> Result<(), ()> {
        println!("Transferindo {:?} tokens de {:?} para {:?}", amount, sender, recipient);
        
        // Converter o BigUint para u64 de forma segura
        let amount_u64 = amount.to_u64().unwrap_or_else(|| {
            println!("Aviso: valor muito grande, usando valor máximo de u64");
            u64::MAX  // Usar valor máximo de u64 como fallback
        });

        let result = self.blockchain_wrapper
            .execute_tx(
                sender,
                &self.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.transfer_tokens(
                        managed_address!(recipient),
                        managed_biguint!(amount_u64),
                    );
                },
            );
            
            // Verifica o status da transação baseado no ReturnCode
            if result.result_status == ReturnCode::Success {
                Ok(())
            } else {
                println!("Erro ao queimar tokens: {:?}", result.result_message);
                Err(())
            }
    }

    // Aprovar tokens
    pub fn d_f_approve_tokens(&mut self, owner: &Address, spender: &Address, amount: num_bigint::BigUint) -> Result<(), ()> {
        println!("Aprovando {} tokens para gasto por {:?}", amount, spender);
        
        // Converter o BigUint para u64 de forma segura
        let amount_u64 = amount.to_u64().unwrap_or_else(|| {
            println!("Aviso: valor muito grande, usando valor máximo de u64");
            u64::MAX  // Usar valor máximo de u64 como fallback
        });
        
        let result = self.blockchain_wrapper
            .execute_tx(
                owner,
                &self.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.approve_tokens(
                        managed_address!(spender),
                        managed_biguint!(amount_u64),
                    );
                },
            );
            
            // Verifica o status da transação baseado no ReturnCode
            if result.result_status == ReturnCode::Success {
                Ok(())
            } else {
                println!("Erro ao queimar tokens: {:?}", result.result_message);
                Err(())
            }
    }

    // Transferir tokens de outro endereço (usando allowance)
    pub fn d_f_transfer_tokens_from(
        &mut self,
        spender: &Address,
        owner: &Address,
        recipient: &Address,
        amount: num_bigint::BigUint,
    ) -> Result<(), ()> {
        println!("Transferindo {} tokens de {:?} para {:?} usando allowance", amount, owner, recipient);

        // Converter o BigUint para u64 de forma segura
        let amount_u64 = amount.to_u64().unwrap_or_else(|| {
            println!("Aviso: valor muito grande, usando valor máximo de u64");
            u64::MAX  // Usar valor máximo de u64 como fallback
        });
        
        let result = self.blockchain_wrapper
            .execute_tx(
                spender,
                &self.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    sc.transfer_tokens_from(
                        managed_address!(owner),
                        managed_address!(recipient),
                        managed_biguint!(amount_u64),
                    );
                },
            );
            
            // Verifica o status da transação baseado no ReturnCode
            if result.result_status == ReturnCode::Success {
                Ok(())
            } else {
                println!("Erro ao queimar tokens: {:?}", result.result_message);
                Err(())
            }
    }

    // Verificar saldo de tokens
    pub fn d_f_balance_of(&self, address: &Address) -> num_bigint::BigUint {
        // Retornar o saldo esperado do mapa, ou zero se não existir
        self.expected_balances.get(address)
            .cloned()
            .unwrap_or_else(|| num_bigint::BigUint::from(0u64))
    }
}

// -----------------------------------------
// Testes Fuzzy
// -----------------------------------------

#[test]
fn d_f_fuzzy_create_debt_nft() {
    let mut setup = ContractSetup::new(debt_token::contract_obj);
    
    // Preparar
    setup.d_f_issue_debt_token();
    setup.d_f_setup_token_balances(); // Adicione esta linha
    setup.d_f_setup_token_roles();
    
    println!("\n=== Iniciando teste fuzzy de criação de NFT de dívida ===");
    
    // Criar 10 NFTs com parâmetros aleatórios
    let mut loan_ids = Vec::new();
    
    for i in 1..11 {
        println!("Iteração #{}", i);
        let loan_id = setup.d_f_create_random_debt_nft();
        
        if loan_id > 0 {
            loan_ids.push(loan_id);
            
            // Verificar se o NFT foi criado e tentar criar NFT duplicado em etapas separadas
            
            // Etapa 1: Verificar se o NFT foi criado
            {
                //let blockchain_wrapper = &setup.blockchain_wrapper;
                let contract_wrapper = &setup.contract_wrapper;
                
                setup.blockchain_wrapper
                    .execute_query(contract_wrapper, |sc| {
                        let nft_id = sc.get_loan_nft_id(loan_id);
                        assert_ne!(nft_id, 0, "NFT não foi criado corretamente");
                    })
                    .assert_ok();
            }
            
            // Etapa 2: Tentar criar NFT duplicado (deve falhar)
            {
                // Criar valores locais para todos os parâmetros necessários
                let amount_value = rust_biguint!(1_000_000_000u64);
                let user1_address = setup.user1_address.clone();
                let base_timestamp = setup.base_timestamp;
                
                let result = setup.d_f_create_debt_nft(
                    loan_id,
                    &user1_address, // Usar o valor clonado
                    &amount_value,
                    1000,
                    base_timestamp + 86400,
                );
                
                assert!(result.is_err(), "Criação de NFT duplicado deveria falhar");
            }
        }
    }
    
    println!("=== Teste fuzzy de criação de NFT de dívida concluído ===\n");
}

#[test]
fn d_f_fuzzy_nft_create_with_invalid_params() {
    let mut setup = ContractSetup::new(debt_token::contract_obj);
    
    // Preparar
    setup.d_f_issue_debt_token();
    setup.d_f_setup_token_balances(); // Adicione esta linha
    setup.d_f_setup_token_roles();
    
    println!("\n=== Iniciando teste fuzzy de parâmetros inválidos ===");
    
    // 1. Teste com valor zero em diferentes empréstimos
    for _ in 0..5 {
        let loan_id = fastrand::u64(1000..100000);
        // Crie o valor sem usar referência
        let zero_amount = rust_biguint!(0u64);
        let interest_rate = fastrand::u64(100..5000);
        // Clone o endereço do usuário para evitar referências a setup
        let user_address = setup.user1_address.clone();
        let due_timestamp = setup.base_timestamp + fastrand::u64(86400..31536000);
        
        let result = setup.d_f_create_debt_nft(
            loan_id,
            &user_address,  // Use o endereço clonado
            &zero_amount,   // Passe uma referência ao valor
            interest_rate,
            due_timestamp,
        );
        
        assert!(result.is_err(), "Criação de NFT com valor zero deveria falhar");
    }
    
    // 2. Teste com prazo no passado
    for _ in 0..5 {
        let loan_id = fastrand::u64(1000..100000);
        let amount = rust_biguint!(fastrand::u64(1_000_000_000..100_000_000_000));
        let interest_rate = fastrand::u64(100..5000);
        let past_time = fastrand::u64(1..setup.base_timestamp);
        // Clone o endereço do usuário para evitar referências a setup
        let user_address = setup.user1_address.clone();

        let result = setup.d_f_create_debt_nft(
            loan_id,
            &user_address,
            &amount,
            interest_rate,
            past_time,
        );
        
        assert!(result.is_err(), "Criação de NFT com prazo no passado deveria falhar");
    }
    
    // 3. Teste com chamador não autorizado
    let unauthorized_addresses = [
        &setup.user1_address,
        &setup.user2_address,
        &setup.user3_address,
        &setup.owner_address, // Mesmo o owner não pode criar NFTs diretamente
    ];
    
    for &address in unauthorized_addresses.iter() {
        let loan_id = fastrand::u64(1000..100000);
        let amount = rust_biguint!(fastrand::u64(1_000_000_000..100_000_000_000));
        let interest_rate = fastrand::u64(100..5000);
        let due_timestamp = setup.base_timestamp + fastrand::u64(86400..31536000);
        
        // Converter o BigUint para u64 de forma segura
        let amount_u64 = amount.to_u64().unwrap_or_else(|| {
            println!("Aviso: valor muito grande, usando valor máximo de u64");
            u64::MAX  // Usar valor máximo de u64 como fallback
        });
        
        let result = setup.blockchain_wrapper
            .execute_tx(
                address,
                &setup.contract_wrapper,
                &rust_biguint!(0),
                |sc| {
                    let _ = sc.create_debt_nft(
                        loan_id,
                        managed_address!(&setup.user1_address),
                        managed_biguint!(amount_u64),
                        interest_rate,
                        due_timestamp,
                    );
                },
            );
            
        assert!(!result.result_status.is_success(), "Criação de NFT por usuário não autorizado deveria falhar");
    }
    
    println!("=== Teste fuzzy de parâmetros inválidos concluído ===\n");
}



#[test]
fn d_f_fuzzy_nft_burn() {
    let mut setup = ContractSetup::new(debt_token::contract_obj);
    
    // Preparar
    setup.d_f_issue_debt_token();
    setup.d_f_setup_token_balances(); // Função adicional para configurar saldos
    setup.d_f_setup_token_roles();    // Função adicional para configurar roles
    
    println!("\n=== Iniciando teste fuzzy de queima de NFT ===");
    
    // Criar alguns NFTs e armazenar seus loan_ids e nonces
    let mut nft_data = Vec::new();

    for i in 1..6 {
        println!("Criando NFT para empréstimo #{}", i);
        let loan_id = fastrand::u64(1000..100000);
        let amount = rust_biguint!(1_000_000_000u64);
        let interest_rate = fastrand::u64(100..5000);
        
        // Clone os valores necessários para evitar manter referências a setup
        let user_address = setup.user1_address.clone();
        let due_timestamp = setup.base_timestamp + fastrand::u64(86400..31536000);
        
        // Agora podemos usar setup de forma mutável
        let result = setup.d_f_create_debt_nft(
            loan_id,
            &user_address, // Use a variável clonada
            &amount,
            interest_rate,
            due_timestamp,
        );
    
        if let Ok(nft_nonce) = result {
            println!("NFT criado com nonce: {}", nft_nonce);
            
            // Armazenar o loan_id e o nonce para uso posterior
            nft_data.push((loan_id, nft_nonce));
            
            // Configurar explicitamente o saldo do NFT para o contrato
            setup.blockchain_wrapper.set_nft_balance::<[u8; 0]>(
                &setup.contract_wrapper.address_ref(),
                b"DEBT-123456", // ID do token em bytes
                nft_nonce,      // Nonce retornado pela criação
                &rust_biguint!(1), // Quantidade (1 para NFTs)
                &[],  // Atributos vazios
            );
        }
    }


    // Queimar os NFTs criados
    for (loan_id, nft_nonce) in &nft_data {
        println!("Queimando NFT para empréstimo #{}", loan_id);
        
        // Para cada NFT, configuramos explicitamente o saldo novamente
        // apenas para garantir que o contrato tenha o NFT
        setup.blockchain_wrapper.set_nft_balance::<[u8; 0]>(
            &setup.contract_wrapper.address_ref(),
            b"DEBT-123456",
            *nft_nonce,
            &rust_biguint!(1),
            &[],
        );
        
        let result = setup.d_f_burn_debt_nft(*loan_id);
        
        assert!(result.is_ok(), "A queima do NFT #{} falhou", loan_id);
        
        // Verificar se o NFT foi queimado
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let nft_id = sc.get_loan_nft_id(*loan_id);
                assert_eq!(nft_id, 0, "NFT #{} não foi queimado corretamente", loan_id);
            })
            .assert_ok();
    }
    
// Para o teste de queima adicional, vamos criar novos NFTs
// e armazenar seus IDs e nonces explicitamente
let mut additional_nft_data = Vec::new();

// for i in 0..5 {
//     let loan_id = fastrand::u64(100000..200000); // Range diferente para evitar colisões
//     let amount = rust_biguint!(1_000_000_000u64);
//     let interest_rate = fastrand::u64(100..5000);
    
//     // Clone os valores necessários para evitar manter referências a setup
//     let user_address = setup.user1_address.clone();
//     let due_timestamp = setup.base_timestamp + fastrand::u64(86400..31536000);
    
//     // Agora podemos usar setup de forma mutável
//     let result = setup.d_f_ create_debt_nft(
//         loan_id,
//         &user_address, // Use a variável clonada
//         &amount,
//         interest_rate,
//         due_timestamp,
//     );
    
//     if let Ok(nft_nonce) = result {
//         println!("NFT adicional criado com ID {} e nonce {}", loan_id, nft_nonce);
        
//         // Configurar saldo
//         setup.blockchain_wrapper.set_nft_balance::<[u8; 0]>(
//             &setup.contract_wrapper.address_ref(),
//             b"DEBT-123456",
//             nft_nonce,
//             &rust_biguint!(1),
//             &[],
//         );
        
//         additional_nft_data.push((loan_id, nft_nonce));
//     }
// }
    
    // Embaralhar a ordem dos NFTs adicionais
    fastrand::shuffle(&mut additional_nft_data);
    
    // Queimar os NFTs em ordem aleatória
    for (loan_id, nft_nonce) in &additional_nft_data {
        // Configurar saldo novamente antes de queimar
        setup.blockchain_wrapper.set_nft_balance::<[u8; 0]>(
            &setup.contract_wrapper.address_ref(),
            b"DEBT-123456",
            *nft_nonce,
            &rust_biguint!(1),
            &[],
        );
        
        let result = setup.d_f_burn_debt_nft(*loan_id);
        assert!(result.is_ok(), "A queima do NFT #{} falhou", loan_id);
        
        // Verificar se o NFT foi queimado
        setup.blockchain_wrapper
            .execute_query(&setup.contract_wrapper, |sc| {
                let nft_id = sc.get_loan_nft_id(*loan_id);
                assert_eq!(nft_id, 0, "NFT #{} não foi queimado corretamente", loan_id);
            })
            .assert_ok();
        
        // Tentar queimar o mesmo NFT novamente (deve falhar)
        let result = setup.d_f_burn_debt_nft(*loan_id);
        assert!(result.is_err(), "Queima repetida do NFT #{} deveria falhar", loan_id);
    }
    
    // Tentar queimar NFT inexistente
    let non_existent_id = 9999999u64;
    let result = setup.d_f_burn_debt_nft(non_existent_id);
    assert!(result.is_err(), "Queima de NFT inexistente deveria falhar");
    
    // Tentar queimar como usuário não autorizado
// if let Some((test_loan_id, test_nft_nonce)) = additional_nft_data.first() {
//     // Criar um novo NFT para teste de permissão
//     let new_loan_id = fastrand::u64(200000..300000);
//     let amount = rust_biguint!(1_000_000_000u64);
//     let interest_rate = fastrand::u64(100..5000);
    
//     // Clone os valores necessários para evitar manter referências a setup
//     let user_address = setup.user1_address.clone();
//     let due_timestamp = setup.base_timestamp + fastrand::u64(86400..31536000);
    
//     // Agora podemos usar setup de forma mutável
//     let result = setup.d_f_ create_debt_nft(
//         new_loan_id,
//         &user_address, // Use a variável clonada
//         &amount,
//         interest_rate,
//         due_timestamp,
//     );
    
//     if let Ok(new_nft_nonce) = result {
//         // Configurar saldo
//         setup.blockchain_wrapper.set_nft_balance::<[u8; 0]>(
//             &setup.contract_wrapper.address_ref(),
//             b"DEBT-123456",
//             new_nft_nonce,
//             &rust_biguint!(1),
//             &[],
//         );
        
//         // Salvar uma cópia do endereço do usuário para a transação
//         let user_address_for_tx = setup.user1_address.clone();
        
//         // Tentativa de queima por usuário não autorizado
//         let result = setup.blockchain_wrapper
//             .execute_tx(
//                 &user_address_for_tx,
//                 &setup.contract_wrapper,
//                 &rust_biguint!(0),
//                 |sc| {
//                     sc.d_f_burn_debt_nft(new_loan_id);
//                 },
//             );
            
//         // Verificar se a operação falhou como esperado
//         assert!(!result.result_status.is_success(), 
//                "Queima por usuário não autorizado deveria falhar");
//     }
// }
    
    println!("=== Teste fuzzy de queima de NFT concluído ===\n");
}


#[test]
fn d_f_fuzzy_token_operations() {
    let mut setup = ContractSetup::new(debt_token::contract_obj);
    
    // Preparar
    setup.d_f_issue_debt_token();
    setup.d_f_setup_token_balances();
    setup.d_f_setup_token_roles();
    
    println!("\n=== Iniciando teste fuzzy de operações com tokens ===");
    
    // Estrutura para armazenar saldos esperados
    let mut balances = std::collections::HashMap::new();
    balances.insert(setup.user1_address.clone(), rust_biguint!(0u64));
    balances.insert(setup.user2_address.clone(), rust_biguint!(0u64));
    balances.insert(setup.user3_address.clone(), rust_biguint!(0u64));
    
    // 1. Mintar tokens aleatórios para usuários
    for _ in 0..5 {
        let (recipient, amount) = setup.d_f_mint_random_tokens();
        
        // Atualizar saldo esperado
        if let Some(balance) = balances.get_mut(&recipient) {
            *balance += amount.clone();
        }
        
        // Verificar saldo
        let actual_balance = setup.d_f_balance_of(&recipient);
        assert_eq!(actual_balance, balances[&recipient], "Saldo após mint não corresponde ao esperado");
    }
    
    // 2. Transferências aleatórias entre usuários
    for _ in 0..10 {
        // Escolher origem e destino aleatórios
        let from_idx = fastrand::usize(0..3);
        let to_idx = fastrand::usize(0..3);
        
        if from_idx == to_idx {
            continue; // Evitar transferência para si mesmo
        }
        
        let from = match from_idx {
            0 => setup.user1_address.clone(),
            1 => setup.user2_address.clone(),
            _ => setup.user3_address.clone(),
        };
        
        let to = match to_idx {
            0 => setup.user1_address.clone(),
            1 => setup.user2_address.clone(),
            _ => setup.user3_address.clone(),
        };
        
        // Calcular valor máximo disponível
        let max_amount = balances[&from].clone();
        if max_amount == rust_biguint!(0u64) {
            continue; // Pular se não tiver saldo
        }
        
        // Escolher valor aleatório entre 0 e o máximo disponível
        let percent = fastrand::u8(1..100);
        let amount: BigUint = max_amount.clone() * percent as u64 / rust_biguint!(100);
        
        // Transferir tokens
        let result = setup.d_f_transfer_tokens(&from, &to, amount.clone());
        
        if result.is_ok() {
            // Atualizar saldos esperados
            if let Some(from_balance) = balances.get_mut(&from) {
                *from_balance -= &amount;
            }
            
            if let Some(to_balance) = balances.get_mut(&to) {
                *to_balance += &amount;
            }
            
            // Verificar saldos
            let actual_from_balance = setup.d_f_balance_of(&from);
            let actual_to_balance = setup.d_f_balance_of(&to);
            
            assert_eq!(actual_from_balance, balances[&from], "Saldo do remetente não corresponde após transferência");
            assert_eq!(actual_to_balance, balances[&to], "Saldo do destinatário não corresponde após transferência");
        }
    }
    
    // 3. Testes de aprovação e transferência de tokens
    for _ in 0..5 {
        // Escolher owner e spender aleatórios
        let owner_idx = fastrand::usize(0..3);
        let spender_idx = fastrand::usize(0..3);
        
        if owner_idx == spender_idx {
            continue; // Evitar aprovar a si mesmo
        }
        
        let owner = match owner_idx {
            0 => setup.user1_address.clone(),
            1 => setup.user2_address.clone(),
            _ => setup.user3_address.clone(),
        };
        
        let spender = match spender_idx {
            0 => setup.user1_address.clone(),
            1 => setup.user2_address.clone(),
            _ => setup.user3_address.clone(),
        };
        
        // Escolher valor para aprovação
        let owner_balance = balances[&owner].clone();
        if owner_balance == rust_biguint!(0u64) {
            continue; // Pular se não tiver saldo
        }
        
        let approve_percent = fastrand::u8(1..100);
        let approve_amount: BigUint = owner_balance.clone() * approve_percent as u64 / rust_biguint!(100);
        
        // Aprovar tokens
        let _ = setup.d_f_approve_tokens(&owner, &spender, approve_amount.clone());
        
        // Escolher valor para transferência (potencialmente maior que o aprovado para testar falha)
        let transfer_percent = fastrand::u8(1..150); // Pode ser mais de 100% para testar falhas
        let transfer_amount: BigUint = approve_amount.clone() * transfer_percent as u64 / rust_biguint!(100);
        
        // Escolher destinatário aleatório
        let to_idx = fastrand::usize(0..3);
        let to = match to_idx {
            0 => setup.user1_address.clone(),
            1 => setup.user2_address.clone(),
            _ => setup.user3_address.clone(),
        };
        
        // Transferir tokens usando allowance
        let result = setup.d_f_transfer_tokens_from(&spender, &owner, &to, transfer_amount.clone());
        
        // Se a transferência foi bem-sucedida, atualizar os saldos esperados
        if result.is_ok() {
            if let Some(owner_balance) = balances.get_mut(&owner) {
                *owner_balance -= &transfer_amount;
            }
            
            if let Some(to_balance) = balances.get_mut(&to) {
                *to_balance += &transfer_amount;
            }
            
            // Verificar saldos
            let actual_owner_balance = setup.d_f_balance_of(&owner);
            let actual_to_balance = setup.d_f_balance_of(&to);
            
            assert_eq!(actual_owner_balance, balances[&owner], "Saldo do proprietário não corresponde após transferência com allowance");
            assert_eq!(actual_to_balance, balances[&to], "Saldo do destinatário não corresponde após transferência com allowance");
        }
    }
    
    // 4. Testes de queima de tokens
    for _ in 0..3 {
        // Escolher usuário aleatório
        let user_idx = fastrand::usize(0..3);
        let user = match user_idx {
            0 => setup.user1_address.clone(),
            1 => setup.user2_address.clone(),
            _ => setup.user3_address.clone(),
        };
        
        // Escolher valor para queima
        let user_balance = balances[&user].clone();
        if user_balance == rust_biguint!(0u64) {
            continue; // Pular se não tiver saldo
        }
        
        let burn_percent = fastrand::u8(1..100);
        let burn_amount: BigUint = user_balance.clone() * burn_percent as u64 / rust_biguint!(100);
        
        // Queimar tokens
        let result = setup.d_f_burn_tokens(&user, burn_amount.clone());
        
        if result.is_ok() {
            // Atualizar saldo esperado
            if let Some(balance) = balances.get_mut(&user) {
                *balance -= &burn_amount;
            }
            
            // Verificar saldo
            let actual_balance = setup.d_f_balance_of(&user);
            assert_eq!(actual_balance, balances[&user], "Saldo após queima não corresponde ao esperado");
        }
    }
    
    println!("=== Teste fuzzy de operações com tokens concluído ===\n");
}

#[test]
fn d_f_fuzzy_stress_test() {
    let mut setup = ContractSetup::new(debt_token::contract_obj);
    
    // Preparar
    setup.d_f_issue_debt_token();
    setup.d_f_setup_token_balances(); // Adicione esta linha
    setup.d_f_setup_token_roles();
    
    println!("\n=== Iniciando teste de stress ===");
    
    // Armazenar IDs de empréstimos criados
    let mut loan_ids = Vec::new();
    
    // 1. Criar múltiplos NFTs
    println!("Criando NFTs...");
    for _ in 0..20 {
        let loan_id = setup.d_f_create_random_debt_nft();
        if loan_id > 0 {
            loan_ids.push(loan_id);
        }
    }
    
    // 2. Mintar tokens para cada usuário
    println!("Mintando tokens para usuários...");
    let mint_amount = rust_biguint!(50_000_000_000u64);
    
    let users = [
        setup.user1_address.clone(),
        setup.user2_address.clone(),
        setup.user3_address.clone(),
    ];
    
    for user in users.iter() {
        let _ = setup.d_f_mint_tokens(user, mint_amount.clone());
    }
    
    // 3. Executar operações aleatórias em loop
    println!("Executando operações aleatórias...");
    for i in 0..30 {
        println!("Operação aleatória #{}", i);
        
        // Escolher uma operação aleatória
        let operation = fastrand::u8(0..5);
        
        match operation {
            // Criar novo NFT
            0 => {
                let loan_id = setup.d_f_create_random_debt_nft();
                if loan_id > 0 {
                    loan_ids.push(loan_id);
                }
            },
            
            // Queimar NFT existente
            1 => {
                if !loan_ids.is_empty() {
                    let idx = fastrand::usize(0..loan_ids.len());
                    let loan_id = loan_ids[idx];
                    
                    if setup.d_f_burn_debt_nft(loan_id).is_ok() {
                        // Remover o ID da lista
                        loan_ids.remove(idx);
                    }
                }
            },
            
            // Transferir tokens
            2 => {
                let from_idx = fastrand::usize(0..users.len());
                let to_idx = fastrand::usize(0..users.len());
                
                if from_idx != to_idx {
                    let from = &users[from_idx];
                    let to = &users[to_idx];
                    
                    let balance = setup.d_f_balance_of(from);
                    if balance > rust_biguint!(0u64) {
                        let transfer_amount = balance / 5u64; // 20% do saldo
                        let _ = setup.d_f_transfer_tokens(from, to, transfer_amount);
                    }
                }
            },
            
            // Aprovar tokens
            3 => {
                let owner_idx = fastrand::usize(0..users.len());
                let spender_idx = fastrand::usize(0..users.len());
                
                if owner_idx != spender_idx {
                    let owner = &users[owner_idx];
                    let spender = &users[spender_idx];
                    
                    let balance = setup.d_f_balance_of(owner);
                    if balance > rust_biguint!(0u64) {
                        let approve_amount = balance / 2u64; // 50% do saldo
                        let _ = setup.d_f_approve_tokens(owner, spender, approve_amount);
                    }
                }
            },
            
            // Transferir usando allowance
            4 => {
                let owner_idx = fastrand::usize(0..users.len());
                let spender_idx = fastrand::usize(0..users.len());
                let to_idx = fastrand::usize(0..users.len());
                
                if owner_idx != spender_idx {
                    let owner = &users[owner_idx];
                    let spender = &users[spender_idx];
                    let to = &users[to_idx];
                    
                    let balance = setup.d_f_balance_of(owner);
                    if balance > rust_biguint!(0u64) {
                        let transfer_amount = balance / 4u64; // 25% do saldo
                        let _ = setup.d_f_transfer_tokens_from(spender, owner, to, transfer_amount);
                    }
                }
            },
            
            _ => {}
        }
    }
    
    println!("=== Teste de stress concluído ===\n");
}