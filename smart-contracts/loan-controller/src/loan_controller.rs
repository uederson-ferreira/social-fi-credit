// ==========================================================================
// MÓDULO: loan-controller/src/lib.rs
// Descrição: Contrato inteligente que gerencia o sistema de empréstimos baseado 
//            em pontuação social na blockchain MultiversX
// ==========================================================================

#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod reputation_score_proxy {
    multiversx_sc::imports!();
    
    #[multiversx_sc::proxy]
    pub trait ReputationScore {
        #[endpoint(isEligibleForLoan)]
        fn is_eligible_for_loan(&self, user: ManagedAddress, min_score: u64) -> bool;
        
        #[endpoint(calculateMaxLoanAmount)]
        fn calculate_max_loan_amount(&self, user: ManagedAddress, base_amount: BigUint) -> BigUint;
        
        #[endpoint(getUserScore)]
        fn get_user_score(&self, user: ManagedAddress) -> u64;
    }
}

#[multiversx_sc::contract]
pub trait LoanController {
    // Inicializa o contrato com os parâmetros básicos
    #[init]
    fn init(
        &self,
        reputation_score_address: ManagedAddress,  // Endereço do contrato ReputationScore
        min_required_score: u64,                   // Pontuação mínima para empréstimo
        interest_rate_base: u64,                   // Taxa de juros base (em pontos base, 1000 = 10%)
        base_loan_amount: BigUint,                 // Valor base para cálculo do empréstimo máximo
    ) {
        self.reputation_score_address().set(reputation_score_address);
        self.min_required_score().set(min_required_score);
        self.interest_rate_base().set(interest_rate_base);
        self.base_loan_amount().set(base_loan_amount);
    }

    // Solicita um empréstimo
    #[payable("*")]
    #[endpoint(requestLoan)]
    fn request_loan(&self, amount: BigUint, duration_days: u64) {
        let caller = self.blockchain().get_caller();

        // Verifica se o usuário tem pontuação suficiente
        let rs_address = self.reputation_score_address().get();
        
        // Obtém a elegibilidade do usuário para o empréstimo
        // Vamos verificar a elegibilidade do usuário para o empréstimo
        let min_score = self.min_required_score().get();
        
        self.reputation_score_proxy(rs_address.clone())
            .is_eligible_for_loan(caller.clone(), min_score)
            .with_callback(self.callbacks().check_eligibility_callback(
                caller.clone(),
                amount.clone(),
                duration_days
            ))
            .call_and_exit();
    }
    
    // Paga um empréstimo
    #[payable("*")]
    #[endpoint(repayLoan)]
    fn repay_loan(&self, loan_id: u64) {
        let caller = self.blockchain().get_caller();

        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");

        let mut loan = self.loans(loan_id).get();
        require!(loan.borrower == caller, "Apenas o tomador pode pagar o empréstimo");
        require!(loan.status == LoanStatus::Active, "Empréstimo não está ativo");

        // Extrai due_timestamp antes de mover loan
        let due_ts = loan.due_timestamp;

        // Atualiza status para pago
        loan.status = LoanStatus::Repaid;
        self.loans(loan_id).set(loan);

        // Contabiliza pagamento em dia
        let current_timestamp = self.blockchain().get_block_timestamp();
        if current_timestamp <= due_ts {
            self.on_time_payments(caller.clone()).update(|count| *count += 1);
        }
    }

