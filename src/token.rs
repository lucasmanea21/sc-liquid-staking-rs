elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const EGLD_NUM_DECIMALS: usize = 18;

#[elrond_wasm::module]
pub trait TokenModule: 
    crate::events::EventsModule 
    + crate::storage::StorageModule
 {

    // Create token

      #[only_owner]
      #[payable("EGLD")]
      #[endpoint(issueToken)]
      fn issue_staked_egld(&self, token_display_name: ManagedBuffer, token_ticker: ManagedBuffer) {
        require!(
            self.staked_egld_id().is_empty(),
            "token was already issued"
        );

        let issue_cost = self.call_value().egld_value();
        let caller = self.blockchain().get_caller();
        let initial_supply = BigUint::zero();

        self.issue_started_event(&caller, &token_ticker, &initial_supply);

        self.send()
            .esdt_system_sc_proxy()
            .issue_fungible(
                issue_cost,
                &token_display_name,
                &token_ticker,
                &initial_supply,
                FungibleTokenProperties {
                    num_decimals: EGLD_NUM_DECIMALS,
                    can_freeze: false,
                    can_wipe: false,
                    can_pause: false,
                    can_mint: true,
                    can_burn: false,
                    can_change_owner: true,
                    can_upgrade: true,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().esdt_issue_callback(&caller))
            .call_and_exit()
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        require!(
            !self.staked_egld_id().is_empty(),
            "Must issue token first"
        );

        let roles = [EsdtLocalRole::Mint, EsdtLocalRole::Burn];
        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.staked_egld_id().get(),
                roles[..].iter().cloned(),
            )
            .async_call()
            .call_and_exit()
    }

    // Callback

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
