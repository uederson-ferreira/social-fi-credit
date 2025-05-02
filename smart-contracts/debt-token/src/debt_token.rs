// ==========================================================================
// MÓDULO: debt-token/src/lib.rs
// Descrição: Contrato inteligente que gerencia tokens de dívida como NFTs
//            para representar empréstimos na blockchain MultiversX
// ==========================================================================
#![no_std]
multiversx_sc::imports!();
use core::convert::TryFrom;

#[multiversx_sc::contract]
pub trait DebtToken {
    // Inicializa o contrato com o endereço do controlador de empréstimos
    #[init]
    fn init(&self, loan_controller_address: ManagedAddress) {
        require!(!loan_controller_address.is_zero(), "Invalid loan controller address");
        self.loan_controller_address().set(loan_controller_address);
        // Inicializa o ID do token como vazio
        self.debt_token_id().set_if_empty(TokenIdentifier::from_esdt_bytes(&[]));
    }

    // Emite o token de dívida como um NFT/SFT
    #[only_owner]
    #[endpoint(issueDebtToken)]
    fn issue_debt_token(&self) {
        require!(self.debt_token_id().is_empty(), "Token already issued");
        let token_name = ManagedBuffer::from("DebtToken");
        let token_ticker = ManagedBuffer::from("DEBT");

        // Ajuste: clone() para converter ManagedRef<BigUint> em BigUint
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

    // Callback após a emissão do token
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

    // Cria um NFT para um empréstimo (somente controlador de empréstimos)
    #[endpoint(createDebtNft)]
    fn create_debt_nft(
        &self,
        loan_id: u64,
        borrower: ManagedAddress,
        amount: BigUint,
        interest_rate: u64,
        due_timestamp: u64,
    ) -> u64 {
        let caller = self.blockchain().get_caller();
        require!(caller == self.loan_controller_address().get(), "Only loan controller can create debt NFTs");
        require!(!self.debt_token_id().is_empty(), "Debt token not issued yet");

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

        self.send().direct_esdt(
            &self.loan_controller_address().get(),
            &token_id,
            nft_nonce,
            &BigUint::from(1u32),
        );

        self.debt_nft_to_loan(nft_nonce).set(loan_id);
        self.loan_to_debt_nft(loan_id).set(nft_nonce);
        self.debt_nft_created_event(loan_id, nft_nonce, borrower);

        nft_nonce
    }

    // Queima o NFT quando o empréstimo é pago ou inadimplente
    #[endpoint(burnDebtNft)]
    fn burn_debt_nft(&self, loan_id: u64) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.loan_controller_address().get(), "Only loan controller can burn debt NFTs");
        require!(!self.loan_to_debt_nft(loan_id).is_empty(), "No NFT exists for this loan");

        let nft_nonce = self.loan_to_debt_nft(loan_id).get();
        let token_id = self.debt_token_id().get();
        self.send().esdt_local_burn(&token_id, nft_nonce, &BigUint::from(1u32));

        self.debt_nft_to_loan(nft_nonce).clear();
        self.loan_to_debt_nft(loan_id).clear();
        self.debt_nft_burned_event(loan_id, nft_nonce);
    }

    // Formata atributos do NFT
    fn create_debt_nft_attributes(
        &self,
        loan_id: u64,
        borrower: &ManagedAddress,
        amount: &BigUint,
        interest_rate: u64,
        due_timestamp: u64,
    ) -> ManagedBuffer {
        let mut attributes = ManagedBuffer::new();
        let loan_buf = self.number_to_managed_buffer(loan_id);
        let amount_buf = self.big_uint_to_managed_buffer(amount);
        let rate_buf = self.number_to_managed_buffer(interest_rate);
        let due_buf = self.number_to_managed_buffer(due_timestamp);

        attributes.append(&loan_buf);
        attributes.append(&ManagedBuffer::from("|"));
        attributes.append(borrower.as_managed_buffer());
        attributes.append(&ManagedBuffer::from("|"));
        attributes.append(&amount_buf);
        attributes.append(&ManagedBuffer::from("|"));
        attributes.append(&rate_buf);
        attributes.append(&ManagedBuffer::from("|"));
        attributes.append(&due_buf);
        attributes
    }

    // Converte números simples para ManagedBuffer
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

    // Simplifica conversão de BigUint para ManagedBuffer (até 64 bits)
    fn big_uint_to_managed_buffer(&self, value: &BigUint) -> ManagedBuffer {
        let val = value.to_u64().unwrap_or(0);
        self.number_to_managed_buffer(val)
    }

    // Eventos de auditoria
    #[event("debt_nft_created")]
    fn debt_nft_created_event(
        &self,
        #[indexed] loan_id: u64,
        #[indexed] nft_nonce: u64,
        #[indexed] borrower: ManagedAddress,
    );

    #[event("debt_nft_burned")]
    fn debt_nft_burned_event(&self, #[indexed] loan_id: u64, #[indexed] nft_nonce: u64);

    // Storage mappers
    #[storage_mapper("loan_controller_address")]
    fn loan_controller_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("debt_token_id")]
    fn debt_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("debt_nft_to_loan")]
    fn debt_nft_to_loan(&self, nft_nonce: u64) -> SingleValueMapper<u64>;

    #[storage_mapper("loan_to_debt_nft")]
    fn loan_to_debt_nft(&self, loan_id: u64) -> SingleValueMapper<u64>;
}