    // Callbacks para processamento assíncrono
    #[callback]
    fn check_eligibility_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<bool>,
        caller: ManagedAddress,
        amount: BigUint,
        duration_days: u64,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(is_eligible) => {
                if !is_eligible {
                    require!(false, "Pontuação do usuário muito baixa para empréstimo");
                    return;
                }
                
                // Continua com a verificação do valor máximo
                let rs_address = self.reputation_score_address().get();
                let base_amount = self.base_loan_amount().get();
                
                self.reputation_score_proxy(rs_address.clone())
                    .calculate_max_loan_amount(caller.clone(), base_amount)
                    .with_callback(self.callbacks().check_amount_callback(
                        caller,
                        amount,
                        duration_days
                    ))
                    .call_and_exit();
            },
            ManagedAsyncCallResult::Err(_) => {
                require!(false, "Erro ao verificar elegibilidade do usuário");
            }
        }
    }
    
    #[callback]
    fn check_amount_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
        caller: ManagedAddress,
        amount: BigUint,
        duration_days: u64,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(max_amount) => {
                if amount > max_amount {
                    require!(false, "Valor solicitado excede o limite permitido");
                    return;
                }
                
                // Continua com a obtenção da pontuação do usuário
                let rs_address = self.reputation_score_address().get();
                
                self.reputation_score_proxy(rs_address)
                    .get_user_score(caller.clone())
                    .with_callback(self.callbacks().process_loan_callback(
                        caller,
                        amount,
                        duration_days
                    ))
                    .call_and_exit();
            },
            ManagedAsyncCallResult::Err(_) => {
                require!(false, "Erro ao calcular valor máximo do empréstimo");
            }
        }
    }
    
    #[callback]
    fn process_loan_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<u64>,
        caller: ManagedAddress,
        amount: BigUint,
        duration_days: u64,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(user_score) => {
                // Calcula a taxa de juros com base na pontuação do usuário
                let interest_rate = self.calculate_interest_rate(user_score);
        
                // Calcula o valor total a ser pago
                let interest_amount = &amount * &BigUint::from(interest_rate) / &BigUint::from(10000u32);
                let repayment_amount = &amount + &interest_amount;
        
                // Cria o registro do empréstimo
                let loan_id = self.loan_counter().get();
                self.loan_counter().set(loan_id + 1);
        
                let current_timestamp = self.blockchain().get_block_timestamp();
                let due_timestamp = current_timestamp + duration_days * 86400;
        
                self.loans(loan_id).set(Loan {
                    borrower: caller.clone(),
                    amount: amount.clone(),
                    repayment_amount,
                    interest_rate,
                    creation_timestamp: current_timestamp,
                    due_timestamp,
                    status: LoanStatus::Active,
                });
        
                // Registra o empréstimo para consulta do usuário
                self.user_loans(caller.clone()).push(&loan_id);
        
                // Transfere os fundos para o usuário
                let token_id = self.call_value().egld_or_single_esdt().token_identifier;
                self.send().direct(&caller, &token_id, 0, &amount);
            },
            ManagedAsyncCallResult::Err(_) => {
                require!(false, "Erro ao obter pontuação do usuário");
            }
        }
    }

    // Calcula a taxa de juros com base na pontuação do usuário
    fn calculate_interest_rate(&self, user_score: u64) -> u64 {
        let base_rate = self.interest_rate_base().get();
        let max_score = 1000u64;

        let score_factor = (user_score * 80) / max_score;
        if score_factor >= 100 {
            return base_rate / 5;
        }
        base_rate * (100 - score_factor) / 100
    }

    // Storage mappers
    #[storage_mapper("reputation_score_address")]
    fn reputation_score_address(&self) -> SingleValueMapper<ManagedAddress>;
    #[storage_mapper("min_required_score")]
    fn min_required_score(&self) -> SingleValueMapper<u64>;
    #[storage_mapper("interest_rate_base")]
    fn interest_rate_base(&self) -> SingleValueMapper<u64>;
    #[storage_mapper("base_loan_amount")]
    fn base_loan_amount(&self) -> SingleValueMapper<BigUint>;
    #[storage_mapper("loan_counter")]
    fn loan_counter(&self) -> SingleValueMapper<u64>;
    #[storage_mapper("loans")]
    fn loans(&self, loan_id: u64) -> SingleValueMapper<Loan<Self::Api>>;
    #[storage_mapper("user_loans")]
    fn user_loans(&self, user: ManagedAddress) -> VecMapper<u64>;
    #[storage_mapper("on_time_payments")]
    fn on_time_payments(&self, user: ManagedAddress) -> SingleValueMapper<u64>;

    // Proxy para o contrato ReputationScore
    #[proxy]
    fn reputation_score_proxy(&self, address: ManagedAddress) -> reputation_score_proxy::Proxy<Self::Api>;
}

/*
Status do empréstimo
*/
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq)]
pub enum LoanStatus {
    Active,
    Repaid,
    Defaulted,
}

// Dados do empréstimo
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Loan<M: ManagedTypeApi> {
    pub borrower: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub repayment_amount: BigUint<M>,
    pub interest_rate: u64,
    pub creation_timestamp: u64,
    pub due_timestamp: u64,
    pub status: LoanStatus,
}