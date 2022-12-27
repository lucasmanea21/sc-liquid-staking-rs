elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::storage::StakeAmount;

#[elrond_wasm::module]
pub trait CallbacksModule: crate::storage::StorageModule + crate::events::EventsModule {

   

    #[callback]
    fn esdt_issue_callback(
        &self,
        caller: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_identifier) => {
                self.issue_success_event(caller, &token_identifier, &BigUint::zero());
                self.staked_egld_id().set(&token_identifier);
            },
            ManagedAsyncCallResult::Err(message) => {
                let (token_identifier, returned_tokens) =
                    self.call_value().egld_or_single_fungible_esdt();
                self.issue_failure_event(caller, &message.err_msg);

                // return issue cost to the owner
                // TODO: test that it works
                if token_identifier.is_egld() && returned_tokens > 0 {
                    self.send().direct_egld(caller, &returned_tokens);
                }
            },
        }
    }    

   
   
}