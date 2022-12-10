#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

mod delegate;
mod storage;
mod callbacks;
mod events;
mod tokens;

use crate::tokens::TokenAttributes;

#[elrond_wasm::contract]
pub trait StakeContract:
        elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule        
        + storage::StorageModule
        + callbacks::CallbacksModule
        + events::EventsModule
        + tokens::TokenModule {


    #[proxy]
    fn delegate_contract(&self, sc_address: ManagedAddress) -> delegate::Proxy<Self::Api>;

    #[init]
    fn init(&self) {
        if self.delta_stake().is_empty() { self.delta_stake().set(BigInt::from(0)) };
        if self.exchange_rate().is_empty(){ self.exchange_rate().set(BigUint::from(1u64)) };
        if self.mapping_index().is_empty(){ self.mapping_index().set(1 as usize) };
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
        self.send().direct_esdt(&caller, &st_egld_id, 0, &st_egld_amount);

        self.total_token_supply().set(&current_total_supply + &st_egld_amount);
        self.delta_stake().set(&current_delta_stake + &value);
    }

    // Receives stEGLD

    #[payable("*")]
    #[endpoint]
    fn unstake(&self) {
        let (token, _, payment) = self.call_value().single_esdt().into_tuple();
        let st_egld_id = self.staked_egld_id().get();
        require!(&token == &st_egld_id, "Invalid token sent");
        
        let caller = self.blockchain().get_caller();
        let current_total_supply = self.total_token_supply().get();
        let current_delta_stake = self.delta_stake().get();
    
        self.send().esdt_local_burn(&st_egld_id, 0, &payment);
        self.total_token_supply().set(&current_total_supply - &payment);
        self.delta_stake().set(&current_delta_stake - &payment);

        // todo: keep track of unstaking, so user can withdraw later
        
        /*
            mint uEGLD based on exchange rate (mint the EGLD equivalent)
        */  
        let exchange_rate = self.exchange_rate().get();

        let attr = &TokenAttributes {
            epoch: self.blockchain().get_block_epoch()
        };


        self.create_and_send_locked_assets(
            &payment / &exchange_rate,
            &caller,
            &attr,
        );
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

        let token_info = self.blockchain().get_esdt_token_data(
            &sc_address,
            &token,
            nonce,
        );

        let attr = self.serializer()
            .top_decode_from_managed_buffer::<TokenAttributes>(
                &token_info.attributes,
            );

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
        // require there hasn't been a delegation this epoch

        let validators = self.validators();
        let mut epoch_validators = self.epoch_validators();

        if epoch_validators.len() == 0 {
            for validator in validators.iter() {
                epoch_validators.push(&validator);
            }
        }

        // todo: replace balance with delta_stake -> delta stake will not change when delegation happens, so amount will be consistent
        let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        let length = validators.len();
        let amount_per_validator = BigUint::from(balance / BigUint::from(length));
     
        self.delegate_contract(epoch_validators.get(1))
            .delegate(EgldOrEsdtTokenIdentifier::egld(), amount_per_validator)
            .async_call()
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint]
    fn push_validators(&self, address: &ManagedAddress) {
        self.validators().push(address);
    }

    #[only_owner]
    #[endpoint(delegateAdmin)]
    fn delegate_admin(&self) {
        let mapping_index = self.mapping_index().get();
        let wanted_address = self.validators().get(mapping_index);

        self.mapping_index().set(&mapping_index + 1);
        
        if &mapping_index >= &(self.validators().len() - 1) {
            self.mapping_index().set(1 as usize);
        }

        self.delegate_contract(wanted_address)
            .delegate(EgldOrEsdtTokenIdentifier::egld(), BigUint::from(1000000000000000000u64))
            .async_call()
            .call_and_exit();

    }

    #[only_owner]
    #[endpoint]
    fn reDelegate(&self, address: ManagedAddress) {
        self.delegate_contract(address)
        .reDelegateRewards()
        .async_call()
        .call_and_exit();
    }

    #[only_owner]
    #[endpoint]
    fn undelegateAdmin(&self, address: ManagedAddress, amount: &BigUint) {
        self.delegate_contract(address)
        .unDelegate(amount)
        .async_call()
        .call_and_exit();
    }
    
}
