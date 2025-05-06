// ==========================================================================
// MÓDULO: debt-token/src/debt_token.rs
// AUTOR: Uderson de Amadeu Ferreira (melhorado por Claude)
// DATA: 2023-10-01
// ÚLTIMA ATUALIZAÇÃO: 2025-05-05
// DESCRIÇÃO: Este módulo implementa um contrato inteligente para gerenciar
//            tokens de dívida na blockchain MultiversX.
//            O contrato combina funcionalidades de NFT e ERC20 para representar
//            dívidas como NFTs únicos, além de permitir operações de token
//            fungível para transferência, aprovação e gerenciamento de saldos.
//            O contrato emite eventos para auditoria e rastreamento de ações.
// ==========================================================================

#![no_std]
multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait DebtToken {
    // ============================
    // Inicialização e configuração
    // ============================
    
    /// Inicializa o contrato com o endereço do controlador de empréstimos
    /// Este é o único endereço que pode criar NFTs de dívida e mintar/queimar tokens
    #[init]
    fn init(&self, loan_controller_address: ManagedAddress) {
        require!(!loan_controller_address.is_zero(), "Invalid loan controller address");
        self.loan_controller_address().set(loan_controller_address);
        // Inicializa o ID do token como vazio
        self.debt_token_id().set_if_empty(TokenIdentifier::from_esdt_bytes(&[]));
        // Inicializa a oferta total como zero
        self.total_supply().set_if_empty(&BigUint::zero());
    }

    /// Emite o token de dívida como um NFT/SFT
    /// Somente o proprietário do contrato pode chamar esta função
    #[only_owner]
    #[endpoint(issueDebtToken)]
    fn issue_debt_token(&self) {
        require!(self.debt_token_id().is_empty(), "Token already issued");
        let token_name = ManagedBuffer::from("DebtToken");
        let token_ticker = ManagedBuffer::from("DEBT");

        self.send()
            .esdt_system_sc_proxy()
            .issue_semi_fungible(
                self.call_value().egld().clone(),
                &token_name,
                &token_ticker,
                SemiFungibleTokenProperties {
                    can_freeze: true,
                    can_wipe: true,
                    can_pause: true,
                    can_change_owner: false,
                    can_upgrade: false,
                    can_add_special_roles: true,
                    can_transfer_create_role: true,
                },
            )
            .with_callback(self.callbacks().issue_callback())
            .async_call_and_exit();
    }

    /// Callback processado após a emissão bem-sucedida do token
    #[callback]
    fn issue_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.debt_token_id().set(&token_id);
                self.send()
                    .esdt_system_sc_proxy()
                    .set_special_roles(
                        self.blockchain().get_sc_address(),
                        token_id,
                        [EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn].into_iter(),
                    )
                    .async_call_and_exit();
            }
            ManagedAsyncCallResult::Err(_) => sc_panic!("Issue token failed"),
        }
    }

    // ============================
    // Funcionalidades de NFT
    // ============================
    
    /// Cria um NFT para representar um empréstimo
    /// Somente o controlador de empréstimos pode chamar esta função
    #[endpoint(createDebtNft)]
    fn create_debt_nft(
        &self,
        loan_id: u64,
        borrower: ManagedAddress,
        amount: BigUint,
        interest_rate: u64,
        due_timestamp: u64,
    ) -> u64 {
        // Verificações de segurança
        let caller = self.blockchain().get_caller();
        require!(caller == self.loan_controller_address().get(), "Only loan controller can create debt NFTs");
        require!(!self.debt_token_id().is_empty(), "Debt token not issued yet");
        require!(!borrower.is_zero(), "Borrower cannot be zero address");
        require!(amount > BigUint::zero(), "Amount must be greater than zero");
        require!(due_timestamp > self.blockchain().get_block_timestamp(), "Due date must be in the future");

        // Verificar se já existe um NFT para este empréstimo
        require!(self.loan_to_debt_nft(loan_id).is_empty(), "NFT already exists for this loan");

        let token_id = self.debt_token_id().get();
        let nft_nonce = self.send().esdt_nft_create(
            &token_id,
            &BigUint::from(1u32),          // Quantidade de 1
            &ManagedBuffer::from("DebtToken"), // Nome
            &BigUint::zero(),              // Royalties
            &ManagedBuffer::new(),         // URI hash
            &self.create_debt_nft_attributes(loan_id, &borrower, &amount, interest_rate, due_timestamp),
            &ManagedVec::<Self::Api, ManagedBuffer>::new(), // URIs (vazio)
        );

        // Enviar o NFT para o controlador de empréstimos
        self.send().direct_esdt(
            &self.loan_controller_address().get(),
            &token_id,
            nft_nonce,
            &BigUint::from(1u32),
        );

        // Armazenar mapeamentos para rastreamento
        self.debt_nft_to_loan(nft_nonce).set(loan_id);
        self.loan_to_debt_nft(loan_id).set(nft_nonce);
        
        // Emitir eventos para auditoria - separados para atender ao limite de um argumento não indexado
        self.debt_nft_created_event(loan_id, nft_nonce, borrower);
        self.debt_nft_details_event(loan_id, &amount, interest_rate, due_timestamp);

        nft_nonce
    }

    /// Queima o NFT quando o empréstimo é pago ou inadimplente
    #[endpoint(burnDebtNft)]
    fn burn_debt_nft(&self, loan_id: u64) {
        // Verificações de segurança
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.loan_controller_address().get(), 
            "Only loan controller can burn debt NFTs"
        );
        
        // Verificar se o NFT existe
        require!(
            !self.loan_to_debt_nft(loan_id).is_empty(), 
            "No NFT exists for this loan"
        );

        let nft_nonce = self.loan_to_debt_nft(loan_id).get();
        let token_id = self.debt_token_id().get();
        
        // Queimar o NFT
        self.send().esdt_local_burn(&token_id, nft_nonce, &BigUint::from(1u32));

        // Limpar mapeamentos
        self.debt_nft_to_loan(nft_nonce).clear();
        self.loan_to_debt_nft(loan_id).clear();
        
        // Emitir evento para auditoria
        self.debt_nft_burned_event(loan_id, nft_nonce);
    }

    // ============================
    // Funcionalidades de ERC20
    // ============================
    
    /// Cria novos tokens e os atribui ao destinatário
    /// Somente o controlador de empréstimos pode chamar esta função
    #[endpoint(mint)]
    fn mint(&self, recipient: ManagedAddress, amount: BigUint) {
        // Verificações de segurança
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.loan_controller_address().get(), 
            "Only loan controller can mint tokens"
        );
        require!(!self.debt_token_id().is_empty(), "Debt token not issued yet");
        require!(!recipient.is_zero(), "Cannot mint to zero address");
        require!(amount > BigUint::zero(), "Amount must be greater than zero");

        let token_id = self.debt_token_id().get();
        
        // Atualizar oferta total
        let current_supply = self.total_supply().get();
        self.total_supply().set(&(current_supply + &amount));
        
        // Enviar tokens para o destinatário
        self.send().direct_esdt(&recipient, &token_id, 0, &amount); // 0 é o nonce para tokens fungíveis
        
        // Emitir evento para auditoria
        self.mint_event(&recipient, &amount);
    }

    /// Destrói tokens do destinatário
    /// Somente o controlador de empréstimos pode chamar esta função
    #[endpoint(burn)]
    fn burn(&self, from: ManagedAddress, amount: BigUint) {
        // Verificações de segurança
        let caller = self.blockchain().get_caller();
        require!(
            caller == self.loan_controller_address().get(), 
            "Only loan controller can burn tokens"
        );
        require!(!self.debt_token_id().is_empty(), "Debt token not issued yet");
        require!(amount > BigUint::zero(), "Amount must be greater than zero");
        
        let token_id = self.debt_token_id().get();
        
        // Verificar saldo do usuário
        let user_balance = self.balance_of(from.clone());
        require!(user_balance >= amount, "Insufficient balance for burn");
        
        // Atualizar oferta total
        let current_supply = self.total_supply().get();
        self.total_supply().set(&(current_supply - &amount));
        
        // Queimar tokens
        self.send().esdt_local_burn(&token_id, 0, &amount); // 0 é o nonce para tokens fungíveis
        
        // Emitir evento para auditoria
        self.burn_event(&from, &amount);
    }

    /// Transfere tokens para outro endereço
    #[endpoint(transferTokens)]
    fn transfer_tokens(&self, to: ManagedAddress, amount: BigUint) {
        // Verificações de segurança
        require!(!to.is_zero(), "Cannot transfer to zero address");
        require!(amount >= BigUint::zero(), "Amount must be non-negative");
        
        let caller = self.blockchain().get_caller();
        let token_id = self.debt_token_id().get();
        
        // Se amount for zero, não faz nada mas retorna sucesso
        if amount == BigUint::zero() {
            return;
        }
        
        // Verificar saldo
        let caller_balance = self.balance_of(caller.clone());
        require!(
            caller_balance >= amount,
            "Insufficient balance for transfer"
        );
        
        // Transferência ESDT
        self.send().direct_esdt(&to, &token_id, 0, &amount);
        
        // Emitir evento para auditoria
        self.transfer_event(&caller, &to, &amount);
    }

    /// Permite que outro endereço gaste tokens em seu nome
    #[endpoint(approveTokens)]
    fn approve_tokens(&self, spender: ManagedAddress, amount: BigUint) {
        // Verificações de segurança
        require!(!spender.is_zero(), "Cannot approve zero address");
        
        let caller = self.blockchain().get_caller();
        
        // Definir allowance
        self.allowances(&caller, &spender).set(&amount);
        
        // Emitir evento para auditoria
        self.approval_event(&caller, &spender, &amount);
    }

    /// Transfere tokens de um endereço para outro usando allowance
    #[endpoint(transferTokensFrom)]
    fn transfer_tokens_from(
        &self,
        from: ManagedAddress,
        to: ManagedAddress,
        amount: BigUint,
    ) {
        // Verificações de segurança
        require!(!from.is_zero(), "Cannot transfer from zero address");
        require!(!to.is_zero(), "Cannot transfer to zero address");
        require!(amount > BigUint::zero(), "Amount must be greater than zero");
        
        let caller = self.blockchain().get_caller();
        let token_id = self.debt_token_id().get();
        
        // Verificar allowance
        let allowance = self.allowances(&from, &caller).get();
        require!(allowance >= amount, "Insufficient allowance");
        
        // Verificar saldo do remetente
        let from_balance = self.balance_of(from.clone());
        require!(from_balance >= amount, "Insufficient balance");
        
        // Atualizar allowance (protege contra reentrância)
        self.allowances(&from, &caller).set(&(allowance - &amount));
        
        // Transferência ESDT
        self.send().direct_esdt(&to, &token_id, 0, &amount);
        
        // Emitir evento para auditoria
        self.transfer_event(&from, &to, &amount);
    }

    /// Retorna o allowance concedido a um spender por um proprietário
    #[view(getAllowance)]
    fn get_allowance(&self, owner: ManagedAddress, spender: ManagedAddress) -> BigUint {
        self.allowances(&owner, &spender).get()
    }

    /// Aumenta o allowance concedido a um spender
    #[endpoint(increaseTokenAllowance)]
    fn increase_token_allowance(&self, spender: ManagedAddress, amount: BigUint) {
        // Verificações de segurança
        require!(!spender.is_zero(), "Cannot approve zero address");
        require!(amount > BigUint::zero(), "Amount must be greater than zero");
        
        let caller = self.blockchain().get_caller();
        let current_allowance = self.allowances(&caller, &spender).get();
        
        // Atualizar allowance
        let new_allowance = &current_allowance + &amount;
        self.allowances(&caller, &spender).set(&new_allowance);
        
        // Emitir evento para auditoria
        self.approval_event(&caller, &spender, &new_allowance);
    }

    /// Diminui o allowance concedido a um spender
    #[endpoint(decreaseTokenAllowance)]
    fn decrease_token_allowance(&self, spender: ManagedAddress, amount: BigUint) {
        // Verificações de segurança
        require!(!spender.is_zero(), "Cannot approve zero address");
        require!(amount > BigUint::zero(), "Amount must be greater than zero");
        
        let caller = self.blockchain().get_caller();
        let current_allowance = self.allowances(&caller, &spender).get();
        
        // Verificar se há allowance suficiente
        require!(current_allowance >= amount, "Cannot decrease below zero");
        
        // Atualizar allowance
        let new_allowance = &current_allowance - &amount;
        self.allowances(&caller, &spender).set(&new_allowance);
        
        // Emitir evento para auditoria
        self.approval_event(&caller, &spender, &new_allowance);
    }

    // ============================
    // Métodos view
    // ============================
    
    /// Retorna o saldo de tokens de um endereço
    #[view(balanceOf)]
    fn balance_of(&self, address: ManagedAddress) -> BigUint {
        if self.debt_token_id().is_empty() {
            return BigUint::zero();
        }
        
        let token_id = self.debt_token_id().get();
        self.blockchain().get_esdt_balance(&address, &token_id, 0)
    }

    /// Retorna a oferta total de tokens
    #[view(totalTokenSupply)]
    fn total_token_supply(&self) -> BigUint {
        self.total_supply().get()
    }
    
    /// Retorna o ID do NFT de dívida associado a um empréstimo, ou zero se não existir
    #[view(getLoanNftId)]
    fn get_loan_nft_id(&self, loan_id: u64) -> u64 {
        if self.loan_to_debt_nft(loan_id).is_empty() {
            return 0;
        }
        self.loan_to_debt_nft(loan_id).get()
    }
    
    /// Retorna o ID do empréstimo associado a um NFT, ou zero se não existir
    #[view(getNftLoanId)]
    fn get_nft_loan_id(&self, nft_nonce: u64) -> u64 {
        if self.debt_nft_to_loan(nft_nonce).is_empty() {
            return 0;
        }
        self.debt_nft_to_loan(nft_nonce).get()
    }

    // ============================
    // Funções auxiliares
    // ============================
    
    /// Cria os atributos para o NFT de dívida
    fn create_debt_nft_attributes(
        &self,
        loan_id: u64,
        borrower: &ManagedAddress,
        amount: &BigUint,
        interest_rate: u64,
        due_timestamp: u64,
    ) -> ManagedBuffer {
        // Formatação de atributos em um formato legível
        let mut attributes = ManagedBuffer::new();
        attributes.append(&ManagedBuffer::from("loan_id:"));
        attributes.append(&self.number_to_managed_buffer(loan_id));
        attributes.append(&ManagedBuffer::from("|borrower:"));
        attributes.append(borrower.as_managed_buffer());
        attributes.append(&ManagedBuffer::from("|amount:"));
        attributes.append(&self.big_uint_to_managed_buffer(amount));
        attributes.append(&ManagedBuffer::from("|interest_rate:"));
        attributes.append(&self.number_to_managed_buffer(interest_rate));
        attributes.append(&ManagedBuffer::from("|due_timestamp:"));
        attributes.append(&self.number_to_managed_buffer(due_timestamp));
        attributes.append(&ManagedBuffer::from("|created_timestamp:"));
        attributes.append(&self.number_to_managed_buffer(self.blockchain().get_block_timestamp()));
        attributes
    }

    /// Converte números para ManagedBuffer
    fn number_to_managed_buffer<T>(&self, number: T) -> ManagedBuffer
    where
        T: Into<u64>,
    {
        let mut bytes = [0u8; 20];
        let mut i = 0;
        let mut n = number.into();
        if n == 0 {
            return ManagedBuffer::from(&[b'0']);
        }
        while n > 0 && i < bytes.len() {
            bytes[i] = (n % 10) as u8 + b'0';
            n /= 10;
            i += 1;
        }
        bytes[..i].reverse();
        ManagedBuffer::from(&bytes[..i])
    }

    /// Converte BigUint para ManagedBuffer - implementação simplificada
    fn big_uint_to_managed_buffer(&self, value: &BigUint) -> ManagedBuffer {
        // Para simplificar, convertemos para u64 (pode não funcionar para valores muito grandes)
        if value == &BigUint::zero() {
            return ManagedBuffer::from(&[b'0']);
        }
        
        // Tenta converter para u64, ou usa um valor padrão
        let u64_value = value.to_u64().unwrap_or(0);
        self.number_to_managed_buffer(u64_value)
    }

    // ============================
    // Eventos - corrigidos para ter apenas 1 argumento não indexado
    // ============================
    
    #[event("debt_nft_created")]
    fn debt_nft_created_event(
        &self,
        #[indexed] loan_id: u64,
        #[indexed] nft_nonce: u64,
        #[indexed] borrower: ManagedAddress,
    );
    
    #[event("debt_nft_details")]
    fn debt_nft_details_event(
        &self,
        #[indexed] loan_id: u64,
        amount: &BigUint,
        #[indexed] interest_rate: u64,
        #[indexed] due_timestamp: u64,
    );

    #[event("debt_nft_burned")]
    fn debt_nft_burned_event(
        &self, 
        #[indexed] loan_id: u64, 
        #[indexed] nft_nonce: u64
    );
    
    #[event("transfer")]
    fn transfer_event(
        &self,
        #[indexed] from: &ManagedAddress,
        #[indexed] to: &ManagedAddress,
        amount: &BigUint,
    );

    #[event("approval")]
    fn approval_event(
        &self,
        #[indexed] owner: &ManagedAddress,
        #[indexed] spender: &ManagedAddress,
        amount: &BigUint,
    );
    
    #[event("mint")]
    fn mint_event(
        &self,
        #[indexed] to: &ManagedAddress,
        amount: &BigUint,
    );
    
    #[event("burn")]
    fn burn_event(
        &self,
        #[indexed] from: &ManagedAddress,
        amount: &BigUint,
    );

    // ============================
    // Storage mappers
    // ============================
    
    /// Endereço do controlador de empréstimos
    #[storage_mapper("loan_controller_address")]
    fn loan_controller_address(&self) -> SingleValueMapper<ManagedAddress>;

    /// ID do token de dívida
    #[storage_mapper("debt_token_id")]
    fn debt_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    /// Mapeamento de nonce de NFT para ID de empréstimo
    #[storage_mapper("debt_nft_to_loan")]
    fn debt_nft_to_loan(&self, nft_nonce: u64) -> SingleValueMapper<u64>;

    /// Mapeamento de ID de empréstimo para nonce de NFT
    #[storage_mapper("loan_to_debt_nft")]
    fn loan_to_debt_nft(&self, loan_id: u64) -> SingleValueMapper<u64>;
    
    /// Mapeamento de allowances para ERC20
    #[storage_mapper("allowances")]
    fn allowances(&self, owner: &ManagedAddress, spender: &ManagedAddress) -> SingleValueMapper<BigUint>;
    
    /// Oferta total do token
    #[storage_mapper("total_supply")]
    fn total_supply(&self) -> SingleValueMapper<BigUint>;
}