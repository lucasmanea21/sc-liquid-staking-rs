elrond_wasm::imports!();
elrond_wasm::derive_imports!();

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

    // Stake

    fn update_exchange_rate(&self) {
        let total_staked = self.total_staked().get();
        let total_token_supply = self.total_token_supply().get();

        self.exchange_rate().set(total_token_supply / total_staked);
    }

    #[only_owner]
    #[endpoint]
    fn set_total_staked(&self, amount: BigUint) {
        self.total_staked().set(amount);
        self.update_exchange_rate();
    }

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

    // Token

    #[view(getStEgldId)]
    #[storage_mapper("staked_egld_id")]
    fn staked_egld_id(&self) -> SingleValueMapper<TokenIdentifier>;
}