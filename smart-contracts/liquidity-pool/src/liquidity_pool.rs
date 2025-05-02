// ==========================================================================
// MÓDULO: liquidity-pool/src/lib.rs
// Descrição: Contrato inteligente que gerencia um pool de liquidez para
//            financiar empréstimos na blockchain MultiversX
// ==========================================================================
#![no_std]
multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait LiquidityPool {
    // Inicializa o contrato com os parâmetros básicos
    // Este método é chamado apenas uma vez, durante a implantação do contrato
    #[init]
    fn init(
        &self,
        loan_controller_address: ManagedAddress,
        min_deposit_amount: BigUint,
        annual_yield_percentage: u64,
    ) {
        require!(!loan_controller_address.is_zero(), "Invalid loan controller address");
        require!(annual_yield_percentage <= 10000, "Yield percentage too high"); // Máximo de 100% (base 10000)
        
        self.loan_controller_address().set(loan_controller_address);
        self.min_deposit_amount().set(min_deposit_amount);
        self.annual_yield_percentage().set(annual_yield_percentage);
        self.total_liquidity().set(BigUint::zero());
    }

    // Deposita fundos no pool de liquidez
    // Esta função permite que usuários depositem tokens para fornecer liquidez
    #[payable("*")]
    #[endpoint(depositFunds)]
    fn deposit_funds(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().egld_or_single_esdt();
        let amount = payment.amount;
        let token_id = payment.token_identifier;
        
        require!(
            amount >= self.min_deposit_amount().get(),
            "Deposit amount below minimum"
        );
        
        let current_timestamp = self.blockchain().get_block_timestamp();
        
        // Se for a primeira vez depositando, cria registro do provedor
        if self.provider_funds(caller.clone()).is_empty() {
            self.provider_funds(caller.clone()).set(ProviderFunds {
                token_id: token_id.clone(),
                amount: amount.clone(),
                last_yield_timestamp: current_timestamp,
            });
            
            // Adiciona à lista de provedores
            self.providers().push(&caller);
        } else {
            // Se já for um provedor, atualiza os registros
            // Primeiro, processa qualquer rendimento pendente
            self.process_pending_yield(&caller)?; // Adicionado ? para propagação de erro
            
            // Em seguida, adiciona o novo depósito
            let mut provider_funds = self.provider_funds(caller.clone()).get();
            require!(
                provider_funds.token_id == token_id,
                "Token type doesn't match existing deposit"
            );
            
            provider_funds.amount += amount;
            provider_funds.last_yield_timestamp = current_timestamp;
            
            self.provider_funds(caller).set(provider_funds);
        }
        
        // Atualiza a liquidez total do pool
        self.total_liquidity().update(|liquidity| *liquidity += amount);
        
        // Emite evento para auditoria
        self.funds_deposited_event(&caller, &amount);
        
        Ok(()) // Este retorno está correto, desde que SCResult seja o tipo importado corretamente
    }
    
    // Retira fundos do pool de liquidez
    // Esta função permite que provedores de liquidez retirem seus fundos e rendimentos
    #[endpoint(withdrawFunds)]
    fn withdraw_funds(&self, amount: BigUint) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        
        require!(
            !self.provider_funds(caller.clone()).is_empty(),
            "Not a liquidity provider"
        );
        
        // Processa qualquer rendimento pendente primeiro
        self.process_pending_yield(&caller)?; // Adicionado ? para propagação de erro
        
        let mut provider_funds = self.provider_funds(caller.clone()).get();
        
        require!(
            amount <= provider_funds.amount,
            "Insufficient funds to withdraw"
        );
        
        // Atualiza fundos do provedor
        provider_funds.amount -= &amount;
        let token_id = provider_funds.token_id.clone();
        let current_timestamp = self.blockchain().get_block_timestamp();
        provider_funds.last_yield_timestamp = current_timestamp;
        
        self.provider_funds(caller.clone()).set(provider_funds);
        
        // Atualiza a liquidez total do pool
        self.total_liquidity().update(|liquidity| *liquidity -= &amount);
        
        // Se retirou completamente, remove da lista de provedores
        if self.provider_funds(caller.clone()).get().amount == BigUint::zero() {
            // CORREÇÃO: O código anterior obtinha toda a lista de provedores de uma vez
            // e depois tentava manipulá-la, o que não funciona com VecMapper.
            // Agora iteramos pelos índices e usamos o método apropriado para remover.
            let provider_count = self.providers().len();
            for i in 0..provider_count {
                let provider_addr = self.providers().get(i);
                if provider_addr == caller {
                    // Usa swap_remove para remover eficientemente
                    self.providers().swap_remove(i);
                    break;
                }
            }
            
            // Limpa o armazenamento provider_funds
            self.provider_funds(caller.clone()).clear();
        }
        
        // CORREÇÃO: Conversão adequada de TokenIdentifier para EgldOrEsdtTokenIdentifier
        // que é o tipo esperado pelo método direct()
        let esdt_token_id = EgldOrEsdtTokenIdentifier::esdt(token_id);
        self.send().direct(&caller, &esdt_token_id, 0, &amount);
        
        // Emite evento para auditoria
        self.funds_withdrawn_event(&caller, &amount);
        
        Ok(())
    }
    
    // Processa rendimento pendente para um provedor
    // Esta função interna calcula e adiciona rendimentos com base no tempo decorrido
    fn process_pending_yield(&self, provider: &ManagedAddress) -> SCResult<()> {
        let mut provider_funds = self.provider_funds(provider.clone()).get();
        let current_timestamp = self.blockchain().get_block_timestamp();
        let time_diff_seconds = current_timestamp - provider_funds.last_yield_timestamp;
        
        // Pula se nenhum tempo passou
        if time_diff_seconds == 0 {
            return Ok(());
        }
        
        // Calcula rendimento: annual_yield_percentage / seconds_in_year * time_diff_seconds * amount
        let annual_yield_percentage = self.annual_yield_percentage().get();
        let seconds_in_year = 31_536_000u64; // 365 dias * 24 horas * 60 minutos * 60 segundos
        
        let yield_amount = provider_funds.amount.clone() * 
            BigUint::from(annual_yield_percentage) * 
            BigUint::from(time_diff_seconds) / 
            BigUint::from(seconds_in_year) / 
            BigUint::from(10_000u32); // Base percentual é 10000 (ex: 1000 = 10%)
        
        // Adiciona rendimento aos fundos do provedor
        provider_funds.amount += &yield_amount;
        provider_funds.last_yield_timestamp = current_timestamp;
        
        self.provider_funds(provider.clone()).set(provider_funds);
        
        // Emite evento para auditoria
        if yield_amount > BigUint::zero() {
            self.yield_processed_event(provider, &yield_amount);
        }
        
        Ok(())
    }
    
    // Fornece fundos para empréstimo
    // Esta função permite que o controlador de empréstimos obtenha fundos do pool
    #[endpoint(provideFundsForLoan)]
    fn provide_funds_for_loan(&self, amount: BigUint, token_id: TokenIdentifier) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        
        require!(
            caller == self.loan_controller_address().get(),
            "Only loan controller can request funds"
        );
        
        require!(
            self.total_liquidity().get() >= amount,
            "Insufficient liquidity in pool"
        );
        
        // CORREÇÃO: Conversão adequada de TokenIdentifier para EgldOrEsdtTokenIdentifier
        let esdt_token_id = EgldOrEsdtTokenIdentifier::esdt(token_id);
        self.send().direct(&caller, &esdt_token_id, 0, &amount);
        
        // Atualiza a liquidez total
        self.total_liquidity().update(|liquidity| *liquidity -= &amount);
        
        // Emite evento para auditoria
        self.funds_provided_for_loan_event(&amount);
        
        Ok(())
    }
    
    // Recebe pagamento de empréstimo
    // Esta função permite que o controlador de empréstimos devolva fundos ao pool
    #[payable("*")]
    #[endpoint(receiveLoanRepayment)]
    fn receive_loan_repayment(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        
        require!(
            caller == self.loan_controller_address().get(),
            "Only loan controller can repay loans"
        );
        
        let payment = self.call_value().egld_or_single_esdt();
        let amount = payment.amount;
        
        // Atualiza a liquidez total
        self.total_liquidity().update(|liquidity| *liquidity += amount);
        
        // Emite evento para auditoria
        self.loan_repayment_received_event(&amount);
        
        Ok(())
    }
    
    // Obter taxa de rendimento anual atual
    // Esta função permite consultar a taxa de rendimento atual do pool
    #[view(getAnnualYieldPercentage)]
    fn get_annual_yield_percentage(&self) -> u64 {
        self.annual_yield_percentage().get()
    }
    
    // Obter liquidez total disponível no pool
    // Esta função permite consultar o total de fundos disponíveis
    #[view(getTotalLiquidity)]
    fn get_total_liquidity(&self) -> BigUint {
        self.total_liquidity().get()
    }
    
    // Obter fundos de um provedor específico
    // Esta função permite consultar os fundos e rendimentos de um provedor
    #[view(getProviderFunds)]
    fn get_provider_funds(&self, provider: ManagedAddress) -> ProviderFunds<Self::Api> {
        if self.provider_funds(provider.clone()).is_empty() {
            // Retorna estrutura vazia se não for um provedor
            return ProviderFunds {
                token_id: TokenIdentifier::from_esdt_bytes(&[]),
                amount: BigUint::zero(),
                last_yield_timestamp: 0,
            };
        }
        self.provider_funds(provider).get()
    }
    
    // Eventos para auditoria
    #[event("funds_deposited")]
    fn funds_deposited_event(
        &self,
        #[indexed] provider: &ManagedAddress,
        #[indexed] amount: &BigUint,
    );
    
    #[event("funds_withdrawn")]
    fn funds_withdrawn_event(
        &self,
        #[indexed] provider: &ManagedAddress,
        #[indexed] amount: &BigUint,
    );
    
    #[event("yield_processed")]
    fn yield_processed_event(
        &self,
        #[indexed] provider: &ManagedAddress,
        #[indexed] amount: &BigUint,
    );
    
    #[event("funds_provided_for_loan")]
    fn funds_provided_for_loan_event(
        &self,
        #[indexed] amount: &BigUint,
    );
    
    #[event("loan_repayment_received")]
    fn loan_repayment_received_event(
        &self,
        #[indexed] amount: &BigUint,
    );
    
    // Definições de armazenamento (storage)
    
    // Endereço do contrato controlador de empréstimos
    #[storage_mapper("loan_controller_address")]
    fn loan_controller_address(&self) -> SingleValueMapper<ManagedAddress>;
    
    // Valor mínimo para depósito
    #[storage_mapper("min_deposit_amount")]
    fn min_deposit_amount(&self) -> SingleValueMapper<BigUint>;
    
    // Percentual de rendimento anual (em base 10000)
    #[storage_mapper("annual_yield_percentage")]
    fn annual_yield_percentage(&self) -> SingleValueMapper<u64>;
    
    // Liquidez total disponível no pool
    #[storage_mapper("total_liquidity")]
    fn total_liquidity(&self) -> SingleValueMapper<BigUint>;
    
    // Lista de endereços dos provedores de liquidez
    #[storage_mapper("providers")]
    fn providers(&self) -> VecMapper<ManagedAddress>;
    
    // Mapeamento de endereços de provedores para seus fundos
    #[storage_mapper("provider_funds")]
    fn provider_funds(&self, provider: ManagedAddress) -> SingleValueMapper<ProviderFunds<Self::Api>>;
}

// Estrutura para armazenar informações dos fundos do provedor
// NOTA: As anotações de derive já estão corretas neste arquivo.
// Apenas certifique-se de que estão importadas corretamente.
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct ProviderFunds<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,    // Identificador do token depositado
    pub amount: BigUint<M>,              // Valor total incluindo rendimentos
    pub last_yield_timestamp: u64,       // Timestamp do último cálculo de rendimento
}