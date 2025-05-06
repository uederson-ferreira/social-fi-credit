    // ==========================================================================
    // ARQUIVO: integrated_system_test.rs
    // Descrição: Testes integrados para todo o sistema de empréstimos
    // ==========================================================================
    #[cfg(test)]
    
    use std::borrow::Borrow;
    use multiversx_sc_scenario::imports::BigUint;
    //use multiversx_sc::proxy_imports::BigUint;
    use multiversx_sc_scenario::DebugApi;
    use multiversx_sc_scenario::{managed_address, managed_biguint, rust_biguint, testing_framework::BlockchainStateWrapper};
    use multiversx_sc::types::Address;
    use multiversx_sc::proxy_imports::ManagedBuffer;
    use multiversx_sc_scenario::testing_framework::ContractObjWrapper;

    use loan_controller::*;
    use reputation_score::*;
    use debt_token::*;
    use liquidity_pool::*;
    use lp_token::*;  // Use the real LP token module
    

    const LOAN_CONTROLLER_WASM_PATH: &str = "output/loan-controller.wasm";
    const REPUTATION_SCORE_WASM_PATH: &str = "output/reputation-score.wasm";
    const DEBT_TOKEN_WASM_PATH: &str = "output/debt-token.wasm";
    const LIQUIDITY_POOL_WASM_PATH: &str = "output/liquidity-pool.wasm";
    const LP_TOKEN_WASM_PATH: &str = "output/lp-token.wasm";
    
    // Estrutura para configuração dos testes integrados
    struct IntegratedSystemSetup {
        pub blockchain_wrapper: BlockchainStateWrapper,
        pub owner_address: Address,
        pub provider1_address: Address,
        pub provider2_address: Address,
        pub borrower1_address: Address,
        pub borrower2_address: Address,
        pub loan_controller_wrapper: ContractObjWrapper<loan_controller::ContractObj<DebugApi>, fn() -> loan_controller::ContractObj<DebugApi>>,
        pub reputation_score_wrapper: ContractObjWrapper<reputation_score::ContractObj<DebugApi>, fn() -> reputation_score::ContractObj<DebugApi>>,
        pub debt_token_wrapper: ContractObjWrapper<debt_token::ContractObj<DebugApi>, fn() -> debt_token::ContractObj<DebugApi>>,
        pub liquidity_pool_wrapper: ContractObjWrapper<liquidity_pool::ContractObj<DebugApi>, fn() -> liquidity_pool::ContractObj<DebugApi>>,
        pub lp_token_wrapper: ContractObjWrapper<lp_token::ContractObj<DebugApi>, fn() -> lp_token::ContractObj<DebugApi>>,
    }


    // Antes de setup_integrated_system, adicione essas funções para cada contrato
    fn loan_controller_obj() -> loan_controller::ContractObj<DebugApi> {
        loan_controller::contract_obj()
    }

    fn reputation_score_obj() -> reputation_score::ContractObj<DebugApi> {
        reputation_score::contract_obj()
    }

    fn debt_token_obj() -> debt_token::ContractObj<DebugApi> {
        debt_token::contract_obj()
    }

    fn liquidity_pool_obj() -> liquidity_pool::ContractObj<DebugApi> {
        liquidity_pool::contract_obj()
    }

    fn lp_token_obj() -> lp_token::ContractObj<DebugApi> {
        lp_token::contract_obj()
    }

    // Função de configuração para os testes integrados
    fn setup_integrated_system() -> IntegratedSystemSetup {
        let rust_zero = rust_biguint!(0u64);
        let mut blockchain_wrapper = BlockchainStateWrapper::new();
        
        // Criar contas
        let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
        let provider1_address = blockchain_wrapper.create_user_account(&rust_biguint!(200000));
        let provider2_address = blockchain_wrapper.create_user_account(&rust_biguint!(300000));
        let borrower1_address = blockchain_wrapper.create_user_account(&rust_biguint!(10000));
        let borrower2_address = blockchain_wrapper.create_user_account(&rust_biguint!(15000));
        
        // Deploy dos contratos - corrigindo o tipo dos wrappers
        let lp_token_wrapper = blockchain_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            lp_token_obj as fn() -> lp_token::ContractObj<DebugApi>, // Usando a função nomeada
            LP_TOKEN_WASM_PATH,
        );
        
        let debt_token_wrapper = blockchain_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            debt_token_obj as fn() -> debt_token::ContractObj<DebugApi>, // Usando a função nomeada
            DEBT_TOKEN_WASM_PATH,
        );
        
        let reputation_score_wrapper = blockchain_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            reputation_score_obj as fn() -> reputation_score::ContractObj<DebugApi>,// Usando a função nomeada
            REPUTATION_SCORE_WASM_PATH,
        );
        
        let liquidity_pool_wrapper = blockchain_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            liquidity_pool_obj as fn() -> liquidity_pool::ContractObj<DebugApi>,
            LIQUIDITY_POOL_WASM_PATH,
        );
        
        // coerção explícita para ponteiro fn() -> ContractObj<DebugApi>
        let loan_controller_wrapper = blockchain_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            loan_controller_obj as fn() -> loan_controller::ContractObj<DebugApi>,
            LOAN_CONTROLLER_WASM_PATH,
        );

            
        // Inicializar contratos - corrigindo as assinaturas
        // 1. Inicializar DebtToken
        blockchain_wrapper
            .execute_tx(&owner_address, &debt_token_wrapper, &rust_zero, |sc| {
                sc.init(managed_address!(&loan_controller_wrapper.address_ref()));
            })
            .assert_ok();
        
        // 2. Inicializar ReputationScore
        blockchain_wrapper
            .execute_tx(&owner_address, &reputation_score_wrapper, &rust_zero, |sc| {
                sc.init(
                    300u64, // min_score
                    900u64  // max_score
                );
            })
            .assert_ok();
        
        // Definir manualmente as pontuações para os borrowers para os testes
        blockchain_wrapper
            .execute_tx(&owner_address, &reputation_score_wrapper, &rust_zero, |sc| {
                sc.user_score(managed_address!(&borrower1_address)).set(700u64);
                sc.user_score(managed_address!(&borrower2_address)).set(600u64);
            })
            .assert_ok();
        
        // 3. Inicializar LiquidityPool - corrigir os parâmetros conforme a assinatura atual
        blockchain_wrapper
            .execute_tx(&owner_address, &liquidity_pool_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_address!(&loan_controller_wrapper.address_ref()),
                    managed_biguint!(1000),  // min_deposit_amount
                    1000u64                  // annual_yield_percentage (10%)
                );
                
                // Definir manualmente os endereços dos tokens relacionados
                sc.debt_token_address().set(managed_address!(&debt_token_wrapper.address_ref()));
                sc.lp_token_address().set(managed_address!(&lp_token_wrapper.address_ref()));
            })
            .assert_ok();
        
        // 4. Inicializar LP Token - usando o contrato real
        blockchain_wrapper
            .execute_tx(&owner_address, &lp_token_wrapper, &rust_zero, |sc| {
                // Inicializando o token LP real
                sc.init(
                    managed_biguint!(0), // initial_supply
                    ManagedBuffer::from("Liquidity Pool Token"), // token_name
                    ManagedBuffer::from("LPT"), // token_ticker
                    18u8, // token_decimals
                );
            })
            .assert_ok();
        
        // 5. Inicializar LoanController
        blockchain_wrapper
            .execute_tx(&owner_address, &loan_controller_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_address!(&reputation_score_wrapper.address_ref()),
                    500u64, // Pontuação mínima necessária
                    1000u64, // Taxa de juros base (10% = 1000 pontos base)
                    managed_biguint!(10_000), // Valor base do empréstimo
                );
            })
            .assert_ok();
        
        // Configurar endereços importantes no controlador de empréstimos
        blockchain_wrapper

            .execute_tx(&owner_address, &loan_controller_wrapper, &rust_zero, |sc| {
                    // 1) Tipo explícito para ManagedBuffer<DebugApi>
                    //let liquidity_pool_key: ManagedBuffer<DebugApi> = ManagedBuffer::from("liquidity_pool_address");
                    //let debt_token_key: ManagedBuffer<DebugApi> = ManagedBuffer::from("debt_token_address");
            
                    // 2) Use storage_raw e as_slice()
                    // depois: usa os novos mappers
                    sc.liquidation_threshold().set(8500u64);
                    sc.liquidation_penalty().set(1000u64);

                    
                })
                .assert_ok();
        
        IntegratedSystemSetup {
            blockchain_wrapper,
            owner_address,
            provider1_address,
            provider2_address,
            borrower1_address,
            borrower2_address,
            loan_controller_wrapper,
            reputation_score_wrapper,
            debt_token_wrapper,
            liquidity_pool_wrapper,
            lp_token_wrapper,
        }
    }

    // Teste integrado do fluxo completo de empréstimo
    #[test]
    fn test_full_loan_lifecycle_integrated() {
        let mut setup = setup_integrated_system();
        
        // Etapa 1: Provedores adicionam liquidez ao pool
        setup.blockchain_wrapper
            .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
                sc.deposit_funds();
            })
            .assert_ok();
        
        // Simular emissão de tokens LP pelo LiquidityPool
        setup.blockchain_wrapper
            .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
                sc.mint_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(100000));
            })
            .assert_ok();
        
        setup.blockchain_wrapper
            .execute_tx(&setup.provider2_address, &setup.liquidity_pool_wrapper, &rust_biguint!(50000), |sc| {
                sc.deposit_funds();
            })
            .assert_ok();
        
        // Simular emissão de tokens LP
        setup.blockchain_wrapper
            .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
                sc.mint_endpoint(&managed_address!(&setup.provider2_address), &managed_biguint!(50000));
            })
            .assert_ok();
        
        // Verificar liquidez total
        setup.blockchain_wrapper
            .execute_query(&setup.liquidity_pool_wrapper, |sc| {
                assert_eq!(sc.total_liquidity().get(), managed_biguint!(150000));
            })
            .assert_ok();
        
        // Etapa 2: Tomador solicita verificação de pontuação
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.request_reputation_check();
            })
            .assert_ok();
        
        // Simular resposta da verificação de pontuação
        setup.blockchain_wrapper
            .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.reputation_check_callback(
                    managed_address!(&setup.borrower1_address),
                    700u64
                );
            })
            .assert_ok();
        
        // Etapa 3: Tomador solicita empréstimo
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                let loan_id = sc.request_loan_standard();
                assert_eq!(loan_id, 1u64);
            })
            .assert_ok();
        
        // Verificar detalhes do empréstimo
        setup.blockchain_wrapper
            .execute_query(&setup.loan_controller_wrapper, |sc| {
                let loan = sc.loans(1u64).get();
                
                // Com pontuação 700, taxa deve ser reduzida
                // Taxa base: 1000 (10%)
                // Redução: 1000 * (1 - (700/1000) * 0.8) = 1000 * (1 - 0.56) = 1000 * 0.44 = 440
                assert_eq!(loan.interest_rate, 440u64);
                
                // Verificar valor do empréstimo
                // Valor base: 10000
                // Ajuste: 10000 * (1 + (700/1000) * 0.5) = 10000 * 1.35 = 13500
                assert_eq!(loan.amount, managed_biguint!(13500));
                
                // Valor de repagamento: 13500 + (13500 * 440 / 10000) = 13500 + 594 = 14094
                assert_eq!(loan.repayment_amount, managed_biguint!(14094));
            })
            .assert_ok();
        
        // Etapa 4: LoanController obtém fundos da LiquidityPool
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
                sc.borrow();
            })
            .assert_ok();
        
        // Etapa 5: LiquidityPool emite tokens de dívida
        setup.blockchain_wrapper
            .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
                sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
            })
            .assert_ok();
        
        // Verificar saldo de tokens de dívida
        setup.blockchain_wrapper
            .execute_query(&setup.debt_token_wrapper, |sc| {
                assert_eq!(sc.balance_of(managed_address!(&setup.borrower1_address)), managed_biguint!(13500));
            })
            .assert_ok();
        
        // Etapa 7: Tomador paga parte do empréstimo
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(5000), |sc| {
                // Simular queima de tokens de dívida
                sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower1_address), managed_biguint!(5000));
                
                sc.repay_endpoint();
            })
            .assert_ok();
        
        // Verificar atualização dos tokens de dívida
        setup.blockchain_wrapper
        .execute_tx(&setup.lp_token_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn_endpoint(&managed_address!(&setup.borrower1_address), &managed_biguint!(5000));
        })
        .assert_ok();
    
        
        // Verificar saldo atualizado
        setup.blockchain_wrapper
            .execute_query(&setup.debt_token_wrapper, |sc| {
                assert_eq!(sc.balance_of(managed_address!(&setup.borrower1_address)), managed_biguint!(8500));
            })
            .assert_ok();
        
        // Etapa 8: Tomador paga o restante do empréstimo
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(9094), |sc| {
                // Simular queima de tokens de dívida
                sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower1_address), managed_biguint!(8500));
                
                sc.repay_endpoint();
            })
            .assert_ok();
        
        // LiquidityPool queima o restante dos tokens de dívida
        setup.blockchain_wrapper
        .execute_tx(&setup.lp_token_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.burn_endpoint(&managed_address!(&setup.borrower1_address), &managed_biguint!(5000));
        })
        .assert_ok();
    
        
        // Etapa 9: LoanController atualiza o status do empréstimo
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                // Alterando para usar um método existente no contrato
                sc.mark_loan_defaulted(1u64);
                
                // Verificar status
                let loan = sc.loans(1u64).get();
                assert_eq!(loan.status, LoanStatus::Repaid);
            })
            .assert_ok();
        
        // Etapa 10: ReputationScore atualiza a pontuação do tomador
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.reputation_score_wrapper, &rust_biguint!(0), |sc| {
                sc.update_score(managed_address!(&setup.borrower1_address), 50); // Aumento por pagamento em dia
                
                // Verificar nova pontuação
                let new_score = sc.get_user_score(managed_address!(&setup.borrower1_address));
                assert_eq!(new_score, 750u64); // 700 + 50
            })
            .assert_ok();
        
        // Etapa 11: Proveedor retira parte de sua liquidez
        setup.blockchain_wrapper
            .execute_tx(&setup.provider1_address, &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
                // Simular queima de tokens LP
                sc.burn_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(40000));
            })
            .assert_ok();
        
        setup.blockchain_wrapper
            .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
                sc.withdraw(managed_biguint!(40000));
            })
            .assert_ok();
    }

    // Teste integrado de empréstimo com inadimplência
    #[test]
    fn test_loan_default_integrated() {
        let mut setup = setup_integrated_system();
        
        // Etapa 1: Provedores adicionam liquidez
        setup.blockchain_wrapper
            .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
                sc.deposit_funds();
            })
            .assert_ok();
        
        // Simular emissão de tokens LP
        setup.blockchain_wrapper
            .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
                sc.mint_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(100000));
            })
            .assert_ok();
        
        // Etapa 2: Tomador solicita empréstimo
        // Primeiro, verificar pontuação
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.request_reputation_check();
            })
            .assert_ok();
        
        // Simular resposta da verificação de pontuação
        setup.blockchain_wrapper
            .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.reputation_check_callback(
                    managed_address!(&setup.borrower2_address),
                    600u64
                );
            })
            .assert_ok();
        
        // Solicitar empréstimo
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                let loan_id = sc.request_loan_standard();
                assert_eq!(loan_id, 1u64);
            })
            .assert_ok();
        
        // Verificar detalhes do empréstimo
        setup.blockchain_wrapper
            .execute_query(&setup.loan_controller_wrapper, |sc| {
                let loan = sc.loans(1u64).get();
                
                // Com pontuação 600, taxa deve ser reduzida menos que no teste anterior
                // Taxa base: 1000 (10%)
                // Redução: 1000 * (1 - (600/1000) * 0.8) = 1000 * (1 - 0.48) = 1000 * 0.52 = 520
                assert_eq!(loan.interest_rate, 520u64);
                
                // Verificar valor do empréstimo
                assert_eq!(loan.amount, managed_biguint!(13000)); // Valor ajustado para pontuação 600
                
                // Valor de repagamento: 13000 + (13000 * 520 / 10000) = 13000 + 676 = 13676
                assert_eq!(loan.repayment_amount, managed_biguint!(13676));
            })
            .assert_ok();
        
        // Etapa 3: LoanController obtém fundos e emite tokens de dívida
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
                sc.borrow();
            })
            .assert_ok();
        
            setup.blockchain_wrapper
            .execute_tx(&setup.debt_token_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
                sc.mint(managed_address!(&setup.provider1_address), managed_biguint!(100000));
            })
            .assert_ok();
        
        
        // Etapa 4: Simular avanço do tempo para além do vencimento
        setup.blockchain_wrapper
            .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                // Definir o timestamp atual para após o vencimento
                sc.set_mock_timestamp(100000); // Muito depois do vencimento
                
                // Verificar e marcar empréstimos vencidos
                sc.mark_expired_loans();
                
                // Verificar status do empréstimo
                let loan = sc.loans(1u64).get();
                assert_eq!(loan.status, LoanStatus::Defaulted);
            })
            .assert_ok();
        
        // Etapa 5: Atualizar pontuação de reputação negativamente
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.reputation_score_wrapper, &rust_biguint!(0), |sc| {
                sc.update_score(managed_address!(&setup.borrower2_address), 100); // Redução por inadimplência
                
                // Verificar nova pontuação
                let new_score = sc.get_user_score(managed_address!(&setup.borrower2_address));
                assert_eq!(new_score, 500u64); // 600 - 100
            })
            .assert_ok();
    }

    // Teste integrado de extensão de prazo de empréstimo
    #[test]
    fn test_loan_extension_integrated() {
        let mut setup = setup_integrated_system();
        
        // Etapa 1: Adicionar liquidez e configurar taxa de extensão
        setup.blockchain_wrapper
            .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
                sc.deposit_funds();
            })
            .assert_ok();
        
        // Simular emissão de tokens LP
        setup.blockchain_wrapper
            .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
                sc.mint_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(100000));
            })
            .assert_ok();
        
        // Configurar taxa de extensão
        setup.blockchain_wrapper
            .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.set_extension_fee_percent(1000u64); // 10%
            })
            .assert_ok();
        
        // Etapa 2: Tomador solicita empréstimo
        // Verificar pontuação
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.request_reputation_check();
            })
            .assert_ok();
        
        // Simular resposta da verificação de pontuação
        setup.blockchain_wrapper
            .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.reputation_check_callback(
                    managed_address!(&setup.borrower1_address),
                    700u64
                );
            })
            .assert_ok();
        
        // Solicitar empréstimo
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
                sc.request_loan_standard();
            })
            .assert_ok();
        
        // Verificar detalhes do empréstimo
        setup.blockchain_wrapper
            .execute_query(&setup.loan_controller_wrapper, |sc| {
                let loan = sc.loans(1u64).get();
                assert_eq!(loan.amount, managed_biguint!(13500));
                assert_eq!(loan.repayment_amount, managed_biguint!(14094));
            })
            .assert_ok();
        
        // Etapa 3: LoanController obtém fundos e emite tokens de dívida
        setup.blockchain_wrapper
            .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
                sc.borrow();
            })
            .assert_ok();
        
        setup.blockchain_wrapper
            .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
                sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
            })
            .assert_ok();
        
        // Etapa 4: Tomador solicita extensão de prazo
        setup.blockchain_wrapper
            .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(1409), |sc| {
                // Taxa de extensão: 10% do valor de repagamento (14094 * 10% = 1409)
                sc.extend_loan_deadline(1u64, 15u64); // Extender por 15 dias
                
                // Verificar detalhes do empréstimo atualizados
                let loan = sc.loans(1u64).get();
                
                // Novo valor de repagamento: 14094 + 1409 = 15503
                assert_eq!(loan.repayment_amount, managed_biguint!(15503));
                
                // Verificar que o prazo foi estendido
                // (Não testamos o timestamp exato, pois depende da implementação interna)
            })
            .assert_ok();
    }

    // Teste integrado de múltiplos tomadores e provedores
    #[test]
    fn test_multiple_borrowers_lenders_integrated() {
        let mut setup = setup_integrated_system();

        // Etapa 1: Múltiplos provedores adicionam liquidez
        setup.blockchain_wrapper
        .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(70000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();

        // Simular emissão de tokens LP
        setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(70000));
        })
        .assert_ok();

        setup.blockchain_wrapper
        .execute_tx(&setup.provider2_address, &setup.liquidity_pool_wrapper, &rust_biguint!(130000), |sc| {
            sc.deposit_funds();
        })
        .assert_ok();

        // Simular emissão de tokens LP
        setup.blockchain_wrapper
        .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
            sc.mint_endpoint(&managed_address!(&setup.provider2_address), &managed_biguint!(130000));
        })
        .assert_ok();

        // Verificar liquidez total
        setup.blockchain_wrapper
        .execute_query(&setup.liquidity_pool_wrapper, |sc| {
            assert_eq!(sc.total_liquidity().get(), managed_biguint!(200000));
        })
        .assert_ok();

        // Etapa 2: Tomador 1 solicita empréstimo
        // Verificar pontuação
        setup.blockchain_wrapper
        .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.request_reputation_check();
        })
        .assert_ok();

        // Simular resposta da verificação de pontuação
        setup.blockchain_wrapper
        .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
            sc.reputation_check_callback(
                managed_address!(&setup.borrower1_address),
                700u64
            );
        })
        .assert_ok();

    // Solicitar empréstimo
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower2_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.request_loan_standard();
    })
    .assert_ok();

    // Etapa 4: Verificar detalhes dos empréstimos
    setup.blockchain_wrapper
    .execute_query(&setup.loan_controller_wrapper, |sc| {
        let loan1 = sc.loans(1u64).get();
        let loan2 = sc.loans(2u64).get();
        
        // Empréstimo 1 (score 700)
        assert_eq!(loan1.amount, managed_biguint!(13500));
        assert_eq!(loan1.interest_rate, 440u64);
        
        // Empréstimo 2 (score 600)
        assert_eq!(loan2.amount, managed_biguint!(13000));
        assert_eq!(loan2.interest_rate, 520u64);
    })
    .assert_ok();

    // Etapa 5: LoanController obtém fundos para ambos
    setup.blockchain_wrapper
    .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
        // Empréstimo 1
        sc.borrow();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
        // Empréstimo 2
        sc.borrow();
    })
    .assert_ok();

    // Etapa 6: Emitir tokens de dívida para ambos
    setup.blockchain_wrapper
    .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
        // Tomador 1
        sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
        // Tomador 2
        sc.mint(managed_address!(&setup.borrower2_address), managed_biguint!(13000));
    })
    .assert_ok();

    // Verificar saldos de tokens
    setup.blockchain_wrapper
    .execute_query(&setup.debt_token_wrapper, |sc| {
        assert_eq!(sc.balance_of(managed_address!(&setup.borrower1_address)), managed_biguint!(13500));
        assert_eq!(sc.balance_of(managed_address!(&setup.borrower2_address)), managed_biguint!(13000));
    })
    .assert_ok();

    // Verificar estado da liquidez
    setup.blockchain_wrapper
    .execute_query(&setup.liquidity_pool_wrapper, |sc| {
        assert_eq!(sc.total_borrows().get(), managed_biguint!(26500));
        assert_eq!(sc.utilization_rate().get(), 1325u64); // 13.25%
    })
    .assert_ok();

    // Etapa 7: Tomador 1 paga empréstimo completo
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(14094), |sc| {
        // Simular queima de tokens de dívida
        sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        
        sc.repay_endpoint();
    })
    .assert_ok();

    // Queimar tokens de dívida
    setup.blockchain_wrapper
    .execute_tx(&setup.lp_token_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
        sc.burn_endpoint(&managed_address!(&setup.borrower1_address), &managed_biguint!(5000));
    })
    .assert_ok();


    // LoanController atualiza status
    setup.blockchain_wrapper
    .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.mark_loan_defaulted(1u64);
    })
    .assert_ok();

    // Etapa 8: Verificar estado atualizado
    setup.blockchain_wrapper
    .execute_query(&setup.liquidity_pool_wrapper, |sc| {
        assert_eq!(sc.total_borrows().get(), managed_biguint!(13000));
        assert_eq!(sc.utilization_rate().get(), 650u64); // 6.5%
    })
    .assert_ok();

    // Etapa 9: Simular acúmulo de juros
    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(676), |sc| {
        // Juros do empréstimo 2: 13000 * 5.2% = 676
        sc.add_accumulated_interest_endpoint(managed_biguint!(676));
    })
    .assert_ok();

    // Etapa 10: Distribuir juros entre provedores
    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
        sc.distribute_interest_endpoint();
        
        // Verificar distribuição
        // Reservas (20%): 676 * 20% = 135
        assert_eq!(sc.total_reserves().get(), managed_biguint!(135));
        
        // Juros para provedores (80%): 676 * 80% = 541
        // Provider 1 (35%): 541 * 35% = 189
        // Provider 2 (65%): 541 * 65% = 352
        assert_eq!(sc.provider_interest(&managed_address!(&setup.provider1_address)).get(), managed_biguint!(189));
        assert_eq!(sc.provider_interest(&managed_address!(&setup.provider2_address)).get(), managed_biguint!(352));
    })
    .assert_ok();
    }

    // Teste integrado de pausa e emergência em todo o sistema
    #[test]
    fn test_system_pause_emergency_integrated() {
    let mut setup = setup_integrated_system();

    // Etapa 1: Configurar estado inicial
    // Adicionar liquidez
    setup.blockchain_wrapper
    .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
        sc.deposit_funds();
    })
    .assert_ok();

    // Simular emissão de tokens LP
    setup.blockchain_wrapper
    .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
        sc.mint_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(100000));
    })
    .assert_ok();

    // Solicitar empréstimo
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.request_reputation_check();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.reputation_check_callback(
            managed_address!(&setup.borrower1_address),
            700u64
        );
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.request_loan_standard();
    })
    .assert_ok();

    // Processar empréstimo
    setup.blockchain_wrapper
    .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
        sc.borrow();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
        sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
    })
    .assert_ok();

    // Etapa 2: Pausar todos os contratos
    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.pause_contract();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
        sc.pause();
    })
    .assert_ok();

    // Etapa 3: Verificar que operações estão bloqueadas
    // Verificar status de pausa
    setup.blockchain_wrapper
    .execute_query(&setup.loan_controller_wrapper, |sc| {
        assert!(sc.is_paused());
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_query(&setup.liquidity_pool_wrapper, |sc| {
        assert!(sc.is_paused());
    })
    .assert_ok();

    // Etapa 4: Pausar o LP token
    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
        sc.pause();
    })
    .assert_ok();

    // Verificar status do token LP
    setup.blockchain_wrapper
    .execute_query(&setup.lp_token_wrapper, |sc| {
        assert!(sc.is_paused());
    })
    .assert_ok();

    // Etapa 5: Despausar e verificar operação normal
    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.unpause_contract();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
        sc.unpause();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
        sc.unpause();
    })
    .assert_ok();

    // Verificar status após despausar
    setup.blockchain_wrapper
    .execute_query(&setup.loan_controller_wrapper, |sc| {
        assert!(!sc.is_paused());
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_query(&setup.liquidity_pool_wrapper, |sc| {
        assert!(!sc.is_paused());
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_query(&setup.lp_token_wrapper, |sc| {
        assert!(!sc.is_paused());
    })
    .assert_ok();
    }

    // Teste integrado de uso de garantias (collateral)
    #[test]
    fn test_collateral_integrated() {
    let mut setup = setup_integrated_system();

    // Etapa 1: Configurar parâmetros de garantia
    setup.blockchain_wrapper
    .execute_tx(&setup.owner_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.set_collateral_ratio(7000u64); // 70% (empréstimo/garantia)
        
        // Use os métodos específicos do contrato diretamente
        // Isso evita a necessidade de criar ManagedBuffer
        sc.liquidation_threshold().set(8500u64);
        sc.liquidation_penalty().set(1000u64);
    })
    .assert_ok();

    // Etapa 2: Tomador fornece garantia
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(20000), |sc| {
        sc.provide_collateral_for_new_loan();
        
        // Verificar registro da garantia
        assert_eq!(sc.pending_collateral(managed_address!(&setup.borrower1_address)).get(), managed_biguint!(20000));
    })
    .assert_ok();

    // Etapa 3: Provedor adiciona liquidez
    setup.blockchain_wrapper
    .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
        sc.deposit_funds();
    })
    .assert_ok();

    // Simular emissão de tokens LP
    setup.blockchain_wrapper
    .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
        sc.mint_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(100000));
    })
    .assert_ok();

    // Etapa 4: Tomador solicita empréstimo com garantia
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        // Solicitar empréstimo baseado na garantia
        let loan_id = sc.request_loan_with_collateral();
        assert_eq!(loan_id, 1u64);
        
        // Verificar detalhes
        let loan = sc.loans(1u64).get();
        
        // Verificar transferência da garantia
        assert_eq!(sc.pending_collateral(managed_address!(&setup.borrower1_address)).get(), managed_biguint!(0));
        assert_eq!(sc.loan_collateral(1u64).get(), managed_biguint!(20000));
    })
    .assert_ok();

    // Etapa 5: LoanController obtém fundos e emite tokens de dívida
    // 1) Declare uma variável para armazenar o valor do empréstimo
    let mut loan_amount = BigUint::zero(); // Inicializa com zero

    // Execute a query e capture o valor
    setup
        .blockchain_wrapper
        .execute_query(&setup.loan_controller_wrapper, |sc| {
            // Remove o ponto e vírgula para capturar o valor
            let amount = sc.loans(1u64).get().amount.clone();
            // Armazena o valor na variável externa
            loan_amount = amount;
            // Retorna explicitamente unit ()
            ()
        })
        .assert_ok();

    // 2) Agora faça a emissão de DebtToken COM o valor correto:
    setup.blockchain_wrapper
        .execute_tx(
            &setup.debt_token_wrapper.address_ref(),
            &setup.debt_token_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.mint(
                    managed_address!(&setup.borrower1_address),
                    loan_amount.clone()
                );
                // Retorno vazio explícito
                ()
            },
        )
        .assert_ok();

