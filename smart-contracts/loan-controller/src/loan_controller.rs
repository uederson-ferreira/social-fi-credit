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

    #[endpoint]
    fn set_min_interest_rate(&self, rate: u64) {
        self.min_interest_rate().set(rate);
    }
    
    #[endpoint]
    fn set_max_interest_rate(&self, rate: u64) {
        self.max_interest_rate().set(rate);
    }
    
    #[view(getMinInterestRate)]
    fn get_min_interest_rate(&self) -> u64 {
        self.min_interest_rate().get()
    }
    
    #[view(getMaxInterestRate)]
    fn get_max_interest_rate(&self) -> u64 {
        self.max_interest_rate().get()
    }

    #[storage_mapper("mock_timestamp")]
    fn mock_timestamp(&self) -> SingleValueMapper<u64>;

    #[endpoint]
    fn set_mock_timestamp(&self, timestamp: u64) {
        self.mock_timestamp().set(timestamp);
    }

    #[view]
    fn get_block_timestamp(&self) -> u64 {
        self.mock_timestamp().get()
    }

    #[endpoint]
    fn set_max_active_loans(&self, max_loans: u64) {
        self.max_active_loans().set(max_loans);
    }

    #[view(getMaxActiveLoans)]
    fn get_max_active_loans(&self) -> u64 {
        self.max_active_loans().get()
    }

    //================================================

    #[endpoint(requestLoanSync)]
    fn request_loan_sync(&self, amount: BigUint, duration_days: u64) -> u64 {
        let caller = self.blockchain().get_caller();

        // Verificar se o usuário já atingiu o limite de empréstimos ativos
        let active_loans = self.user_loans(caller.clone()).len();
        let max_active_loans = self.max_active_loans().get();
        require!(active_loans < max_active_loans as usize, "Limite de empréstimos ativos atingido");

        // Gerar um novo ID de empréstimo
        let loan_id = self.loan_counter().get();
        self.loan_counter().set(loan_id + 1);

        // Calcular o valor total a ser pago
        let interest_rate = self.interest_rate_base().get();
        let interest_amount = &amount * &BigUint::from(interest_rate) / &BigUint::from(10000u64);
        let repayment_amount = &amount + &interest_amount;

        // Criar o empréstimo
        let current_timestamp = self.blockchain().get_block_timestamp();
        let due_timestamp = current_timestamp + duration_days * 86400;

        let loan = Loan {
            borrower: caller.clone(),
            amount: amount.clone(),
            repayment_amount,
            interest_rate,
            creation_timestamp: current_timestamp,
            due_timestamp,
            status: LoanStatus::Active,
        };

        // Armazenar o empréstimo
        self.loans(loan_id).set(loan);

        // Associar o empréstimo ao usuário
        self.user_loans(caller.clone()).push(&loan_id);

        // Retornar o ID do empréstimo
        loan_id
    }

    #[endpoint]
    fn mark_expired_loans(&self) {
        let current_timestamp = self.blockchain().get_block_timestamp();
    
        // Iterate over all loans
        let loan_counter = self.loan_counter().get();
        for loan_id in 0..loan_counter {
            let mut loan = self.loans(loan_id).get(); // Use `get` to retrieve the loan
            // Check if the loan is active and expired
            if loan.status == LoanStatus::Active && current_timestamp >= loan.due_timestamp {
                // Mark as defaulted
                loan.status = LoanStatus::Defaulted;
                self.loans(loan_id).set(loan);
            }
        }
    }

    // Adicione um endpoint para configurar a taxa de extensão
    #[endpoint(setExtensionFeePercent)]
    fn set_extension_fee_percent(&self, fee_percent: u64) {
        require!(fee_percent <= 10000, "A taxa de extensão não pode exceder 100%");
        self.extension_fee_percent().set(fee_percent);
    }

    // Adicione a view para acessar a taxa de extensão
    #[view(getExtensionFeePercent)]
    fn get_extension_fee_percent(&self) -> u64 {
        self.extension_fee_percent().get()
    }

    // Adicione um endpoint para configurar a taxa diária de atraso
    #[endpoint(setLateFeeDailyRate)]
    fn set_late_fee_daily_rate(&self, rate: u64) {
        require!(rate <= 10000, "A taxa diária de atraso não pode exceder 100%");
        self.late_fee_daily_rate().set(rate);
    }

    // Adicione uma view para consultar a taxa diária de atraso
    #[view(getLateFeeDailyRate)]
    fn get_late_fee_daily_rate(&self) -> u64 {
        self.late_fee_daily_rate().get()
    }

    // Endpoints para configuração
    #[endpoint(setCollateralRatio)]
    fn set_collateral_ratio(&self, ratio: u64) {
        require!(ratio <= 10000, "A razão de garantia não pode exceder 100%");
        self.collateral_ratio().set(ratio);
    }

    #[endpoint(setLiquidationDiscount)]
    fn set_liquidation_discount(&self, discount: u64) {
        require!(discount <= 10000, "O desconto de liquidação não pode exceder 100%");
        self.liquidation_discount().set(discount);
    }

    // #[endpoint]
    // fn request_loan(&self, amount: BigUint, interest_rate: u64) -> u64 {
    //     let loan_id = self.loan_counter().get() + 1;
    //     self.loan_counter().set(loan_id);

    //     let loan = Loan {
    //         borrower: self.blockchain().get_caller(),
    //         amount,
    //         interest_rate,
    //         status: LoanStatus::Active,
    //         creation_timestamp: self.blockchain().get_block_timestamp(),
    //         due_timestamp: self.blockchain().get_block_timestamp() + 30 * 24 * 60 * 60, // 30 days
    //     };

    //     self.loans(loan_id).set(loan);
    //     loan_id
    // }

    // Views para consulta
    #[view(getCollateralRatio)]
    fn get_collateral_ratio(&self) -> u64 {
        self.collateral_ratio().get()
    }

    #[view(getLiquidationDiscount)]
    fn get_liquidation_discount(&self) -> u64 {
        self.liquidation_discount().get()
    }

    // Endpoint para adicionar ou atualizar um investidor
    #[endpoint]
    fn add_investor(&self, investor: ManagedAddress, shares: u64) {
        let current_shares = self.investor_shares(&investor).get();
        self.investor_shares(&investor).set(current_shares + shares);

        let total_shares = self.total_investor_shares().get();
        self.total_investor_shares().set(total_shares + shares);
    }

    // View para consultar as participações de um investidor
    #[view(getInvestorShares)]
    fn get_investor_shares(&self, investor: ManagedAddress) -> u64 {
        self.investor_shares(&investor).get()
    }
    // View para consultar o total de participações
    #[view(getTotalInvestorShares)]
    fn get_total_investor_shares(&self) -> u64 {
        self.total_investor_shares().get()
    }

    #[endpoint]
    fn set_standard_loan_term_days(&self, days: u64) {
        self.standard_loan_term_days().set(days);
    }
    
    #[endpoint]
    fn set_extended_loan_term_days(&self, days: u64) {
        self.extended_loan_term_days().set(days);
    }
    
    #[endpoint]
    fn set_max_loan_term_days(&self, days: u64) {
        self.max_loan_term_days().set(days);
    }

    #[endpoint]
    fn reputation_check_callback(
        &self,
        user_address: ManagedAddress,
        score: u64,
    ) {
        self.user_reputation_scores(&user_address).set(score);
    }
    
    #[view(getStandardLoanTermDays)]
    fn get_standard_loan_term_days(&self) -> u64 {
        self.standard_loan_term_days().get()
    }
    
    #[view(getExtendedLoanTermDays)]
    fn get_extended_loan_term_days(&self) -> u64 {
        self.extended_loan_term_days().get()
    }
    
    #[view(getMaxLoanTermDays)]
    fn get_max_loan_term_days(&self) -> u64 {
        self.max_loan_term_days().get()
    }

    // Endpoints para configuração
    #[endpoint]
    fn set_interest_rate_base(&self, rate: u64) {
        self.interest_rate_base().set(rate);
    }

    #[endpoint]
    fn set_extended_term_rate_multiplier(&self, multiplier: u64) {
        self.extended_term_rate_multiplier().set(multiplier);
    }

    #[endpoint]
    fn set_max_term_rate_multiplier(&self, multiplier: u64) {
        self.max_term_rate_multiplier().set(multiplier);
    }

    #[endpoint]
    fn set_min_required_score(&self, score: u64) {
        require!(self.blockchain().get_caller() == self.blockchain().get_owner_address(), "Only owner can call this function");
        self.min_required_score().set(score);
    }

    #[view(getMinRequiredScore)]
    fn get_min_required_score(&self) -> u64 {
        self.min_required_score().get()
    }

    // Views para consulta
    #[view(getInterestRateBase)]
    fn get_interest_rate_base(&self) -> u64 {
        self.interest_rate_base().get()
    }

    #[view(getExtendedTermRateMultiplier)]
    fn get_extended_term_rate_multiplier(&self) -> u64 {
        self.extended_term_rate_multiplier().get()
    }

    #[view(getMaxTermRateMultiplier)]
    fn get_max_term_rate_multiplier(&self) -> u64 {
        self.max_term_rate_multiplier().get()
    }

    #[view(calculateDueDateSafely)]
    fn calculate_due_date_safely(&self, term_in_seconds: u64) -> u64 {
        let current_timestamp = self.blockchain().get_block_timestamp();
        let max_seconds = 3650u64 * 24u64 * 60u64 * 60u64; // 10 years in seconds

        // Ensure the term does not exceed the maximum allowed duration
        let safe_term = core::cmp::min(term_in_seconds, max_seconds);

        // Calculate the due date
        current_timestamp + safe_term
    }

    #[view(calculateLoanAmountWithLimits)]
    fn calculate_loan_amount_with_limits(&self, base_amount: BigUint) -> BigUint {
        let min_amount = self.base_loan_amount().get();
        let max_amount = self.max_loan_amount().get();

        if base_amount < min_amount {
            BigUint::from(min_amount)
        } else if base_amount > max_amount {
            BigUint::from(max_amount)
        } else {
            base_amount
        }
    }

    #[view(getActiveLoansCount)]
    fn get_active_loans_count(&self) -> u64 {
        self.active_loans_count().get()
    }

    #[endpoint]
    fn set_operation_timelock(&self, timelock: u64) {
        self.operation_timelock().set(timelock);
    }

    #[view(getOperationTimelock)]
    fn get_operation_timelock(&self) -> u64 {
        self.operation_timelock().get()
    }
    //================================================

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
    
    //#[storage_mapper("interest_rate_base")]
    //fn interest_rate_base(&self) -> SingleValueMapper<u64>;

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

    //=====================================================================
    // Adicione o mapper de armazenamento para a taxa de extensão
    #[storage_mapper("extension_fee_percent")]
    fn extension_fee_percent(&self) -> SingleValueMapper<u64>;
    
    // Adicione o mapper de armazenamento para a taxa diária de atraso
    #[storage_mapper("late_fee_daily_rate")]
    fn late_fee_daily_rate(&self) -> SingleValueMapper<u64>;

    // Mappers de armazenamento
    #[storage_mapper("collateral_ratio")]
    fn collateral_ratio(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("liquidation_discount")]
    fn liquidation_discount(&self) -> SingleValueMapper<u64>;

    // Mappers de armazenamento para investidores
    #[storage_mapper("investor_shares")]
    fn investor_shares(&self, investor: &ManagedAddress) -> SingleValueMapper<u64>;

    #[storage_mapper("total_investor_shares")]
    fn total_investor_shares(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("standard_loan_term_days")]
    fn standard_loan_term_days(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("extended_loan_term_days")]
    fn extended_loan_term_days(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("max_loan_term_days")]
    fn max_loan_term_days(&self) -> SingleValueMapper<u64>;

    // Mappers de armazenamento para taxas de juros e multiplicadores
    #[storage_mapper("interest_rate_base")]
    fn interest_rate_base(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("extended_term_rate_multiplier")]
    fn extended_term_rate_multiplier(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("max_term_rate_multiplier")]
    fn max_term_rate_multiplier(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("min_loan_amount")]
    fn min_loan_amount(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("max_loan_amount")]
    fn max_loan_amount(&self) -> SingleValueMapper<BigUint>;    

    #[storage_mapper("min_interest_rate")]
    fn min_interest_rate(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("max_interest_rate")]
    fn max_interest_rate(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("user_reputation_scores")]
    fn user_reputation_scores(&self, user: &ManagedAddress) -> SingleValueMapper<u64>;
       
    #[storage_mapper("max_active_loans")]
    fn max_active_loans(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("active_loans_count")]
    fn active_loans_count(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("operation_timelock")]
    fn operation_timelock(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("pending_parameter_changes")]
    fn pending_parameter_changes(&self, param_type: ParamType) -> SingleValueMapper<ParameterChange>;

    #[storage_mapper("pending_parameter_changes_by_key")]
    fn pending_parameter_changes_by_key(&self, param_key: BigUint) -> SingleValueMapper<ParameterChange>;
    //=====================================================================

    // Proxy para o contrato ReputationScore
    #[proxy]
    fn reputation_score_proxy(&self, address: ManagedAddress) -> reputation_score_proxy::Proxy<Self::Api>;
}

/*Status do empréstimo*/
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug, Clone)]
pub enum LoanStatus {
    Active,
    Repaid,
    Defaulted,
}
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ParamType {
    MinScore,
    MaxScore,
    InterestRate,
}

#[derive(Debug, PartialEq, Eq, Clone, TopEncode, TopDecode, TypeAbi)]
pub struct ParameterChange {
    pub value: u64,       // The new value for the parameter
    pub timestamp: u64,   // The timestamp when the change was requested
}
// Dados do empréstimo
#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone)]
pub struct Loan<M: ManagedTypeApi> {
    pub borrower: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub repayment_amount: BigUint<M>,
    pub interest_rate: u64,
    pub creation_timestamp: u64,
    pub due_timestamp: u64,
    pub status: LoanStatus,
}