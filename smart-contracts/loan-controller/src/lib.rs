#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait LoanController {
    #[init]
    fn init(
        &self,
        reputation_score_address: ManagedAddress,
        min_required_score: u64,
        interest_rate_base: u64,
    ) {
        self.reputation_score_address().set(reputation_score_address);
        self.min_required_score().set(min_required_score);
        self.interest_rate_base().set(interest_rate_base);
    }

    // Request a loan
    #[payable("*")]
    #[endpoint(requestLoan)]
    fn request_loan(&self, amount: BigUint, duration_days: u64) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        
        // Check if user has sufficient score
        let rs_proxy = self.reputation_score_proxy(self.reputation_score_address().get());
        require!(
            rs_proxy.is_eligible_for_loan(caller.clone(), self.min_required_score().get()),
            "User score too low for loan"
        );
        
        // Check if amount is within user's limit
        let max_amount = rs_proxy.calculate_max_loan_amount(caller.clone(), self.base_loan_amount().get());
        require!(
            amount <= max_amount,
            "Requested amount exceeds maximum allowed"
        );
        
        // Calculate interest rate based on user's score
        let user_score = rs_proxy.get_user_score(caller.clone());
        let interest_rate = self.calculate_interest_rate(user_score);
        
        // Calculate repayment amount
        let interest_amount = &amount * &BigUint::from(interest_rate) / &BigUint::from(10000u32);
        let repayment_amount = &amount + &interest_amount;
        
        // Create loan
        let loan_id = self.loan_counter().get();
        self.loan_counter().set(loan_id + 1);
        
        let current_timestamp = self.blockchain().get_block_timestamp();
        let due_timestamp = current_timestamp + duration_days * 86400; // 86400 seconds = 1 day
        
        self.loans(loan_id).set(Loan {
            borrower: caller.clone(),
            amount: amount.clone(),
            repayment_amount,
            interest_rate,
            creation_timestamp: current_timestamp,
            due_timestamp,
            status: LoanStatus::Active,
        });
        
        // Associate loan with user
        self.user_loans(caller).push(&loan_id);
        
        // Transfer funds to user
        let token_id = self.call_value().egld_or_single_esdt().token_identifier;
        self.send().direct(&caller, &token_id, 0, &amount);
        
        Ok(())
    }
    
    // Repay a loan
    #[payable("*")]
    #[endpoint(repayLoan)]
    fn repay_loan(&self, loan_id: u64) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        
        require!(
            !self.loans(loan_id).is_empty(),
            "Loan does not exist"
        );
        
        let mut loan = self.loans(loan_id).get();
        
        require!(
            loan.borrower == caller,
            "Only borrower can repay loan"
        );
        
        require!(
            loan.status == LoanStatus::Active,
            "Loan is not active"
        );
        
        // Check payment amount
        let payment = self.call_value().egld_or_single_esdt();
        require!(
            payment.amount == loan.repayment_amount,
            "Payment amount does not match repayment amount"
        );
        
        // Update loan status
        loan.status = LoanStatus::Repaid;
        self.loans(loan_id).set(loan);
        
        // Update user score if paid on time
        let current_timestamp = self.blockchain().get_block_timestamp();
        if current_timestamp <= loan.due_timestamp {
            // Positive score update would be triggered by the oracle
            // but we could record the on-time payment
            self.on_time_payments(caller).update(|count| *count += 1);
        }
        
        // Transfer funds to liquidity pool
        // In a real implementation, this would distribute to investors
        
        Ok(())
    }
    
    // Calculate interest rate based on user score
    fn calculate_interest_rate(&self, user_score: u64) -> u64 {
        let base_rate = self.interest_rate_base().get();
        let max_score = 1000u64; // Assume max score is 1000
        
        // Formula: base_rate * (1 - (user_score / max_score) * 0.8)
        // This means higher score = lower interest rate
        // E.g., if base_rate = 1000 (10%), a max score user would pay 2%
        
        let score_factor = (user_score * 80) / max_score; // 0-80 range
        if score_factor >= 100 { 
            return base_rate / 5; // Minimum 20% of base rate
        }
        
        base_rate * (100 - score_factor) / 100
    }
    
    // Storage
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
    
    // Proxy to reputation score contract
    #[proxy]
    fn reputation_score_proxy(&self, address: ManagedAddress) -> reputation_score::Proxy<Self::Api>;
}

// Loan status enum
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq)]
pub enum LoanStatus {
    Active,
    Repaid,
    Defaulted,
}

// Loan struct
#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Loan<M: ManagedTypeApi> {
    pub borrower: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub repayment_amount: BigUint<M>,
    pub interest_rate: u64,
    pub creation_timestamp: u64,
    pub due_timestamp: u64,
    pub status: LoanStatus,
}
