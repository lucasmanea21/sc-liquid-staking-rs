#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod callbacks;
mod delegate;
mod events;
mod helpers;
mod maintenance;
mod storage;
mod tokens;

use crate::callbacks::CallbacksModule;
use crate::heap::Vec;
use crate::storage::StakeAmount;
use crate::tokens::TokenAttributes;

#[elrond_wasm::contract]
pub trait StakeContract:
    elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + storage::StorageModule
    + callbacks::CallbacksModule
    + events::EventsModule
    + tokens::TokenModule
    + helpers::HelpersModule
    + maintenance::MaintenanceModule
{
    #[proxy]
    fn delegate_contract(&self, sc_address: ManagedAddress) -> delegate::Proxy<Self::Api>;

    #[init]
    fn init(&self) {
        if self.delta_stake().is_empty() {
            self.delta_stake().set(BigInt::from(0))
        };
        if self.exchange_rate().is_empty() {
            self.exchange_rate().set(BigUint::from(1u64))
        };
        if self.mapping_index().is_empty() {
            self.mapping_index().set(1 as usize)
        };
        if self.withdraw_mapping_index().is_empty() {
            self.withdraw_mapping_index().set(1 as usize)
        };
        if self.rewards_mapping_index().is_empty() {
            self.rewards_mapping_index().set(1 as usize)
        };
        if self.redelegate_mapping_index().is_empty() {
            self.redelegate_mapping_index().set(1 as usize)
        };
        if self.exchange_rate_multiplier().is_empty() {
            self.exchange_rate_multiplier().set(BigUint::from(10u64.pow(18)))
        };
    }


    // Receives EGLD, mints and sends stEGLD
    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self) {
        let value = self.call_value().egld_value();

        require!(&value > &0, "Stake value must be bigger than 0");

        let caller = self.blockchain().get_caller();
        let exchange_rate = self.exchange_rate().get();
        let exchange_rate_multiplier = self.exchange_rate_multiplier().get();

        let st_egld_id = self.staked_egld_id().get();
        let st_egld_amount = exchange_rate * &value / exchange_rate_multiplier;

        let current_total_supply = self.total_token_supply().get();
        let current_delta_stake = self.delta_stake().get();

        self.send().esdt_local_mint(&st_egld_id, 0, &st_egld_amount);
        self.send()
            .direct_esdt(&caller, &st_egld_id, 0, &st_egld_amount);

        self.total_token_supply()
            .set(&current_total_supply + &st_egld_amount);
        self.delta_stake().set(&current_delta_stake + &value);
    }

    // Receives stEGLD

    #[payable("*")]
    #[endpoint]
    fn unstake(&self) {
        let (token, _, payment) = self.call_value().single_esdt().into_tuple();
        let st_egld_id = self.staked_egld_id().get();

        require!(&token == &st_egld_id, "Invalid token sent");
        require!(&payment > &0, "Cannot receive 0 amount");

        let caller = self.blockchain().get_caller();
        let current_total_supply = self.total_token_supply().get();
        let current_delta_stake = self.delta_stake().get();

        self.send().esdt_local_burn(&st_egld_id, 0, &payment);
        self.total_token_supply()
            .set(&current_total_supply - &payment);
        self.delta_stake().set(&current_delta_stake - &payment);

        // todo: keep track of unstaking, so user can withdraw later

        /*
            mint uEGLD based on exchange rate (mint the EGLD equivalent)
        */
        let exchange_rate = self.exchange_rate().get();
        let exchange_rate_multiplier = self.exchange_rate_multiplier().get();

        let attr = &TokenAttributes {
            epoch: self.blockchain().get_block_epoch(),
        };

        self.create_and_send_assets(&payment / &exchange_rate * exchange_rate_multiplier, &caller, &attr);
    }

    #[payable("*")]
    #[endpoint]
    fn claim(&self) {
        let (token, nonce, payment) = self.call_value().single_esdt().into_tuple();
        let caller = self.blockchain().get_caller();
        let sc_address = &self.blockchain().get_sc_address();
        let current_epoch = self.blockchain().get_block_epoch();
        let undelegated_token = self.undelegated_token().get_token_id();

        require!(&token == &undelegated_token, "Invalid token sent");
        require!(&payment > &0, "Cannot receive 0 amount");

        let token_info = self
            .blockchain()
            .get_esdt_token_data(&sc_address, &token, nonce);

        let attr = self
            .serializer()
            .top_decode_from_managed_buffer::<TokenAttributes>(&token_info.attributes);

        let service_fee = self.service_fee().get();

        // check if the epochs required to claim passed

        if (&attr.epoch - current_epoch) >= 1 {
            self.send().direct_egld(&caller, &payment)
        }
    }

    // Admin operations
    // todo: move all to admin.rs file

    // solution for getting how much the SC has staked to validators inside the SC
    // could be replaced with off-chain daemon

    #[only_owner]
    #[endpoint(getStakeAdmin)]
    fn get_stake_admin(&self) {
        let current_epoch = self.blockchain().get_block_epoch();
        let mapping_index = self.mapping_index().get();
        let sc_address = self.blockchain().get_sc_address();
        let epoch_exists = self.stake_info_started().contains(&current_epoch);

        let rewards_fetched = self.rewards_info_finished().contains(&current_epoch);
        let redelegate_finished = self.redelegate_finished().contains(&current_epoch);


        require!(rewards_fetched, "must fetch rewards first");
        require!(redelegate_finished, "must redelegate rewards first");

        require!(
            !(epoch_exists && mapping_index == 1),
            "already got stake amount this epoch"
        );

        if !epoch_exists {
            self.mapping_index().set(1 as usize);
            self.stake_info_started().insert(current_epoch.clone());
        };

        let wanted_address = self.validators().get(mapping_index);

        self.increment_index();

        self.delegate_contract(wanted_address.clone())
            .getUserActiveStake(sc_address)
            .async_call()
            .with_callback(StakeContract::callbacks(self).get_stake_callback(current_epoch,wanted_address.clone()))
            .call_and_exit();
    }

    // todo: move this to callbacks.rs
    #[callback]
    fn get_stake_callback(
        &self,
        current_epoch: u64,
        validator: ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        let old_value = self.stake_amounts().get(&current_epoch);

        match result {
            ManagedAsyncCallResult::Ok(value) => {
                self.stake_amounts().insert(
                    current_epoch,
                    match old_value {
                        Some(n) => n,
                        None => BigUint::from(0u64),
                    } + value.clone(),
                );

                self.validator_stake_amount().insert(
                    validator,
                    value.clone()
                );
            }
            ManagedAsyncCallResult::Err(err) => {
                self.stake_amounts().insert(
                    current_epoch,
                    match old_value {
                        Some(n) => n,
                        None => BigUint::from(0u64),
                    } + BigUint::from(0u64),
                );
            }
        }
    }

    #[only_owner]
    #[endpoint(getRewardsAdmin)]
    fn get_rewards_admin(&self) {
        // solution for getting how much the SC has staked to validators inside the SC
        // could be replaced with off-chain daemon

        let current_epoch = self.blockchain().get_block_epoch();
        let mapping_index = self.rewards_mapping_index().get();
        let sc_address = self.blockchain().get_sc_address();

        // let current_epoch_amounts: _= self.stake_amounts().iter().filter(|amount| amount.epoch == current_epoch).collect::<ManagedVec<StakeAmount<Self::Api>>>();
        let epoch_exists = self.rewards_info_started().contains(&current_epoch);

        require!(
            !(epoch_exists && mapping_index == 1),
            "already got rewards amount this epoch"
        );

        if !epoch_exists {
            self.rewards_mapping_index().set(1 as usize);
            self.rewards_info_started().insert(current_epoch.clone());
        };

        let wanted_address = self.validators().get(mapping_index);

        self.increment_index_rewards();

        self.delegate_contract(wanted_address)
            .getClaimableRewards(sc_address)
            .async_call()
            .with_callback(StakeContract::callbacks(self).get_rewards_callback(current_epoch))
            .call_and_exit();
    }

    #[callback]
    fn get_rewards_callback(
        &self,
        current_epoch: u64,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        let old_value = self.rewards_amounts().get(&current_epoch);
        let mapping_index = self.mapping_index().get();

        match result {
            ManagedAsyncCallResult::Ok(value) => {

                self.rewards_amounts().insert(
                    current_epoch,
                    match old_value {
                        Some(n) => n,
                        None => BigUint::from(0u64),
                    } + value.clone(),
                );

                self.update_protocol_revenue(&current_epoch);
            }
            ManagedAsyncCallResult::Err(err) => {
                self.rewards_amounts().insert(
                    current_epoch,
                    match old_value {
                        Some(n) => n,
                        None => BigUint::from(0u64),
                    } + BigUint::from(0u64)
                );

                self.update_protocol_revenue(&current_epoch);
            }
        }
    }

    // #[endpoint(dailyDelegation)]
    // fn daily_delegation(&self) {
    //     // take mapping of validator and stake
        

    //     // find the one with the least amount
    //     // find the one with the biggest amount

    //     let delta_stake = self.delta_stake().get();

    //     if delta_stake > 0 {
    //         // stake to the one with least
    //     } else {
    //         // unstake all from the one with the most
    //     }

    //     // find the one with the least stake

    //     // try to stake all to it
    // }

    #[callback]
    fn delegation_callback(
        &self,
        current_epoch: u64,
        address:ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(value) => {
                // perfect scenario, daily delegation is finished
                self.delta_stake().clear();
            }
            ManagedAsyncCallResult::Err(err) => {
                self.validator_stake_amount().remove(&address);
            }
        }
    }

    #[only_owner]
    #[endpoint(delegateAdmin)]
    fn delegate_admin(&self, amount: BigUint) {
        // solution for on-chain delegation

        let mapping_index = self.mapping_index().get();
        let wanted_address = self.validators().get(mapping_index);

        self.mapping_index().set(&mapping_index + 1);

        if &mapping_index >= &(self.validators().len() - 1) {
            self.mapping_index().set(1 as usize);
        }

        self.delegate_contract(wanted_address)
            .delegate(EgldOrEsdtTokenIdentifier::egld(), amount)
            .async_call()
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint(delegate_direct)]
    fn delegate_direct(&self, address: ManagedAddress, amount: BigUint) {
        // directly delegates to the contract specified.
        let current_epoch = self.blockchain().get_block_epoch();

        self.delegate_contract(address.clone() )
            .delegate(EgldOrEsdtTokenIdentifier::egld(), amount)
            .async_call().with_callback(StakeContract::callbacks(self).delegation_callback(current_epoch, address))
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint]
    fn undelegate_direct(&self, address: ManagedAddress, amount: &BigUint) {
        // directly undelegates from the contract specified.
        let current_epoch = self.blockchain().get_block_epoch();

        self.delegate_contract(address.clone())
            .unDelegate(amount)
            .async_call().with_callback(StakeContract::callbacks(self).delegation_callback(current_epoch, address))
            .call_and_exit();
    }


    // redelegates rewards at each validator.
    // should be done after computing rewards

    #[only_owner]
    #[endpoint(redelegateAdmin)]
    fn redelegateAdmin(&self) {
        let mapping_index = self.redelegate_mapping_index().get();
        let current_epoch = self.blockchain().get_block_epoch();
        let validators = self.validators();
        let epoch_exists = self.redelegate_started().contains(&current_epoch);
        
        let is_rewards_info_finished = self.rewards_info_finished().contains(&current_epoch);
        let wanted_address = self.validators().get(mapping_index);

        require!(
            is_rewards_info_finished,
            "must get rewards first"
        );
        
        require!(
            !(epoch_exists && mapping_index == 1),
            "already redelegated"
        );

        if !epoch_exists {
            self.mapping_index().set(1 as usize);
            self.redelegate_started().insert(current_epoch.clone());
        };

        self.increment_index_redelegate();

        self.delegate_contract(wanted_address)
            .reDelegateRewards()
            .async_call()
            .with_callback(StakeContract::callbacks(self).redelegate_callback(current_epoch, mapping_index))
            .call_and_exit();
    }

    #[callback]
    fn redelegate_callback(
        &self,
        current_epoch: u64,
        mapping_index: usize,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        // if mapping index is 1, it means it was the last one
        if &mapping_index == &(1 as usize) {
            self.redelegate_finished().insert(current_epoch);
        }
    }

    #[only_owner]
    #[endpoint(withdrawAdmin)]
    fn withdraw_admin(&self) {
        // solution for getting how much the SC has staked to validators inside the SC
        // could be replaced with off-chain daemon

        let current_epoch = self.blockchain().get_block_epoch();
        let mapping_index = self.withdraw_mapping_index().get();
        let sc_address = self.blockchain().get_sc_address();
    
        let epoch_exists = self.withdraw_started().contains(&current_epoch);

        require!(
            !(epoch_exists && mapping_index == 1),
            "already got rewards amount this epoch"
        );

        if !epoch_exists {
            self.withdraw_mapping_index().set(1 as usize);
            self.withdraw_started().insert(current_epoch);
        };

        let wanted_address = self.validators().get(mapping_index);

        self.increment_index_withdraw();

        self.delegate_contract(wanted_address)
            .withdraw()
            .async_call()
            .with_callback(StakeContract::callbacks(self).withdraw_callback(current_epoch, mapping_index))
            .call_and_exit();
    }

    #[callback]
    fn withdraw_callback(
        &self,
        current_epoch: u64,
        mapping_index: usize,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        // if mapping index is 1, it means it was the last one
        if &mapping_index == &(1 as usize) {
            self.withdraw_finished().insert(current_epoch);
        }
    }

    #[only_owner]
    #[endpoint]
    fn push_validators(&self, address: &ManagedAddress) {
        self.validators().push(address);
    }

    #[endpoint(dailyDelegation)]
    fn daily_delegation(&self) {
        // take mapping of validator and stake
        
        let validators = self.validator_stake_amount();
        let mut smallest = BigUint::from(0u64);
        let mut biggest = BigUint::from(0u64);
        let delta_stake = self.delta_stake().get();


        // set smallest as 1st entry
        for validator in validators.values() {
            smallest = validator;
            break;
        }

        // set biggest as 1st entry
        for validator in validators.values() {
            biggest = validator;
            break;
        }

        // find the one with the least amount [smallest]
        for validator in validators.values() {
            if validator < smallest{
                smallest = validator; //find smallest
            }
        }

        //find the one with the biggest amount [biggest]
        for validator in validators.values() {
            if validator > biggest{
                biggest = validator; //find biggest
            } 
        }

        if delta_stake > 0 {
            // stake to the one with least
            for validator in validators.iter() {
                if validator.1 == smallest {
                   self.delegate_direct(validator.0 , delta_stake.magnitude());
                   break;
                } 
            }
        } else if delta_stake < 0 {
            // unstake delta from the one with the most
            for validator in validators.iter() {
                if validator.1 == biggest {
                   self.undelegate_direct(validator.0 , &delta_stake.magnitude());
                   break;
                } 
            }
        }
    }

}
