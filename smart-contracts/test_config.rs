// test_config.rs

use multiversx_sc::types::{Address, BigUint};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, rust_biguint,
    testing_framework::BlockchainStateWrapper,
    DebugApi,
};

// Constantes compartilhadas
pub const LOAN_CONTROLLER_WASM_PATH: &str = "output/loan-controller.wasm";
pub const REPUTATION_SCORE_WASM_PATH: &str = "output/reputation-score.wasm";
pub const DEBT_TOKEN_WASM_PATH: &str = "output/debt-token.wasm";
pub const LIQUIDITY_POOL_WASM_PATH: &str = "output/liquidity-pool.wasm";
pub const LP_TOKEN_WASM_PATH: &str = "output/lp-token.wasm";

// Estrutura básica para testes
pub struct TestWallets {
    pub owner: Address,
    pub provider1: Address,
    pub provider2: Address,
    pub borrower1: Address,
    pub borrower2: Address,
    pub attacker: Address,
}

// Função para criar carteiras de teste
pub fn setup_test_wallets(blockchain: &mut BlockchainStateWrapper) -> TestWallets {
    let owner = blockchain.create_user_account(&rust_biguint!(0));
    let provider1 = blockchain.create_user_account(&rust_biguint!(200000));
    let provider2 = blockchain.create_user_account(&rust_biguint!(300000));
    let borrower1 = blockchain.create_user_account(&rust_biguint!(10000));
    let borrower2 = blockchain.create_user_account(&rust_biguint!(15000));
    let attacker = blockchain.create_user_account(&rust_biguint!(50000));
    
    TestWallets {
        owner,
        provider1,
        provider2,
        borrower1,
        borrower2,
        attacker,
    }
}

// Funções de utilidade para cálculos comuns
pub fn calculate_interest_amount(principal: &BigUint, rate: u64, duration_days: u64) -> BigUint {
    let annual_factor = BigUint::from(365u64);
    let rate_factor = BigUint::from(rate);
    let duration_factor = BigUint::from(duration_days);
    let base_factor = BigUint::from(10000u64);
    
    principal * rate_factor * duration_factor / (base_factor * annual_factor)
}

// Funções para verificações comuns
pub fn verify_loan_consistency(
    loan_amount: &BigUint,
    repayment_amount: &BigUint,
    interest_rate: u64
) -> bool {
    let interest = loan_amount * BigUint::from(interest_rate) / BigUint::from(10000u64);
    let expected_repayment = loan_amount + interest;
    
    expected_repayment == *repayment_amount
}