// Emissão de tokens de dívida
// 1) Primeiro, faça a query do valor do empréstimo
let mut loan_amount = BigUint::zero(); // Inicializa com zero

setup
    .blockchain_wrapper
    .execute_query(&setup.loan_controller_wrapper, |sc| {
        // Captura o valor em uma variável temporária
        let temp_amount = sc.loans(1u64).get().amount.clone();
        
        // Armazena o valor em uma variável externa
        loan_amount = temp_amount;
        
        // Retorna unit type () para satisfazer o compilador
        ()
    })
    .assert_ok();

// 2) Agora faça a emissão de DebtToken COM o valor correto:
setup.blockchain_wrapper
    .execute_tx(
        &setup.debt_token_wrapper.address_ref(),
        &setup.debt_token_wrapper,
        &rust_biguint!(0),
        |sc| {
            sc.mint(
                managed_address!(&setup.borrower1_address),
                loan_amount.clone()
            );
            ()
        },
    )
    .assert_ok();


// 2) Agora emita os tokens de dívida usando esse valor
setup.blockchain_wrapper
    .execute_tx(
        &setup.debt_token_wrapper.address_ref(),
        &setup.debt_token_wrapper,
        &rust_biguint!(0),
        |sc| {
            sc.mint(
                managed_address!(&setup.borrower1_address),
                loan_amount.clone(),
            );
        },
    )
    .assert_ok();

    }

    // Teste de histórico de empréstimos
    #[test]
    fn test_loan_history_tracking() {
    let mut setup = setup_integrated_system();

    // Configurar liquidez inicial
    setup.blockchain_wrapper
    .execute_tx(&setup.provider1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(100000), |sc| {
        sc.deposit_funds();
    })
    .assert_ok();

    // Simular emissão de tokens LP
    setup.blockchain_wrapper
    .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.lp_token_wrapper, &rust_biguint!(0), |sc| {
        sc.mint_endpoint(&managed_address!(&setup.provider1_address), &managed_biguint!(100000));
    })
    .assert_ok();

    // Tomador solicita primeiro empréstimo
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.request_reputation_check();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.reputation_score_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.reputation_check_callback(
            managed_address!(&setup.borrower1_address),
            700u64
        );
    })
    .assert_ok();

    // Solicitar e processar o primeiro empréstimo
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        let loan_id = sc.request_loan_standard();
        assert_eq!(loan_id, 1u64);        
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.liquidity_pool_wrapper, &rust_biguint!(0), |sc| {
        sc.borrow();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.liquidity_pool_wrapper.address_ref(), &setup.debt_token_wrapper, &rust_biguint!(0), |sc| {
        sc.mint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
    })
    .assert_ok();

    // Pagar o primeiro empréstimo
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.liquidity_pool_wrapper, &rust_biguint!(14094), |sc| {
        // Simular queima de tokens de dívida
        sc.debt_tokens_burned_endpoint(managed_address!(&setup.borrower1_address), managed_biguint!(13500));
        sc.repay_endpoint();
    })
    .assert_ok();

    setup.blockchain_wrapper
    .execute_tx(&setup.loan_controller_wrapper.address_ref(), &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        sc.mark_loan_defaulted(1u64);
    })
    .assert_ok();

    // Solicitar segundo empréstimo
    setup.blockchain_wrapper
    .execute_tx(&setup.borrower1_address, &setup.loan_controller_wrapper, &rust_biguint!(0), |sc| {
        let loan_id = sc.request_loan_standard();
        assert_eq!(loan_id, 2u64);
    })
    .assert_ok();

    // Verificar histórico via user_loans
    setup.blockchain_wrapper
    .execute_query(&setup.loan_controller_wrapper, |sc| {
        let user_loans = sc.user_loans(managed_address!(&setup.borrower1_address));
        assert_eq!(user_loans.len(), 2); // Dois empréstimos para o mesmo usuário
        
        // Verificar status do primeiro empréstimo
        let loan1 = sc.loans(1u64).get();
        assert_eq!(loan1.status, LoanStatus::Repaid);
        
        // Verificar status do segundo empréstimo
        let loan2 = sc.loans(2u64).get();
        assert_eq!(loan2.status, LoanStatus::Active);
    })
    .assert_ok();

    // Verificar métricas de histórico, se o contrato implementar os endpoints
    setup.blockchain_wrapper
    .execute_query(&setup.loan_controller_wrapper, |sc| {
        if sc.user_loans(managed_address!(&setup.borrower1_address)).len() > 0 {
            let active_loans = sc.get_user_active_loans(managed_address!(&setup.borrower1_address));
            assert_eq!(active_loans.len(), 1);
            assert_eq!(active_loans.get(0), 2u64);
            
            let repaid_loans = sc.get_user_repaid_loans(managed_address!(&setup.borrower1_address));
            assert_eq!(repaid_loans.len(), 1);
            assert_eq!(repaid_loans.get(0), 1u64);
        }
    })
    .assert_ok();
    }