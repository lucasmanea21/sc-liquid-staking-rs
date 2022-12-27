elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::module]
pub trait HelpersModule: 
    crate::storage::StorageModule {

    #[inline]
    fn increment_index(&self) {
        let mapping_index = self.mapping_index().get();

        self.mapping_index().set(&mapping_index + &1);
        
        if &mapping_index >= &(self.validators().len()) {
            self.mapping_index().set(1 as usize);
            self.stake_info_finished().set(true);
        }
    }

    #[inline]
    fn increment_index_rewards(&self) {
        let mapping_index = self.rewards_mapping_index().get();

        self.rewards_mapping_index().set(&mapping_index + &1);
        
        if &mapping_index >= &(self.validators().len()) {
            self.rewards_mapping_index().set(1 as usize);
            self.rewards_info_finished().set(true);
        }
    }

    #[inline]
    fn update_protocol_revenue(&self, epoch: &u64) {
        if self.rewards_info_finished().get() == true {
            let rewards_value = self.rewards_amounts().get(&epoch);
            let protocol_fee = self.service_fee().get();

            self.protocol_revenue().set(match rewards_value {
                Some(n) => n,
                None => BigUint::from(0u64)
            } * protocol_fee  / 1000u64);
        }

    }

}