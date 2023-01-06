elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode,TypeAbi, Clone)]
pub struct StakeAmount<M: ManagedTypeApi> {
    pub epoch: u64,
    pub amount: BigUint<M>
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode,TypeAbi, Clone)]
pub struct RewardsAmount<M: ManagedTypeApi> {
    pub epoch: u64,
    pub amount: BigUint<M>
}

#[elrond_wasm::module]
pub trait StorageModule {

    // Validators

    #[view(getValidatorsCount)]
    fn get_validators_count(&self) {
        self.validators().len();
    }

    #[view(getValidators)]
    #[storage_mapper("validators")]
    fn validators(&self) -> VecMapper<ManagedAddress>;

    #[view(getValidatorStakeAmount)]
    #[storage_mapper("validator_stake_amount")]
    fn validator_stake_amount(&self) -> MapMapper<ManagedAddress,BigUint>;

    //delete
    #[view(getFlag)]
    #[storage_mapper("flag")]
    fn flag(&self) -> SingleValueMapper<BigUint>;
    

    // Stake

    #[view(getTotalStaked)]
    #[storage_mapper("total_staked")]
    fn total_staked(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalTokenSupply)]
    #[storage_mapper("total_token_supply")]
    fn total_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[view(getExchangeRate)]
    #[storage_mapper("exchange_rate")]
    fn exchange_rate(&self) -> SingleValueMapper<BigUint>;

    #[view(getExchangeRateMultiplier)]
    #[storage_mapper("exchange_rate_multiplier")]
    fn exchange_rate_multiplier(&self) -> SingleValueMapper<BigUint>;

    #[view(getMinValue)]
    #[storage_mapper("min_value")]
    fn min_value(&self) -> SingleValueMapper<BigUint>;

    #[view(getDeltaStake)]
    #[storage_mapper("delta_stake")]
    fn delta_stake(&self) -> SingleValueMapper<BigInt>;

    #[view(getRewardsAmount)]
    #[storage_mapper("rewards_amount")]
    fn rewards_amount(&self) -> VecMapper<RewardsAmount<Self::Api>>;

    #[view(getProtocolRevenue)]
    #[storage_mapper("protocol_revenue")]
    fn protocol_revenue(&self) -> SingleValueMapper<BigUint>;

    #[view(getServiceFee)]
    #[storage_mapper("service_fee")]
    fn service_fee(&self) -> SingleValueMapper<BigUint>;

    // Mappers
    // - used for the maintenance tasks


    #[view(getMappingIndex)]
    #[storage_mapper("mapping_index")]
    fn mapping_index(&self) -> SingleValueMapper<usize>;

    #[view(getRewardsMappingIndex)]
    #[storage_mapper("rewards_mapping_index")]
    fn rewards_mapping_index(&self) -> SingleValueMapper<usize>;

    #[view(getWithdrawMappingIndex)]
    #[storage_mapper("withdraw_mapping_index")]
    fn withdraw_mapping_index(&self) -> SingleValueMapper<usize>;

    #[view(getStakeValue)]
    #[storage_mapper("stake_value")]
    fn stake_value(&self) -> SingleValueMapper<BigUint>;

    #[view(getRedelegateMappingIndex)]
    #[storage_mapper("redelegate_mapping_index")]
    fn redelegate_mapping_index(&self) -> SingleValueMapper<usize>;

    // Maintenance info

    #[view(getStakeAmounts)]
    #[storage_mapper("stake_amounts")]
    fn stake_amounts(&self) -> MapMapper<u64,BigUint>;

    #[view(getRewardsAmounts)]
    #[storage_mapper("rewards_amounts")]
    fn rewards_amounts(&self) -> MapMapper<u64,BigUint>;


    #[view(getStakeInfoFinished)]
    #[storage_mapper("stake_info_finished")]
    fn stake_info_finished(&self) -> SetMapper<u64>;

    #[view(getRewardsInfoFinished)]
    #[storage_mapper("rewards_info_finished")]
    fn rewards_info_finished(&self) -> SetMapper<u64>;

    #[view(getWithdrawFinished)]
    #[storage_mapper("withdraw_finished")]
    fn withdraw_finished(&self) -> SetMapper<u64>;

    #[view(getRedelegateFinished)]
    #[storage_mapper("redelegate_finished")]
    fn redelegate_finished(&self) -> SetMapper<u64>;

    #[view(getExchangeRateUpdateFinished)]
    #[storage_mapper("exchange_rate_update_finished")]
    fn exchange_rate_update_finished(&self) -> SetMapper<u64>;


    #[view(getStakeInfoStarted)]
    #[storage_mapper("stake_info_started")]
    fn stake_info_started(&self) -> SetMapper<u64>;

    #[view(getRewardsInfoStarted)]
    #[storage_mapper("rewards_info_started")]
    fn rewards_info_started(&self) -> SetMapper<u64>;

    #[view(getWithdrawStarted)]
    #[storage_mapper("withdraw_started")]
    fn withdraw_started(&self) -> SetMapper<u64>;

    #[view(getRedelegateStarted)]
    #[storage_mapper("redelegate_started")]
    fn redelegate_started(&self) -> SetMapper<u64>;
    
    

    // Tokens

    #[view(getStEgldId)]
    #[storage_mapper("staked_egld_id")]
    fn staked_egld_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getUEgldId)]
    #[storage_mapper("undelegated_token")]
    fn undelegated_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    /*
        Storage modifiers
    */

    #[only_owner]
    #[endpoint(setValidatorStakeAmount)]
    fn set_validator_stake_amount(&self, validator: ManagedAddress, amount: BigUint) {
        self.validator_stake_amount().insert(validator, amount);
    }

    #[only_owner]
    #[endpoint(setServiceFee)]
    fn set_service_fee(&self, amount: BigUint) {
        self.service_fee().set(amount);
    }

    #[only_owner]
    #[endpoint(setMappingIndex)]
    fn set_mapping_index(&self, index: usize) {
        self.mapping_index().set(index);
    }

    #[only_owner]
    #[endpoint(setRewardsMappingIndex)]
    fn set_rewards_mapping_index(&self, index: usize) {
        self.rewards_mapping_index().set(index);
    }

    #[only_owner]
    #[endpoint(setDeltaStake)]
    fn set_delta_stake(&self, amount: BigInt) {
        self.delta_stake().set(&amount);
    }

    #[only_owner]
    #[endpoint(setTotalStaked)]
    fn set_total_staked(&self, amount: BigUint) {
        self.total_staked().set(amount);
    }

    #[only_owner]
    #[endpoint(clearRewardsAmounts)]
    fn clear_rewards_amounts(&self) {
        self.rewards_amounts().clear();
    }


    #[only_owner]
    #[endpoint(clearRewardsFinished)]
    fn clear_rewards_finished(&self) {
        self.rewards_info_finished().clear();
    }

    #[only_owner]
    #[endpoint(clearRewardsStarted)]
    fn clear_rewards_started(&self) {
        self.rewards_info_started().clear();
    }

    #[only_owner]
    #[endpoint(clearValidatorStakeAmounts)]
    fn clear_validator_stake_amounts(&self) {
        self.validator_stake_amount().clear();
    }

    #[only_owner]
    #[endpoint(clearValidators)]
    fn clear_validators(&self) {
        self.validators().clear();
    }

}