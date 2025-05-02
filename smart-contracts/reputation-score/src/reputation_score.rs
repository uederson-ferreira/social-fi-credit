// ==========================================================================
// MÓDULO: reputation-score/src/lib.rs
// Descrição: Contrato inteligente que gerencia pontuações de reputação social
//            na blockchain MultiversX para avaliar elegibilidade de empréstimos
// ==========================================================================

#![no_std]
multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ReputationScore {
    // Inicializa o contrato com valores mínimos e máximos para a pontuação
    #[init]
    fn init(&self, min_score: u64, max_score: u64) {
        require!(min_score < max_score, "Min score must be less than max score");
        self.min_score().set(min_score);
        self.max_score().set(max_score);
    }

    // Define o endereço do oráculo autorizado a atualizar pontuações
    #[only_owner]
    #[endpoint(setOracleAddress)]
    fn set_oracle_address(&self, oracle_address: ManagedAddress) {
        require!(!oracle_address.is_zero(), "Oracle address cannot be zero");
        self.oracle_address().set(oracle_address);
    }

    // Atualiza a pontuação de reputação de um usuário (somente oráculo)
    #[endpoint(updateScore)]
    fn update_score(&self, user_address: ManagedAddress, score: u64) {
        // Verifica se o oráculo já foi configurado
        require!(!self.oracle_address().is_empty(), "Oracle not configured");

        // Apenas o oráculo configurado pode chamar
        require!(
            self.blockchain().get_caller() == self.oracle_address().get(),
            "Only oracle can update scores"
        );

        // Score deve estar no intervalo permitido
        require!(
            score >= self.min_score().get() && score <= self.max_score().get(),
            "Score out of valid range"
        );

        self.user_score(user_address.clone()).set(score);
        self.score_updated_event(user_address, score);
    }

    // Retorna a pontuação do usuário, ou o valor mínimo se ainda não houver
    #[view(getUserScore)]
    fn get_user_score(&self, user_address: ManagedAddress) -> u64 {
        let default_score = self.min_score().get();
        if self.user_score(user_address.clone()).is_empty() {
            default_score
        } else {
            self.user_score(user_address).get()
        }
    }

    // Verifica elegibilidade para empréstimo
    #[view(isEligibleForLoan)]
    fn is_eligible_for_loan(&self, user_address: ManagedAddress, required_score: u64) -> bool {
        self.get_user_score(user_address) >= required_score
    }

    // Calcula valor máximo de empréstimo: base_amount * (user_score / max_score) * 2
    #[view(calculateMaxLoanAmount)]
    fn calculate_max_loan_amount(&self, user_address: ManagedAddress, base_amount: BigUint) -> BigUint {
        let user_score = self.get_user_score(user_address);
        let max_score = self.max_score().get();
        require!(max_score > 0, "Max score cannot be zero");

        let user_score_big = BigUint::from(user_score);
        let max_score_big = BigUint::from(max_score);

        base_amount * user_score_big * 2u32 / max_score_big
    }

    // Evento de atualização de score
    #[event("score_updated")]
    fn score_updated_event(&self, #[indexed] user_address: ManagedAddress, #[indexed] score: u64);

    // --- Storage mappers ---
    #[storage_mapper("oracle_address")]
    fn oracle_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("min_score")]
    fn min_score(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("max_score")]
    fn max_score(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("user_score")]
    fn user_score(&self, user_address: ManagedAddress) -> SingleValueMapper<u64>;
}
