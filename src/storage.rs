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

    #[view(getEpochValidators)]
    #[storage_mapper("epoch_validators")]
    fn epoch_validators(&self) -> VecMapper<ManagedAddress>;

    #[view(getUsedValidators)]
    #[storage_mapper("used_validators")]
    fn used_validators(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getValidators)]
    #[storage_mapper("validators")]
    fn validators(&self) -> VecMapper<ManagedAddress>;

    #[view(getValidatorStakeAmount)]
    #[storage_mapper("validator_stake_amount")]
    fn validator_stake_amount(&self, address: &ManagedAddress) -> SingleValueMapper<BigUint>;
    

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


    #[view(getDeltaStake)]
    #[storage_mapper("delta_stake")]
    fn delta_stake(&self) -> SingleValueMapper<BigInt>;


    #[view(getFilteredStakeAmountsLength)]
    #[storage_mapper("filtered_stake_amounts_length")]
    fn filtered_stake_amounts_length(&self) -> SingleValueMapper<usize>;


    #[view(getFilteredStakeAmounts)]
    #[storage_mapper("filtered_stake_amounts")]
    fn filtered_stake_amounts(&self) -> SingleValueMapper<ManagedVec<StakeAmount<Self::Api>>>;

    #[view(getRewardsAmount)]
    #[storage_mapper("rewards_amount")]
    fn rewards_amount(&self) -> VecMapper<RewardsAmount<Self::Api>>;


    #[view(getCallbackResult)]
    #[storage_mapper("callback_result")]
    fn callback_result(&self) -> SingleValueMapper<BigUint>;

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

    // Maintenance info

    #[view(getStakeAmounts)]
    #[storage_mapper("stake_amounts")]
    fn stake_amounts(&self) -> MapMapper<u64,BigUint>;

    #[view(getRewardsAmounts)]
    #[storage_mapper("rewards_amounts")]
    fn rewards_amounts(&self) -> MapMapper<u64,BigUint>;


    #[view(getStakeInfoFinished)]
    #[storage_mapper("stake_info_finished")]
    fn stake_info_finished(&self) -> SingleValueMapper<bool>;


    #[view(getRewardsInfoFinished)]
    #[storage_mapper("rewards_info_finished")]
    fn rewards_info_finished(&self) -> SingleValueMapper<bool>;
    

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
    fn set_validator_stake_amount(&self, validator: &ManagedAddress, amount: &BigUint) {
        self.validator_stake_amount(&validator).set(amount);
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
    #[endpoint(clearValidators)]
    fn clear_validators(&self) {
        self.validators().clear();
    }

}