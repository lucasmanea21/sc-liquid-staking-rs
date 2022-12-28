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
    }

    // #[endpoint]
    // fn daily_delegation(&self) {
    //     // check if delta_stake is positive or not

    //     /*
    //         delta_stake is positive
    //     */
    //     // stakeable_validators = round::floor(delta_stake, 1)
    //     // amount_per_validator = delta_stake / stakeable_validators
    //     // interate through addresses and call delegate with amount_per_validator

    //     /*
    //         delta_stake is negative
    //     */
    //     // TODO
    // }

    // Receives EGLD, mints and sends stEGLD

    #[only_owner]
    #[endpoint]
    fn daily_delegation(&self) {
        let mut delta = self.delta_stake().get();
        let validators = self.validators();
        let validators_len = self.validators().len();

        while delta > 0 && delta != 1 {
            let MAX: u32 = <u32>::max_value();
            let mut smallest_count = 0;
            let mut smallest = BigUint::from(MAX as u32);
            let mut second_smallest = BigUint::from(MAX as u32);

            // Search for smallest and second_smallest
            for validator in validators.iter() {
                if self.validator_stake_amount(&validator).get() < smallest {
                    second_smallest = smallest;
                    smallest = self.validator_stake_amount(&validator).get();
                } else if self.validator_stake_amount(&validator).get() < second_smallest
                    && self.validator_stake_amount(&validator).get() != smallest
                {
                    second_smallest = self.validator_stake_amount(&validator).get();
                }
            }

            // Calculate difference
            let difference = second_smallest - &smallest;

            // Count smallest in array
            for validator in validators.iter() {
                if self.validator_stake_amount(&validator).get() == smallest {
                    smallest_count += 1;
                }
            }

            // Max per item
            let smallest_count_bigint = BigInt::from(smallest_count);
            let max_per_validator = &delta / &smallest_count_bigint;

            let max_per_validator_biguint = max_per_validator.magnitude();

            let difference_bigint = BigInt::from(difference.clone());

            // Update values in storage
            for validator in validators.iter() {
                if self.validator_stake_amount(&validator).get() == smallest {
                    if difference_bigint > max_per_validator {
                        let amount = self.validator_stake_amount(&validator).get();
                        let new_value = &amount * &max_per_validator.magnitude();
                        self.validator_stake_amount(&validator).set(&new_value);

                        let value_validator=new_value-&amount;
                        self.delegate_direct(validator, value_validator);
                    } else {
                        let amount = self.validator_stake_amount(&validator).get();
                        let new_value = &amount * &difference;
                        self.validator_stake_amount(&validator).set(&new_value);
                        let value_validator=new_value-&amount;
                        self.delegate_direct(validator, value_validator);
                    }
                }
            }

            //delta decrease

            if difference_bigint > max_per_validator {
                delta -= &max_per_validator * &smallest_count_bigint
            } else {
                delta -= &difference_bigint * &smallest_count_bigint;
            }
        }
    }

    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self) {
        let value = self.call_value().egld_value();

        require!(&value > &0, "Stake value must be bigger than 0");

        let caller = self.blockchain().get_caller();
        let exchange_rate = self.exchange_rate().get();

        let st_egld_id = self.staked_egld_id().get();
        let st_egld_amount = exchange_rate * &value;

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

        let attr = &TokenAttributes {
            epoch: self.blockchain().get_block_epoch(),
        };

        self.create_and_send_assets(&payment / &exchange_rate, &caller, &attr);
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

    /*
        Endpoint for delegating EGLD to validators, claiming rewards and other maintenance actions.

        Could be replaced by a bot that provides that info, so contract itself won't have to make all the calls.
    */

    #[only_owner]
    #[endpoint]
    fn delegate_test(&self) {
        // solution for delegating inside SC
        // todo: make this work
        // requires there hasn't been a delegation this epoch

        let validators = self.validators();
        let mut epoch_validators = self.epoch_validators();

        if epoch_validators.len() == 0 {
            for validator in validators.iter() {
                epoch_validators.push(&validator);
            }
        }

        // todo: replace balance with delta_stake -> delta stake will not change when delegation happens, so amount will be consistent
        let balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        let length = validators.len();
        let amount_per_validator = BigUint::from(balance / BigUint::from(length));

        self.delegate_contract(epoch_validators.get(1))
            .delegate(EgldOrEsdtTokenIdentifier::egld(), amount_per_validator)
            .async_call()
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint(getStakeAdmin)]
    fn get_stake_admin(&self) {
        // solution for getting how much the SC has staked to validators inside the SC
        // could be replaced with off-chain daemon

        let current_epoch = self.blockchain().get_block_epoch();
        let mapping_index = self.mapping_index().get();
        let sc_address = self.blockchain().get_sc_address();

        // let current_epoch_amounts: _= self.stake_amounts().iter().filter(|amount| amount.epoch == current_epoch).collect::<ManagedVec<StakeAmount<Self::Api>>>();
        let epoch_exists = self.stake_amounts().contains_key(&current_epoch);

        // self.filtered_stake_amounts_length().set(&current_epoch_amounts.len());
        // self.filtered_stake_amounts().set(&current_epoch_amounts);

        require!(
            !(epoch_exists && mapping_index == 1),
            "already got stake amount this epoch"
        );

        if !epoch_exists {
            self.mapping_index().set(1 as usize);
            self.stake_info_finished().set(false);
        };

        let wanted_address = self.validators().get(mapping_index);

        self.increment_index();

        self.delegate_contract(wanted_address)
            .getUserActiveStake(sc_address)
            .async_call()
            .with_callback(StakeContract::callbacks(self).get_stake_callback(current_epoch))
            .call_and_exit();
    }

    // todo: move this to callbacks.rs
    #[callback]
    fn get_stake_callback(
        &self,
        current_epoch: u64,
        #[call_result] result: ManagedAsyncCallResult<BigUint>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(value) => {
                let old_value = self.stake_amounts().get(&current_epoch);

                self.stake_amounts().insert(
                    current_epoch,
                    match old_value {
                        Some(n) => n,
                        None => BigUint::from(0u64),
                    } + value.clone(),
                );

                self.callback_result().set(&value);
            }
            ManagedAsyncCallResult::Err(err) => {}
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
        let epoch_exists = self.rewards_amounts().contains_key(&current_epoch);

        // self.filtered_stake_amounts_length().set(&current_epoch_amounts.len());
        // self.filtered_stake_amounts().set(&current_epoch_amounts);

        require!(
            !(epoch_exists && mapping_index == 1),
            "already got rewards amount this epoch"
        );

        if !epoch_exists {
            self.rewards_mapping_index().set(1 as usize);
            self.stake_info_finished().set(false);
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
        match result {
            ManagedAsyncCallResult::Ok(value) => {
                let old_value = self.rewards_amounts().get(&current_epoch);
                let mapping_index = self.mapping_index().get();

                self.rewards_amounts().insert(
                    current_epoch,
                    match old_value {
                        Some(n) => n,
                        None => BigUint::from(0u64),
                    } + value.clone(),
                );

                self.update_protocol_revenue(&current_epoch);

                self.callback_result().set(&value);
            }
            ManagedAsyncCallResult::Err(err) => {}
        }
    }

    #[only_owner]
    #[endpoint]
    fn push_validators(&self, address: &ManagedAddress) {
        self.validators().push(address);
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

        self.delegate_contract(address)
            .delegate(EgldOrEsdtTokenIdentifier::egld(), amount)
            .async_call()
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint]
    fn undelegate_direct(&self, address: ManagedAddress, amount: &BigUint) {
        // directly undelegates from the contract specified.

        self.delegate_contract(address)
            .unDelegate(amount)
            .async_call()
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint(redelegate)]
    fn redelegate(&self) {
        // redelegates rewards at each validator.
        // should be done after computing rewards

        let current_epoch = self.blockchain().get_block_epoch();
        let epoch_exists = self.rewards_amounts().contains_key(&current_epoch);
        let is_rewards_info_finished = self.rewards_info_finished().get();
        let validators = self.validators();

        // todo: stop it from running twice in an epoch

        require!(
            epoch_exists && is_rewards_info_finished,
            "must get rewards first"
        );

        for address in validators.iter() {
            self.delegate_contract(address)
                .reDelegateRewards()
                .with_gas_limit(7000000)
                .transfer_execute();
        }
    }

    // #[only_owner]
    // #[endpoint(claimProtocolRewards)]
    // fn claim_protocol_rewards(&self) {
    //     // based on the rewards amount of the epoch, mint 7.5% of it of stEGLD

    //     let rewards = self.rewards_amount().get();
    //     let protocol_fee = self.service_fee().get();
    //     let protocol_rewards = (protocol_fee / 1000)* rewards;
    //     let owner = self.blockchain().get_owner_address();

    //     let st_egld_id = self.staked_egld_id().get();

    //     self.send().esdt_local_mint(&st_egld_id, 0, &protocol_rewards);
    //     self.send().direct_esdt(&owner, &st_egld_id, 0, &protocol_rewards);

    // }
}
