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

    #[callback]
    fn get_stake_callback(
        &self,
        current_epoch: u64,
        #[call_result] result: ManagedAsyncCallResult<BigUint>
    ) {
        match result {
            ManagedAsyncCallResult::Ok(value) => {
                let mapping_index = self.mapping_index().get();

                self.mapping_index().set(&mapping_index + &1);
                
                if &mapping_index >= &(self.validators().len()) {
                    self.mapping_index().set(1 as usize);
                }

                let stake_amount = StakeAmount {
                    epoch: current_epoch,
                    amount: value.clone()
                };

                self.stake_amounts().push(&stake_amount);

                self.callback_result().set(&value);
            },
            ManagedAsyncCallResult::Err(err) => {
                let mapping_index = self.mapping_index().get();

                self.mapping_index().set(&mapping_index + &1);
                
                if &mapping_index >= &(self.validators().len()) {
                    self.mapping_index().set(1 as usize);
                }
            },
        }
    }
   
}