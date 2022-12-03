elrond_wasm::derive_imports!();
elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait EventsModule {
    #[event("issue-started")]
    fn issue_started_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] token_ticker: &ManagedBuffer,
        initial_supply: &BigUint,
    );

    #[event("issue-success")]
    fn issue_success_event(
        &self,
        #[indexed] caller: &ManagedAddress,
        #[indexed] token_identifier: &TokenIdentifier,
        initial_supply: &BigUint,
    );

    #[event("issue-failure")]
    fn issue_failure_event(&self, #[indexed] caller: &ManagedAddress, message: &ManagedBuffer);
}