elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait HelpersModule: 
    crate::storage::StorageModule {

    // TODO: transform all the increment functions into a single function

    #[inline]
    fn increment_index(&self) {
       
        let mapping_index = self.mapping_index().get();
        let current_epoch = self.blockchain().get_block_epoch();

        self.mapping_index().set(&mapping_index + &1);
        
        if &mapping_index >= &(self.validators().len()) {
            self.mapping_index().set(1 as usize);
            self.stake_info_finished().insert(current_epoch);
        }
    }

    #[inline]
    fn increment_index_rewards(&self) {
        let mapping_index = self.rewards_mapping_index().get();
        let current_epoch = self.blockchain().get_block_epoch();

        self.rewards_mapping_index().set(&mapping_index + &1);
        
        if &mapping_index >= &(self.validators().len()) {
            self.rewards_mapping_index().set(1 as usize);
            self.rewards_info_finished().insert(current_epoch);
        }
    }

    #[inline]
    fn increment_index_withdraw(&self) {
        let mapping_index = self.withdraw_mapping_index().get();
        let current_epoch = self.blockchain().get_block_epoch();

        self.withdraw_mapping_index().set(&mapping_index + &1);
        
        if &mapping_index >= &(self.validators().len()) {
            self.withdraw_mapping_index().set(1 as usize);
            self.withdraw_finished().insert(current_epoch);
        }
    }

    #[inline]
    fn increment_index_redelegate(&self) {
        let mapping_index = self.redelegate_mapping_index().get();
        let current_epoch = self.blockchain().get_block_epoch();

        self.redelegate_mapping_index().set(&mapping_index + &1);
        
        if &mapping_index >= &(self.validators().len()) {
            self.redelegate_mapping_index().set(1 as usize);
            self.redelegate_finished().insert(current_epoch);
        }
    }

    #[inline]
    fn update_protocol_revenue(&self, epoch: &u64) {
        if self.rewards_info_finished().contains(&epoch) == true {
            let rewards_value = self.rewards_amounts().get(&epoch);
            let protocol_fee = self.service_fee().get();

            self.protocol_revenue().set(match rewards_value {
                Some(n) => n,
                None => BigUint::from(0u64)
            } * protocol_fee  / 1000u64);
        }

    }

}