#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ReputationScore {
    #[init]
    fn init(&self, min_score: u64, max_score: u64) {
        self.min_score().set(min_score);
        self.max_score().set(max_score);
    }

    // Sets or updates the social score for a user
    #[endpoint]
    fn update_score(&self, user_address: ManagedAddress, score: u64) -> SCResult<()> {
        // Only oracle can update scores
        require!(
            self.blockchain().get_caller() == self.oracle_address().get(),
            "Only oracle can update scores"
        );
        
        // Validate score range
        require!(
            score >= self.min_score().get() && score <= self.max_score().get(),
            "Score out of valid range"
        );
        
        self.user_score(user_address).set(score);
        
        Ok(())
    }
    
    // Get user's current score
    #[view(getUserScore)]
    fn get_user_score(&self, user_address: ManagedAddress) -> u64 {
        let default_score = self.min_score().get();
        if self.user_score(user_address).is_empty() {
            return default_score;
        }
        
        self.user_score(user_address).get()
    }
    
    // Check if user is eligible for loan based on minimum score
    #[view(isEligibleForLoan)]
    fn is_eligible_for_loan(&self, user_address: ManagedAddress, required_score: u64) -> bool {
        let user_score = self.get_user_score(user_address);
        user_score >= required_score
    }
    
    // Calculate max loan amount based on user score
    #[view(calculateMaxLoanAmount)]
    fn calculate_max_loan_amount(&self, user_address: ManagedAddress, base_amount: BigUint) -> BigUint {
        let user_score = self.get_user_score(user_address);
        let max_score = self.max_score().get();
        
        // Simple formula: base_amount * (user_score / max_score) * 2
        let user_score_big = BigUint::from(user_score);
        let max_score_big = BigUint::from(max_score);
        
        base_amount * user_score_big * 2u32 / max_score_big
    }
    
    // Storage
    #[storage_mapper("oracle_address")]
    fn oracle_address(&self) -> SingleValueMapper<ManagedAddress>;
    
    #[storage_mapper("min_score")]
    fn min_score(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("max_score")]
    fn max_score(&self) -> SingleValueMapper<u64>;
    
    #[storage_mapper("user_score")]
    fn user_score(&self, user_address: ManagedAddress) -> SingleValueMapper<u64>;
}
