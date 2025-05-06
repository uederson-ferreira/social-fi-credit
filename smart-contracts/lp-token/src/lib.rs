#![no_std]
// #![no_std] indica que este código não usa a biblioteca padrão do Rust

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

/// Estrutura que armazena as informações básicas do token
/// - name: Nome do token
/// - ticker: Símbolo/código do token (ex: BTC, ETH)
/// - decimals: Número de casas decimais (geralmente 18 para compatibilidade com ERC-20)
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct TokenInfo <M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub ticker: ManagedBuffer<M>,
    pub decimals: u8,
}

/// Contrato principal de token
/// Implementa funcionalidades similares ao padrão ERC-20 do Ethereum
#[multiversx_sc::contract]
pub trait LpToken {
    /// Função de inicialização que é chamada uma única vez quando o contrato é deployado
    /// @param initial_supply: Quantidade inicial de tokens a ser criada
    /// @param token_name: Nome completo do token (ex: "Bitcoin")
    /// @param token_ticker: Símbolo/código do token (ex: "BTC")
    /// @param token_decimals: Número de casas decimais do token
    #[init]
    fn init(
        &self,
        initial_supply: BigUint,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        token_decimals: u8,
    ) {
        // Armazena informações do token na blockchain
        let token_info = TokenInfo {
            name: token_name,
            ticker: token_ticker,
            decimals: token_decimals,
        };
        self.token_info().set(&token_info);

        // Cria (mint) os tokens iniciais e atribui ao criador do contrato
        let caller = self.blockchain().get_caller();
        self.mint(&caller, &initial_supply);

        // Inicializa configurações padrão do contrato
        self.paused().set(false);  // Contrato começa ativo (não pausado)
        self.fee_percentage().set(0u64);  // Sem taxa de transferência inicialmente (0%)
    }

    // ======== FUNÇÕES DE VISUALIZAÇÃO (VIEW) ========

    /// Retorna o nome completo do token
    #[view(getName)]
    fn get_name(&self) -> ManagedBuffer {
        self.token_info().get().name
    }

    /// Retorna o símbolo/ticker do token
    #[view(getTicker)]
    fn get_ticker(&self) -> ManagedBuffer {
        self.token_info().get().ticker
    }

    /// Retorna o número de casas decimais do token
    #[view(getDecimals)]
    fn get_decimals(&self) -> u8 {
        self.token_info().get().decimals
    }

    /// Retorna o suprimento total de tokens em circulação
    #[view(totalSupply)]
    fn total_supply(&self) -> BigUint {
        self.total_token_supply().get()
    }

    // ======== FUNÇÕES ERC-20 BÁSICAS ========

    /// Retorna o saldo de tokens de um determinado endereço
    /// @param address: Endereço a consultar o saldo
    #[view(balanceOf)]
    fn balance_of(&self, address: &ManagedAddress) -> BigUint {
        self.balances(address).get()
    }





    /// Transfere tokens do remetente (caller) para outro endereço.
    /// 
    /// # Parâmetros
    /// - `to`: Endereço de destino que receberá os tokens.
    /// - `amount`: Quantidade de tokens a transferir.
    /// 
    /// # Regras
    /// - A operação é bloqueada se o contrato estiver pausado.
    /// - A transferência considera a taxa de transação (se configurada).
    #[endpoint]
    fn transfer(&self, to: &ManagedAddress, amount: &BigUint) {
        // Garante que o contrato esteja ativo (não pausado)
        self.require_not_paused();

        // Obtém o endereço de quem chamou a função
        let caller = self.blockchain().get_caller();

        // Realiza a transferência com lógica de taxa embutida
        self.perform_transfer(&caller, to, amount);
    }

    /// Função auxiliar interna que verifica se o contrato está pausado. 
    /// # Comportamento
    /// - Se o contrato estiver pausado, a execução é interrompida com erro.
    /// - Essa função é usada internamente por funções como transfer, approve, etc.
    fn require_not_paused(&self) {
        require!(!self.paused().get(), "contract is paused");
    }


    /// Retorna quanto um spender está autorizado a gastar em nome de um owner
    /// @param owner: Dono dos tokens
    /// @param spender: Endereço autorizado a gastar
    #[view(allowance)]
    fn allowance(&self, owner: &ManagedAddress, spender: &ManagedAddress) -> BigUint {
        self.allowances(owner, spender).get()
    }

    /// Permite que um endereço autorizado transfira tokens em nome de outro
    /// @param from: Endereço de origem dos tokens
    /// @param to: Endereço de destino para receber os tokens
    /// @param amount: Quantidade de tokens a transferir
    #[endpoint(transferFrom)]
    fn transfer_from(
        &self,
        from: &ManagedAddress,
        to: &ManagedAddress,
        amount: &BigUint,
    ) -> () {
        self.require_not_paused();
        
        let caller = self.blockchain().get_caller();
        
        // Verifica se há allowance (permissão) suficiente
        let allowance = self.allowances(from, &caller).get();
        require!(
            &allowance >= amount,
            "insufficient allowance"
        );
        
        // Reduz o allowance pelo valor transferido
        self.allowances(from, &caller).set(&(&allowance - amount));
        
        // Realiza a transferência efetiva
        self.perform_transfer(from, to, amount)
    }

