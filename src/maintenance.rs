use elrond_wasm::api::HandleConstraints;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait MaintenanceModule: 
    crate::storage::StorageModule 
    {

        #[endpoint(distributeProtocolRevenue)]
        fn distribute_protocol_revenue(&self) {
            // the endpoint for distributing the protocol fees
            // protocol rewards are in stEGLD.
            // this endpoint mints & sends the revenue to the contract owner

            let exchange_rate = self.exchange_rate().get();
            let st_egld_id = self.staked_egld_id().get();
            let protocol_revenue = self.protocol_revenue().get();
            let sc_owner = self.blockchain().get_owner_address();

            let amount_to_send = &protocol_revenue * &exchange_rate;

            self.send().esdt_local_mint(&st_egld_id, 0, &amount_to_send);
            self.send().direct_esdt(&sc_owner, &st_egld_id, 0, &amount_to_send);

            self.protocol_revenue().set(BigUint::from(0u64));
        }

       
        #[endpoint(updateExchangeRate)]
        fn update_exchange_rate(&self) {
            // todo: require all other operations have finished: get rewards, redelegation, delegation , get total staked
            let current_epoch = self.blockchain().get_block_epoch(); 
            let rewards_value = self.rewards_amounts().get(&current_epoch);
            let stake_value = self.stake_amounts().get(&current_epoch);
            let total_token_supply = self.total_token_supply().get();
    
    
            require!(match rewards_value {
                Some(n) => true, 
                None => false
            }, "didn't get rewards this epoch");
    
            require!(match stake_value.clone() {
                Some(n) => true,
                None => false
            }, "didn't get total stake this epoch");
    
        
    
            self.exchange_rate().set(&total_token_supply / &stake_value.unwrap_or(BigUint::from(1u64)));
        }

        #[inline]
        fn calculate_delegation(&self) {
            // get the array of validators with rewards

            // run algorithm to even out stake without removing any 


        }        
    
    }