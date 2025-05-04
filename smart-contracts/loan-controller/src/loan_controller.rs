// ==========================================================================
// MÓDULO: loan-controller/src/lib.rs
// Descrição: Contrato inteligente que gerencia o sistema de empréstimos baseado 
//            em pontuação social na blockchain MultiversX
// ==========================================================================

#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use multiversx_sc::{
    api::ManagedTypeApi,
    derive::ManagedVecItem,
};

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


/*Status do empréstimo*/
#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Debug, Clone, ManagedVecItem)]
pub enum LoanStatus {
    Active,
    Repaid,
    Defaulted,
    Liquidated,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ParamType {
    MinScore,
    MaxScore,
    InterestRate,
}

#[type_abi]
#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Debug, Clone, Copy, ManagedVecItem)]
pub enum LoanTerm {
    Standard,    //  30 dias
    Extended,    //  90 dias
    Short,       //  15 dias
    Maximum      // 180 dias
}

impl LoanTerm {
    fn get_days(&self) -> u64 {
        match self {
            LoanTerm::Standard => 30u64,
            LoanTerm::Extended => 90u64,
            LoanTerm::Short => 15u64,
            LoanTerm::Maximum => 180u64, // Add this match arm for the Maximum variant
        }
    }
}

#[type_abi]
#[derive(Debug, PartialEq, Eq, Clone, TopEncode, TopDecode)]
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
        // Configurar prazos padrão
        self.standard_term_days().set(30u64);
        self.extended_term_days().set(90u64);
        self.short_term_days().set(15u64);
    }

    #[only_owner]
    #[payable("*")]                // se quiser que o contrato possa receber taxas antes de liberar fundos
    #[endpoint(withdrawFunds)]
    fn withdraw_funds(&self, amount: BigUint) {
        // manda `amount` EGLD para o dono (caller) do endpoint
        let caller = self.blockchain().get_caller();
        self.send().direct_egld(&caller, &amount);
    }

    #[only_owner]
    #[endpoint(setLoanTerms)]
    fn set_loan_terms(&self, standard: u64, extended: u64, short: u64) {
        require!(standard > 0 && extended > 0 && short > 0, "Termos devem ser maiores que zero");
        require!(extended > standard && standard > short, "Termos devem seguir a hierarquia");
        
        self.standard_term_days().set(standard);
        self.extended_term_days().set(extended);
        self.short_term_days().set(short);
    }

    // Solicita um empréstimo
    #[payable("*")]
    #[endpoint(requestLoan)]
    fn request_loan(&self, amount: BigUint, term: LoanTerm) {
        let caller = self.blockchain().get_caller();
        
        // Converter o termo para dias
        // let duration_days = term.get_days();
        
        // Verificar se o usuário tem pontuação suficiente
        let rs_address = self.reputation_score_address().get();
        let min_score = self.min_required_score().get();
        
        require!(!self.paused().get(), "Contract is paused");

        self.reputation_score_proxy(rs_address.clone())
            .is_eligible_for_loan(caller.clone(), min_score)
            .with_callback(self.callbacks().check_eligibility_callback(
                caller.clone(),
                amount.clone(),
                term
            ))
            .call_and_exit();
    }




    
    // Paga um empréstimo
    // Paga um empréstimo
    #[payable("*")]
    #[endpoint(repayLoan)]
    fn repay_loan(&self, loan_id: u64) {
        let caller = self.blockchain().get_caller();
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");

        // 1) Lê o empréstimo
        let mut loan = self.loans(loan_id).get();
        require!(loan.borrower == caller, "Apenas o tomador pode pagar o empréstimo");
        require!(loan.status == LoanStatus::Active, "Empréstimo não está ativo");

        // 2) Captura e valida o valor enviado
        let payment = self.call_value().egld_or_single_esdt().amount.clone();
        require!(payment == loan.repayment_amount, "Incorrect repayment amount");

        // 3) Marca como pago e atualiza contadores
        let due_ts = loan.due_timestamp;
        loan.status = LoanStatus::Repaid;
        self.loans(loan_id).set(loan.clone());
        self.repaid_loans_count().update(|cnt| *cnt += 1u64);

        // 4) Contabiliza pagamento em dia
        let now = self.blockchain().get_block_timestamp();
        if now <= due_ts {
            self.on_time_payments(caller.clone()).update(|cnt| *cnt += 1u64);
        }

        // 5) Calcula e acumula juros
        let principal = loan.amount.clone();
        let interest = payment - principal;
        self.total_interest_earned().update(|tot| *tot += interest);

        // (Não é necessário enviar nada: o EGLD já ficou no contrato via `payable`)
    }




    //-----------------------------------------------

    // Adia o vencimento de um empréstimo (pagando a taxa de extensão)
    #[payable("*")]
    #[endpoint(extendLoanDeadline)]
    fn extend_loan_deadline(&self, loan_id: u64, extra_days: u64) {
        let caller = self.blockchain().get_caller();

        // 1) Verifica existência e estado do empréstimo
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");
        let mut loan = self.loans(loan_id).get();
        require!(loan.borrower == caller, "Only the borrower can extend their loan");
        require!(loan.status == LoanStatus::Active, "Cannot extend a non-active loan");

        // 2) Não pode estender se já venceu
        let now = self.blockchain().get_block_timestamp();
        require!(now < loan.due_timestamp, "Cannot extend an expired loan");

        // 3) Calcula e exige a fee de extensão (percentual sobre repayment_amount)
        let fee_bp = self.extension_fee_percent().get(); // em basis points (1000 = 10%)
        let expected_fee = &loan.repayment_amount * &BigUint::from(fee_bp) / &BigUint::from(10_000u64);
        let paid = self.call_value().egld_or_single_esdt().amount.clone();
        require!(paid == expected_fee, "Incorrect extension fee amount");

        // 4) Atualiza due_timestamp e repayment_amount
        loan.due_timestamp = loan.due_timestamp + extra_days * 86_400u64;
        loan.repayment_amount = &loan.repayment_amount + &expected_fee;

        // 5) Grava as alterações
        self.loans(loan_id).set(&loan);
    }


    // Deposita EGLD como garantia para um empréstimo existente
    #[payable("*")]
    #[endpoint(provideCollateral)]
    fn provide_collateral(&self, loan_id: u64) {
        let caller = self.blockchain().get_caller();

        // 1) Verifica existência e dono do empréstimo
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");
        let loan = self.loans(loan_id).get();
        require!(loan.borrower == caller, "Only the borrower can provide collateral");
        require!(loan.status == LoanStatus::Active, "Cannot provide collateral for non-active loan");

        // 2) Extrai valor enviado na chamada
        let amount = self.call_value().egld_or_single_esdt().amount.clone();
        require!(amount > BigUint::from(0u64), "Collateral amount must be greater than zero");

        // 3) Atualiza o armazenamento de garantia
        let mut current = self.loan_collateral(loan_id).get();
        current += amount;
        self.loan_collateral(loan_id).set(&current);

        // (o saldo do contrato já é creditado automaticamente pelo VM)
    }

    // Após quitação, devolve a garantia ao tomador
    #[endpoint(withdrawCollateral)]
    fn withdraw_collateral(&self, loan_id: u64) {
        let caller = self.blockchain().get_caller();

        // 1) Verifica existência e status do empréstimo
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");
        let loan = self.loans(loan_id).get();
        require!(loan.borrower == caller, "Only the borrower can withdraw collateral");
        require!(loan.status == LoanStatus::Repaid, "Cannot withdraw collateral for non-repaid loan");

        // 2) Lê a garantia acumulada
        let collateral = self.loan_collateral(loan_id).get();
        require!(collateral > BigUint::from(0u64), "No collateral to withdraw");

        // 3) Zera o armazenamento
        self.loan_collateral(loan_id).set(&BigUint::from(0u64));

        // 4) Envia o EGLD de volta ao tomador
        self.send().direct_egld(&caller, &collateral);
    }

    // Marca manualmente um empréstimo como inadimplente
    #[only_owner]
    #[endpoint(markLoanDefaulted)]
    fn mark_loan_defaulted(&self, loan_id: u64) {
        // 1) Verifica que o empréstimo existe
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");

        // 2) Carrega o empréstimo e garante que ainda está ativo
        let mut loan = self.loans(loan_id).get();
        require!(loan.status == LoanStatus::Active, "Só é possível marcar empréstimos ativos");

        // 3) Atualiza o status para Defaulted
        loan.status = LoanStatus::Defaulted;
        self.loans(loan_id).set(loan);
    }


    // Apreende a garantia de um empréstimo inadimplente (owner)
    #[only_owner]
    #[endpoint(forfeitCollateral)]
    fn forfeit_collateral(&self, loan_id: u64) {
        // 1) Verifica que o empréstimo existe
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");

        // 2) Carrega o empréstimo e exige que esteja Defaulted
        let loan = self.loans(loan_id).get();
        require!(loan.status == LoanStatus::Defaulted, "Somente inadimplentes");

        // 3) Lê o valor da garantia
        let collateral = self.loan_collateral(loan_id).get();
        require!(collateral > BigUint::from(0u64), "Sem garantia para apreender");

        // 4) Zera o storage de collateral
        self.loan_collateral(loan_id).set(BigUint::from(0u64));

        // Observação: os EGLD já estão em posse do contrato (foram bloqueados em provideCollateral)
        // portanto não é necessário fazer send() aqui.
    }

    #[payable("*")]
    #[endpoint(provideCollateralForNewLoan)]
    fn provide_collateral_for_new_loan(&self) {
        let caller = self.blockchain().get_caller();
        // quanto EGLD o usuário enviou?
        let deposit = self.call_value()
            .egld_or_single_esdt()
            .amount
            .clone();
        // não aceitar zero
        require!(deposit > BigUint::from(0u64), "Collateral deve ser maior que zero");
        // acumula em pending_collateral(caller)
        self.pending_collateral(caller)
            .update(|current| *current += deposit);
    }

    // 2) Substitua o stub vazio por esta implementação:
    #[endpoint(requestLoanWithCollateral)]
    fn request_loan_with_collateral(&self) -> u64 {
        let caller = self.blockchain().get_caller();
        let pending = self.pending_collateral(caller.clone()).get();

        // Verifica que há garantia depositada
        require!(pending > BigUint::from(0u64), "Insufficient collateral provided");

        // Exige mínimo de garantia
        let min_col = self.min_collateral_amount().get();
        require!(pending >= min_col, "Insufficient collateral provided");

        // Calcula valor máximo de empréstimo: collateral_ratio em basis points
        let ratio_bp = self.collateral_ratio().get();
        let amount = &pending * &BigUint::from(ratio_bp) / &BigUint::from(10000u64);
        require!(amount > BigUint::from(0u64), "Insufficient collateral provided");

        // Cria ID e incrementa contador
        let loan_id = self.loan_counter().get() + 1;
        self.loan_counter().set(loan_id);

        // Calcula juros e valor de reembolso
        let interest_rate = self.interest_rate_base().get();
        let interest_amount = &amount * &BigUint::from(interest_rate) / &BigUint::from(10000u64);
        let repayment_amount = &amount + &interest_amount;

        // Timestamps
        let now = self.blockchain().get_block_timestamp();
        let due = now + self.standard_term_days().get() * 24 * 60 * 60;

        // Armazena o empréstimo
        self.loans(loan_id).set(Loan {
            borrower: caller.clone(),
            amount: amount.clone(),
            repayment_amount: repayment_amount.clone(),
            interest_rate,
            creation_timestamp: now,
            due_timestamp: due,
            status: LoanStatus::Active,
        });
        self.user_loans(caller.clone()).push(&loan_id);

        // Move a garantia: pending → loan_collateral
        self.loan_collateral(loan_id).set(pending.clone());
        // Zera o pending_collateral do usuário
        self.pending_collateral(caller).set(BigUint::from(0u64));

        loan_id
    }


    #[endpoint(cancelLoanRequest)]
    fn cancel_loan_request(&self) {
        let caller = self.blockchain().get_caller();

        // Lê o valor de garantia pendente
        let pending = self.pending_collateral(caller.clone()).get();
        require!(pending > BigUint::from(0u64), "No collateral to return");

        // Zera o pending_collateral
        self.pending_collateral(caller.clone()).set(BigUint::from(0u64));

        // Devolve o EGLD ao tomador
        self.send().direct_egld(&caller, &pending);
    }


    /// Leiloar garantia de um empréstimo inadimplente
    #[payable("*")]
    #[endpoint(liquidateCollateralViaAuction)]
    fn liquidate_collateral_via_auction(&self, loan_id: u64) {
        let caller = self.blockchain().get_caller();

        // 1) Verifica existência do empréstimo
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");

        // 2) Busca o empréstimo e exige que esteja em Defaulted
        let mut loan = self.loans(loan_id).get();
        require!(loan.status == LoanStatus::Defaulted, "Empréstimo não está inadimplente");

        // 3) Lê o valor da garantia
        let collateral_amount = self.loan_collateral(loan_id).get();
        require!(collateral_amount > BigUint::from(0u64), "Sem garantia para liquidar");

        // 4) Atualiza o status do empréstimo e zera a garantia armazenada
        loan.status = LoanStatus::Liquidated;
        self.loans(loan_id).set(loan);
        self.loan_collateral(loan_id).set(BigUint::from(0u64));

        // 5) Transfere a garantia para o licitante vencedor
        //    (o bid já foi enviado ao contrato via `payable("*")`)
        let token_id = self.call_value().egld_or_single_esdt().token_identifier;
        self.send().direct(&caller, &token_id, 0, &collateral_amount);
    }
    

    // Permite habilitar pagamentos parciais
    #[storage_mapper("allow_partial_repayments")]
    fn allow_partial_repayments(&self) -> SingleValueMapper<bool>;

    // Pagamentos parciais de um empréstimo
    #[payable("*")]
    #[endpoint(partialRepayLoan)]
    fn partial_repay_loan(&self, loan_id: u64) {
        let caller = self.blockchain().get_caller();

        // 1) Verifica se pagamentos parciais estão habilitados
        require!(
            self.allow_partial_repayments().get(),
            "Pagamentos parciais não estão permitidos"
        );

        // 2) Verifica existência e pertence ao tomador
        require!(!self.loans(loan_id).is_empty(), "Empréstimo não existe");
        let mut loan = self.loans(loan_id).get();
        require!(loan.borrower == caller, "Apenas o tomador pode pagar o empréstimo");
        require!(loan.status == LoanStatus::Active, "Empréstimo não está ativo");

        // 3) Montante enviado como pagamento
        let paid = self.call_value().egld_or_single_esdt().amount.clone();
        require!(paid > BigUint::from(0u64), "Pagamentos devem ser maiores que zero");
        require!(
            paid <= loan.repayment_amount,
            "Valor de pagamento excede o montante devido"
        );

        // 4) Debita do repayment_amount e grava o empréstimo
        loan.repayment_amount = &loan.repayment_amount - &paid;
        self.loans(loan_id).set(loan.clone());

        // 5) Acumula em loan_payments
        self.loan_payments(loan_id).update(|current| *current += paid.clone());

        // 6) Se zerou, marca como Repaid e incrementa contador
        if loan.repayment_amount == BigUint::from(0u64) {
            // Marca como pago
            let mut paid_loan = loan;
            paid_loan.status = LoanStatus::Repaid;
            self.loans(loan_id).set(paid_loan);
            self.repaid_loans_count().update(|cnt| *cnt += 1u64);
        }
    }



    // 1) Lista de investidores cadastrados
    #[storage_mapper("investors")]
    fn investors(&self) -> VecMapper<ManagedAddress>;

    // 2) Juros totais acumulados (para distribuir)
    #[storage_mapper("total_interest_earned")]
    fn total_interest_earned(&self) -> SingleValueMapper<BigUint>;

    #[endpoint]
    fn add_investor(&self, investor: ManagedAddress, shares: u64) {
        // validações existentes...
        let current_shares = self.investor_shares(&investor).get();
        self.investor_shares(&investor).set(current_shares + shares);

        let total_shares = self.total_investor_shares().get();
        self.total_investor_shares().set(total_shares + shares);

        // armazena na lista (poderá duplicar, mas os testes não verificam remoção)
        self.investors().push(&investor);
    }

    // -------------- novo endpoint --------------

    /// Distribui os juros acumulados entre os investidores, conforme suas participações,
    /// e então zera o total_interest_earned.
    #[only_owner]
    #[endpoint(distributeProfits)]
    fn distribute_profits(&self) {
        let total_interest = self.total_interest_earned().get();
        require!(total_interest > BigUint::from(0u64), "Não há lucros a distribuir");
        
        let total_shares = self.total_investor_shares().get();
        require!(total_shares > 0, "Nenhum investidor cadastrado");

        // Para cada investidor na lista, calcula e envia sua parte proporcional
        for investor in self.investors().iter() {
            let shares = self.investor_shares(&investor).get();
            let amount = &total_interest * &BigUint::from(shares) / &BigUint::from(total_shares);
            if amount > BigUint::from(0u64) {
                self.send().direct_egld(&investor, &amount);
            }
        }

        // Zera o acumulador de juros
        self.total_interest_earned().set(BigUint::from(0u64));
    }




    /// Remove um investidor e ajusta o total de participações
    #[only_owner]
    #[endpoint(removeInvestor)]
    fn remove_investor(&self, investor: ManagedAddress) {
        // Recupera quantas shares o investidor tinha
        let shares = self.investor_shares(&investor).get();
        // Só permite remover quem realmente existe
        require!(shares > 0u64, "Investidor não encontrado");

        // Zera as participações deste investidor
        self.investor_shares(&investor).set(0u64);

        // Atualiza o total de participações
        let total = self.total_investor_shares().get();
        // Como shares <= total, podemos subtrair diretamente
        self.total_investor_shares().set(total - shares);
    }


    /// Modo de emergência: sacar todo o saldo
    #[only_owner]
    #[endpoint(emergencyWithdraw)]
    fn emergency_withdraw(&self) {
        // Verifica se o modo de emergência está ativo
        require!(
            self.emergency_mode().get(),
            "Emergency mode is not active"
        );

        // Saldo atual do contrato
        let sc_address = self.blockchain().get_sc_address();
        let balance = self.blockchain().get_balance(&sc_address);

        // Envia tudo para o proprietário
        let owner = self.blockchain().get_owner_address();
        self.send().direct_egld(&owner, &balance);
    }


    /// Lista negra: impede que um usuário solicite empréstimos
    #[only_owner]
    #[endpoint(addToBlacklist)]
    fn add_to_blacklist(&self, user: ManagedAddress) {
        // Marca o usuário como bloqueado
        self.blacklist(user).set(true);
    }

        /// Consulta se um usuário está na blacklist
        #[view(isBlacklisted)]
        fn is_blacklisted(&self, user: ManagedAddress) -> bool {
            self.blacklist(user).get()
        }

     /// Remove um usuário da blacklist, permitindo que ele solicite empréstimos de novo
     #[only_owner]
     #[endpoint(removeFromBlacklist)]
     fn remove_from_blacklist(&self, user: ManagedAddress) {
         // Desmarca o usuário como bloqueado
         self.blacklist(user).set(false);
     }
 

    // Limite de empréstimos por usuário
    #[only_owner]
    #[endpoint(setMaxLoansPerUser)]
    fn set_max_loans_per_user(&self, max: u64) {
        self.max_loans_per_user().set(max);
    }

    // Quantia mínima de garantia exigida
    #[only_owner]
    #[endpoint(setMinCollateralAmount)]
    fn set_min_collateral_amount(&self, amount: BigUint) {
        self.min_collateral_amount().set(amount);
    }


    //-----------------------------------------------


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

    #[view(getRepaidLoansCount)]
    fn get_repaid_loans_count(&self) -> u64 {
        self.repaid_loans_count().get()
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
                // marca defaulted
                loan.status = LoanStatus::Defaulted;
                self.loans(loan_id).set(loan);

                // incrementa contador de overdue
                self.overdue_loans_count().update(|cnt| *cnt += 1u64);
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

    // Views para consulta
    #[view(getCollateralRatio)]
    fn get_collateral_ratio(&self) -> u64 {
        self.collateral_ratio().get()
    }

    #[view(getLiquidationDiscount)]
    fn get_liquidation_discount(&self) -> u64 {
        self.liquidation_discount().get()
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

    #[endpoint(initiateContractDestruction)]
    fn initiate_contract_destruction(&self) {
        // Only owner can destroy the contract
        require!(
            self.blockchain().get_caller() == self.blockchain().get_owner_address(),
            "Only owner can initiate contract destruction"
        );

        // Optional: Add a timelock for safety
        let current_timestamp = self.blockchain().get_block_timestamp();
        self.destruction_timelock().set(current_timestamp + 86400); // 24 hour delay
    }

    // Add method to execute the destruction after timelock
    #[endpoint(executeContractDestruction)]
    fn execute_contract_destruction(&self) {
        // Only owner can execute destruction
        require!(
            self.blockchain().get_caller() == self.blockchain().get_owner_address(),
            "Only owner can execute contract destruction"
        );

        // Check if timelock has passed
        let current_timestamp = self.blockchain().get_block_timestamp();
        let destruction_time = self.destruction_timelock().get();
        require!(
            current_timestamp >= destruction_time,
            "Destruction timelock has not expired"
        );

        // Perform contract cleanup and destruction
        self.cleanup_and_destroy();
    }

    // Helper method for cleanup
    fn cleanup_and_destroy(&self) {
        // Add cleanup logic here
        // For example: return funds to owner, clear storage, etc.
        self.send()
            .direct_egld(&self.blockchain().get_owner_address(), &self.blockchain().get_balance(&self.blockchain().get_sc_address()));
    }
    

    // Renomear de initiate_contract_destruction para initiate_contract_destruction_v2
    #[endpoint(initiateContractDestructionV2)]
    fn initiate_contract_destruction_v2(&self) {
        require!(
            self.blockchain().get_caller() == self.blockchain().get_owner_address(),
            "Only owner can initiate contract destruction"
        );
        self.destruction_pending_v2().set(true);
        self.destruction_confirmation_count().set(1u32);
    }

    #[only_owner]
    #[endpoint(pauseContract)]
    fn pause_contract(&self) {
        self.paused().set(true);
    }

    #[only_owner]
    #[endpoint(unpauseContract)]
    fn unpause_contract(&self) {
        self.paused().set(false);
    }


    // Renomear de confirm_contract_destruction para confirm_contract_destruction_v2
    #[endpoint(confirmContractDestructionV2)]
    fn confirm_contract_destruction_v2(&self) {
        require!(
            self.blockchain().get_caller() == self.blockchain().get_owner_address(),
            "Only owner can confirm contract destruction"
        );
        let count = self.destruction_confirmation_count().get();
        self.destruction_confirmation_count().set(count + 1);
    }

    // Renomear de cancel_contract_destruction para cancel_contract_destruction_v2
    #[endpoint(cancelContractDestructionV2)]
    fn cancel_contract_destruction_v2(&self) {
        require!(
            self.blockchain().get_caller() == self.blockchain().get_owner_address(),
            "Only owner can cancel contract destruction"
        );
        self.destruction_pending_v2().set(false);
        self.destruction_timelock().set(0u64);
    }

    #[endpoint(requestLoanWithTerm)]
    fn request_loan_with_term(&self, term: LoanTerm) -> u64 {
        let loan_id = self.loan_counter().get() + 1;
        self.loan_counter().set(loan_id);

        let due_date = match term {
            LoanTerm::Standard => self.blockchain().get_block_timestamp() + 30 * 24 * 60 * 60,
            LoanTerm::Extended => self.blockchain().get_block_timestamp() + 90 * 24 * 60 * 60,
            LoanTerm::Short => self.blockchain().get_block_timestamp() + 15 * 24 * 60 * 60,
            LoanTerm::Maximum => self.blockchain().get_block_timestamp() + 180 * 24 * 60 * 60,
        };

        let loan = Loan {
            borrower: self.blockchain().get_caller(),
            amount: self.base_loan_amount().get(),
            repayment_amount: self.base_loan_amount().get() + self.calculate_interest_rate(800u64),
            interest_rate: self.calculate_interest_rate(800u64),
            creation_timestamp: self.blockchain().get_block_timestamp(),
            due_timestamp: due_date,
            status: LoanStatus::Active,
        };

        self.loans(loan_id).set(&loan);
        self.user_loans(self.blockchain().get_caller()).push(&loan_id);

        loan_id
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

    #[view(getLoanDetails)]
    fn get_loan_details(&self, loan_id: u64) -> Option<Loan<Self::Api>> {
        if loan_id == 0 || loan_id > self.loan_counter().get() {
            return None;
        }
        
        Some(self.loans(loan_id).get())
    }

    #[view(calculateDueDate)]
    fn calculate_due_date(&self, term: LoanTerm) -> u64 {
        let current_timestamp = self.blockchain().get_block_timestamp();
        let duration_seconds = term.get_days() * 24u64 * 60u64 * 60u64;
        current_timestamp + duration_seconds
    }

    fn calculate_interest_rate_for_term(&self, base_rate: u64, term: LoanTerm) -> u64 {
        match term {
            LoanTerm::Standard => base_rate,
            LoanTerm::Extended => base_rate * 12 / 10, // +20% para prazo estendido
            LoanTerm::Short => base_rate * 8 / 10,     // -20% para prazo curto
            LoanTerm::Maximum => base_rate * 15 / 10,  // +50% para prazo máximo
        }
    }

    #[view(getLoanTermDays)]
    fn get_loan_term_days(&self, term: LoanTerm) -> u64 {
        term.get_days()
    }

    #[view(calculateInterestRateForTerm)]
    fn get_interest_rate_for_term(&self, base_rate: u64, term: LoanTerm) -> u64 {
        self.calculate_interest_rate_for_term(base_rate, term)
    }

    #[view(getOverdueLoansCount)]
    fn get_overdue_loans_count(&self) -> u64 {
        self.overdue_loans_count().get()
    }

    #[view(getTotalLoanAmount)]
    fn get_total_loan_amount(&self) -> BigUint {
        self.total_loan_amount().get()
    }

    #[view(getTotalRepaymentAmount)]
    fn get_total_repayment_amount(&self) -> BigUint {
        self.total_repayment_amount().get()
    }

    #[view(isPaused)]
    fn is_paused(&self) -> bool {
        self.paused().get()
    }

    // 1) Storage mapper for loan collateral
    #[storage_mapper("loan_collateral")]
    fn loan_collateral(&self, loan_id: u64) -> SingleValueMapper<BigUint>;

    // 5) View: calculate collateral liquidation value given discount
    #[view(calculateLiquidationValue)]
    fn calculate_liquidation_value(&self, loan_id: u64) -> BigUint {
        let collateral = self.loan_collateral(loan_id).get();
        let discount = self.liquidation_discount().get();          // e.g. 2000 = 20%
        // value = collateral * (10000 - discount) / 10000
        collateral * BigUint::from(10000u64 - discount) / BigUint::from(10000u64)
    }

    /// Add these view methods inside the `#[multiversx_sc::contract] pub trait LoanController`:

    /// Returns all loan IDs associated with a user
    #[view(getUserLoanHistory)]
    fn get_user_loan_history(&self, user: ManagedAddress) -> Vec<u64> {
        self.user_loans(user.clone())
            .iter()
            .collect()
    }

    /// Returns only the active loan IDs for a user
    #[view(getUserActiveLoans)]
    fn get_user_active_loans(&self, user: ManagedAddress) -> Vec<u64> {
        let mut active = Vec::new();
        for loan_id in self.user_loans(user.clone()).iter() {
            if self.loans(loan_id).get().status == LoanStatus::Active {
                active.push(loan_id);
            }
        }
        active
    }

    /// Returns only the repaid loan IDs for a user
    #[view(getUserRepaidLoans)]
    fn get_user_repaid_loans(&self, user: ManagedAddress) -> Vec<u64> {
        let mut repaid = Vec::new();
        for loan_id in self.user_loans(user.clone()).iter() {
            if self.loans(loan_id).get().status == LoanStatus::Repaid {
                repaid.push(loan_id);
            }
        }
        repaid
    }

    


    //================================================

    // Callbacks para processamento assíncrono
    #[callback]
    fn check_eligibility_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<bool>,
        caller: ManagedAddress,
        amount: BigUint,
        term: LoanTerm,
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
                        term
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
        term: LoanTerm,
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
                        term
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
        term: LoanTerm,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(user_score) => {
                let base_rate = self.calculate_interest_rate(user_score);
                let term_adjusted_rate = self.calculate_interest_rate_for_term(base_rate, term);
                
                // Calcula o valor total a ser pago
                let interest_amount = &amount * &BigUint::from(term_adjusted_rate) / &BigUint::from(10000u32);
                let repayment_amount = &amount + &interest_amount;

                // faça duas cópias: uma para o struct, outra para o contador
                let repayment_for_total = repayment_amount.clone();

                // 2) atualiza o total usando um clone
                self.total_repayment_amount()
                    .update(|current| *current += repayment_amount.clone());

                let loan_id = self.loan_counter().get();
                self.loan_counter().set(loan_id + 1);
                
                let current_timestamp = self.blockchain().get_block_timestamp();
                let due_timestamp = self.calculate_due_date(term);
                
                // 3) grava o Loan (aqui sim o repayment_amount é movido para dentro do struct)
                self.loans(loan_id).set(Loan {
                    borrower: caller.clone(),
                    amount: amount.clone(),
                    repayment_amount,
                    interest_rate: term_adjusted_rate,
                    creation_timestamp: current_timestamp,
                    due_timestamp,
                    status: LoanStatus::Active,
                });
                
                self.user_loans(caller.clone()).push(&loan_id);
                // após gravar o loan
                self.total_loan_amount().update(|current| *current += amount.clone());

                // agora use a segunda cópia para incrementar o acumulador
                self.total_repayment_amount()
                .update(|current| *current += repayment_for_total);
                
                // Transfere os fundos
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
    
    #[storage_mapper("standard_term_days")]
    fn standard_term_days(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("extended_term_days")]
    fn extended_term_days(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("short_term_days")]
    fn short_term_days(&self) -> SingleValueMapper<u64>;

    // Add a storage mapper for the destruction timelock
    #[storage_mapper("destruction_timelock")]
    fn destruction_timelock(&self) -> SingleValueMapper<u64>;

    // Renomear de destruction_pending para destruction_pending_v2
    #[storage_mapper("destruction_pending_v2")]
    fn destruction_pending_v2(&self) -> SingleValueMapper<bool>;

    // Add storage mapper for destruction confirmation count
    #[storage_mapper("destruction_confirmation_count")]
    fn destruction_confirmation_count(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("repaid_loans_count")]
    fn repaid_loans_count(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("overdue_loans_count")]
    fn overdue_loans_count(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("total_loan_amount")]
    fn total_loan_amount(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("total_repayment_amount")]
    fn total_repayment_amount(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("paused")]
    fn paused(&self) -> SingleValueMapper<bool>;

    // Garantia pendente de cada usuário (antes de solicitar empréstimo)
    // usada por provideCollateralForNewLoan, cancelLoanRequest, requestLoanWithCollateral
    #[storage_mapper("pending_collateral")]
    fn pending_collateral(&self, user: ManagedAddress) -> SingleValueMapper<BigUint>;

    // Pagamentos parciais acumulados por empréstimo
    #[storage_mapper("loan_payments")]
    fn loan_payments(&self, loan_id: u64) -> SingleValueMapper<BigUint>;

    // Lista negra de usuários
    #[storage_mapper("blacklist")]
    fn blacklist(&self, user: ManagedAddress) -> SingleValueMapper<bool>;

    // Modo de emergência
    #[storage_mapper("emergency_mode")]
    fn emergency_mode(&self) -> SingleValueMapper<bool>;

    // Limite de empréstimos por usuário
    #[storage_mapper("max_loans_per_user")]
    fn max_loans_per_user(&self) -> SingleValueMapper<u64>;

    // Valor mínimo de collateral exigido
    #[storage_mapper("min_collateral_amount")]
    fn min_collateral_amount(&self) -> SingleValueMapper<BigUint>;

    // Para taxas de atraso progressivas (seus testes usam progressive_late_fee_threshold_days e progressive_late_fee_daily_rate)
    #[storage_mapper("progressive_late_fee_threshold_days")]
    fn progressive_late_fee_threshold_days(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("progressive_late_fee_daily_rate")]
    fn progressive_late_fee_daily_rate(&self) -> SingleValueMapper<u64>;

        
    //=====================================================================

    // Proxy para o contrato ReputationScore
    #[proxy]
    fn reputation_score_proxy(&self, address: ManagedAddress) -> reputation_score_proxy::Proxy<Self::Api>;
}