    // ======== FUNÇÕES DE MINT (CRIAR) E BURN (DESTRUIR) TOKENS ========

    /// Cria novos tokens e os atribui a um endereço (somente owner)
    /// @param to: Endereço que receberá os novos tokens
    /// @param amount: Quantidade de tokens a criar
    #[only_owner]
    #[endpoint(mint)]
    fn mint_endpoint(&self, to: &ManagedAddress, amount: &BigUint) -> () {
        self.mint(to, amount);
    }

    /// Destrói tokens de um endereço específico (somente owner)
    /// @param address: Endereço de onde os tokens serão destruídos
    /// @param amount: Quantidade de tokens a destruir
    #[only_owner]
    #[endpoint(burn)]
    fn burn_endpoint(&self, address: &ManagedAddress, amount: &BigUint) -> () {
        self.burn(address, amount)
    }

    /// Permite que um usuário destrua seus próprios tokens
    /// @param amount: Quantidade de tokens a destruir
    #[endpoint(burnOwn)]
    fn burn_own(&self, amount: &BigUint) -> () {
        let caller = self.blockchain().get_caller();
        self.burn(&caller, amount)
    }

    // ======== FUNÇÕES DE CONTROLE DO CONTRATO ========

    /// Pausa todas as operações do contrato (somente owner)
    /// Útil em caso de emergência ou manutenção
    #[only_owner]
    #[endpoint(pause)]
    fn pause(&self) -> () {
        self.paused().set(true);
    }

    /// Despausa o contrato, permitindo que as operações voltem ao normal (somente owner)
    #[only_owner]
    #[endpoint(unpause)]
    fn unpause(&self) -> () {
        self.paused().set(false);
    }

    /// Verifica se o contrato está pausado
    #[view(isPaused)]
    fn is_paused(&self) -> bool {
        self.paused().get()
    }

    // ======== GERENCIAMENTO DE TAXAS ========

