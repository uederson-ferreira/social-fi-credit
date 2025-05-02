#![no_std]

// Importações completas do MultiversX SC
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

// Structs com anotações na ordem correta conforme documentação
#[multiversx_sc::derive::type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct ProviderFunds<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub amount: BigUint<M>,
    pub last_yield_timestamp: u64,
}

#[multiversx_sc::derive::type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Loan<M: ManagedTypeApi> {
    pub borrower: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub token_id: TokenIdentifier<M>,
    pub interest_rate: u64,
    pub creation_timestamp: u64,
    pub due_timestamp: u64,
    pub is_repaid: bool,
    pub collateral_token_id: TokenIdentifier<M>,
    pub collateral_amount: BigUint<M>,
}

// Interfaces com trait bounds necessários
pub trait ILiquidityPool: multiversx_sc::contract_base::ContractBase {
    // Fornece fundos para um empréstimo
    fn provide_funds_for_loan(&self, amount: BigUint<Self::Api>, token_id: TokenIdentifier<Self::Api>);
    
    // Recebe pagamento de empréstimo
    fn receive_loan_repayment(&self);
    
    // Obtém liquidez total disponível
    fn get_total_liquidity(&self) -> BigUint<Self::Api>;
}

pub trait IReputationScore: multiversx_sc::contract_base::ContractBase {
    // Obtém a pontuação de reputação de um endereço
    fn get_score(&self, address: &ManagedAddress<Self::Api>) -> BigUint<Self::Api>;
    
    // Atualiza a pontuação após pagamento de empréstimo
    fn update_score_after_repayment(&self, address: &ManagedAddress<Self::Api>, amount: &BigUint<Self::Api>);
    
    // Atualiza a pontuação após atraso no pagamento
    fn update_score_after_late_payment(&self, address: &ManagedAddress<Self::Api>, amount: &BigUint<Self::Api>, delay_days: u64);
    
    // Atualiza a pontuação após inadimplência
    fn update_score_after_default(&self, address: &ManagedAddress<Self::Api>, amount: &BigUint<Self::Api>);
}

pub trait ILoanController: multiversx_sc::contract_base::ContractBase {
    // Solicita um empréstimo
    fn request_loan(&self, amount: BigUint<Self::Api>, token_id: TokenIdentifier<Self::Api>, duration_days: u64) -> u64;
    
    // Paga um empréstimo
    fn repay_loan(&self, loan_id: u64);
    
    // Obtém informações de um empréstimo
    fn get_loan(&self, loan_id: u64) -> Loan<Self::Api>;
    
    // Verifica se um empréstimo está em atraso
    fn is_loan_overdue(&self, loan_id: u64) -> bool;
}

// Enum para erros comuns
#[multiversx_sc::derive::type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum CommonError {
    InsufficientFunds,
    InvalidAddress,
    Unauthorized,
    InvalidAmount,
    LoanNotFound,
    LoanAlreadyRepaid,
    LoanOverdue,
    InsufficientLiquidity,
    InvalidTokenId,
}