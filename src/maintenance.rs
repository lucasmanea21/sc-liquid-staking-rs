use elrond_wasm::api::HandleConstraints;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait MaintenanceModule: 
    crate::storage::StorageModule 
    {
        
        // the endpoint for distributing the protocol fees
        // protocol rewards are in stEGLD.
        // this endpoint mints & sends the revenue to the contract owner
        #[endpoint(distributeProtocolRevenue)]
        fn distribute_protocol_revenue(&self) {
            let st_egld_id = self.staked_egld_id().get();
            let sc_owner = self.blockchain().get_owner_address();
            let protocol_revenue = self.protocol_revenue().get();
            
            let exchange_rate = self.exchange_rate().get();
            let exchange_rate_multiplier = self.exchange_rate_multiplier().get();

            let amount_to_send = &protocol_revenue * &exchange_rate / exchange_rate_multiplier;
            let current_total_supply = self.total_token_supply().get();

            self.send().esdt_local_mint(&st_egld_id, 0, &amount_to_send);
            self.send().direct_esdt(&sc_owner, &st_egld_id, 0, &amount_to_send);

            self.total_token_supply()
                .set(&current_total_supply + &amount_to_send);

            self.protocol_revenue().set(BigUint::from(0u64));
        }

       
        #[endpoint(updateExchangeRate)]
        fn update_exchange_rate(&self) {
            let current_epoch = self.blockchain().get_block_epoch(); 
            let already_updated = self.exchange_rate_update_finished().contains(&current_epoch);

            require!(!already_updated, "Exchange rate already updated for this epoch");

            let stake_info_finished = self.stake_info_finished().contains(&current_epoch);
            let rewards_info_finished = self.rewards_info_finished().contains(&current_epoch);
            let withdraw_finished = self.withdraw_finished().contains(&current_epoch);
            
            require!(
                stake_info_finished && rewards_info_finished && withdraw_finished,
                "All operations must be finished before updating exchange rate"
            );

            let stake_value = self.stake_amounts().get(&current_epoch);
            let total_token_supply = self.total_token_supply().get();
            let exchange_rate_multiplier = self.exchange_rate_multiplier().get();
    
            self.exchange_rate().set((total_token_supply * exchange_rate_multiplier) / 
               (match stake_value {
                    Some(n) => if n > 0 { n } else { BigUint::from(1u64) },
                    None => BigUint::from(1u64),
                })
            );

            self.exchange_rate_update_finished().insert(current_epoch);
        }      
    }