    /// Define a porcentagem da taxa cobrada em transferências (somente owner)
    /// @param fee_percentage: Valor da taxa em basis points (1% = 100, 0.5% = 50)
    #[only_owner]
    #[endpoint(setFeePercentage)]
    fn set_fee_percentage(&self, fee_percentage: u64) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.blockchain().get_owner_address(), "unauthorized");
    
        require!(fee_percentage <= 10000, "fee percentage too high");
        self.fee_percentage().set(fee_percentage);
    }
    
    

    /// Retorna a porcentagem atual da taxa em basis points
    #[view(getFeePercentage)]
    fn get_fee_percentage(&self) -> u64 {
        self.fee_percentage().get()
    }

    #[endpoint]
    fn approve(&self, spender: &ManagedAddress, amount: &BigUint) {
        self.require_not_paused();
        let caller = self.blockchain().get_caller();
        self.allowances(&caller, spender).set(amount);
        self.approve_event(&caller, spender, amount);
    }

    // ======== FUNÇÕES PÚBLICAS DE MINT ========

    /// Permite que qualquer usuário receba tokens gratuitos (limitado a uma vez por endereço)
    /// Função demonstrativa - útil para faucets ou airdrops
    #[endpoint(publicMint)]
    fn public_mint(&self) -> () {
        self.require_not_paused();
        
        let caller = self.blockchain().get_caller();
        // Quantidade fixa de 10 tokens para simplificar o exemplo
        let mint_amount = BigUint::from(10u64);
        
        // Verifica se o usuário já recebeu tokens gratuitos antes
        require!(
            self.balance_of(&caller) == 0,
            "already claimed free tokens"
        );
        
        // Cria os tokens para o usuário
        self.mint(&caller, &mint_amount);
    }

    // ======== FUNÇÕES INTERNAS (AUXILIARES) ========

    /// Função interna para criar tokens
    /// @param to: Endereço que receberá os tokens
    /// @param amount: Quantidade de tokens a criar
    fn mint(&self, to: &ManagedAddress, amount: &BigUint) {
        if amount > &0 {
            // Atualiza o saldo do endereço
            let mut balance = self.balances(to).get();
            balance += amount;
            self.balances(to).set(&balance);
            
            // Atualiza o suprimento total de tokens
            let mut total_supply = self.total_token_supply().get();
            total_supply += amount;
            self.total_token_supply().set(&total_supply);
            
            // Emite evento de mint para indexadores e DApps
            self.mint_event(to, amount);
        }
    }

    /// Função interna para destruir tokens
    /// @param address: Endereço de onde os tokens serão destruídos
    /// @param amount: Quantidade de tokens a destruir
    fn burn(&self, address: &ManagedAddress, amount: &BigUint) -> () {
        // Verifica se há saldo suficiente
        let balance = self.balances(address).get();
        require!(
            &balance >= amount,
            "insufficient balance for burn"
        );
        
        // Atualiza o saldo do endereço
        self.balances(address).set(&(&balance - amount));
        
        // Atualiza o suprimento total de tokens
        let mut total_supply = self.total_token_supply().get();
        total_supply -= amount;
        self.total_token_supply().set(&total_supply);
        
        // Emite evento de burn para indexadores e DApps
        self.burn_event(address, amount);
    }

    /// Função interna que implementa a lógica de transferência com taxas
    /// @param from: Endereço de origem dos tokens
    /// @param to: Endereço de destino
    /// @param amount: Quantidade total a transferir (incluindo taxa)
    fn perform_transfer(
        &self,
        from: &ManagedAddress,
        to: &ManagedAddress,
        amount: &BigUint,
    ) -> () {
        // Verifica se há saldo suficiente
        let balance = self.balances(from).get();
        require!(
            &balance >= amount,
            "insufficient balance"
        );

        // Calcula a taxa se houver uma configurada
        let fee_percentage = self.fee_percentage().get();
        let fee_amount = if fee_percentage > 0 {
            // Cálculo: amount * fee_percentage / 10000
            // (usando basis points: 1% = 100 basis points)
            amount * &BigUint::from(fee_percentage) / &BigUint::from(10000u64)
        } else {
            BigUint::zero() // Sem taxa
        };

        // Valor líquido a ser transferido (após descontar a taxa)
        let transfer_amount = amount - &fee_amount;
        
        // Atualiza o saldo do remetente (deduz o valor total)
        self.balances(from).set(&(&balance - amount));
        
        // Atualiza o saldo do destinatário (adiciona o valor líquido)
        let mut to_balance = self.balances(to).get();
        to_balance += &transfer_amount;
        self.balances(to).set(&to_balance);
        
        // Se houver taxa, adiciona ao saldo do owner do contrato
        if fee_amount > 0 {
            let owner = self.blockchain().get_owner_address();
            let mut owner_balance = self.balances(&owner).get();
            owner_balance += &fee_amount;
            self.balances(&owner).set(&owner_balance);
        }
        
        // Emite evento de transferência para indexadores e DApps
        self.transfer_event(from, to, &transfer_amount, &fee_amount);
    }
  

    // ======== DEFINIÇÃO DE EVENTOS ========
    // Eventos são emitidos durante operações importantes e podem ser capturados por DApps e indexadores

    /// Evento emitido quando tokens são transferidos
    #[event("transfer")]
    fn transfer_event(
        &self,
        #[indexed] from: &ManagedAddress,  // Endereço de origem
        #[indexed] to: &ManagedAddress,    // Endereço de destino
        #[indexed] amount: &BigUint,       // Valor líquido transferido
        fee: &BigUint,                     // Valor da taxa (se houver)
    );

    /// Evento emitido quando uma aprovação é concedida
    #[event("approve")]
    fn approve_event(
        &self,
        #[indexed] owner: &ManagedAddress,    // Dono dos tokens
        #[indexed] spender: &ManagedAddress,  // Endereço autorizado a gastar
        #[indexed] amount: &BigUint,          // Quantidade aprovada
    );

    /// Evento emitido quando novos tokens são criados
    #[event("mint")]
    fn mint_event(
        &self,
        #[indexed] to: &ManagedAddress,     // Destinatário dos novos tokens
        #[indexed] amount: &BigUint,        // Quantidade criada
    );

    /// Evento emitido quando tokens são destruídos
    #[event("burn")]
    fn burn_event(
        &self,
        #[indexed] from: &ManagedAddress,   // Endereço de onde os tokens foram queimados
        #[indexed] amount: &BigUint,        // Quantidade destruída
    );

    // ======== DEFINIÇÃO DE STORAGE ========
    // Mappers são estruturas que permitem armazenar dados na blockchain

    /// Armazena as informações básicas do token (nome, ticker, decimais)
    #[storage_mapper("token_info")]
    fn token_info(&self) -> SingleValueMapper<TokenInfo<Self::Api>>;    

    /// Armazena o suprimento total de tokens em circulação
    #[storage_mapper("total_supply")]
    fn total_token_supply(&self) -> SingleValueMapper<BigUint>;

    /// Armazena o saldo de tokens de cada endereço
    #[storage_mapper("balances")]
    fn balances(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;

    /// Armazena as permissões de gastos concedidas entre endereços
    /// (quanto um endereço pode gastar em nome de outro)
    #[storage_mapper("allowances")]
    fn allowances(&self, owner: &ManagedAddress, spender: &ManagedAddress) -> SingleValueMapper<BigUint>;

    /// Armazena o estado de pausa do contrato (true = pausado)
    #[storage_mapper("paused")]
    fn paused(&self) -> SingleValueMapper<bool>;

    /// Armazena a porcentagem da taxa de transferência em basis points
    /// (1% = 100 basis points, 0.5% = 50 basis points)
    #[storage_mapper("fee_percentage")]
    fn fee_percentage(&self) -> SingleValueMapper<u64>;
}