#![no_std]

elrond_wasm::imports!();

mod delegate;
mod storage;
mod callbacks;
mod events;
mod token;

/// One of the simplest smart contracts possible,
/// it holds a single variable in storage, which anyone can increment.
#[elrond_wasm::contract]
pub trait StakeContract:
        elrond_wasm_modules::default_issue_callbacks::DefaultIssueCallbacksModule        
        + storage::StorageModule
        + callbacks::CallbacksModule
        + events::EventsModule
        + token::TokenModule {

    #[proxy]
    fn delegate_contract(&self, sc_address: ManagedAddress) -> delegate::Proxy<Self::Api>;

    #[init]
    fn init(&self) {
        if self.delta_stake().is_empty() { self.delta_stake().set(BigInt::from(0)) };
        if self.exchange_rate().is_empty(){ self.exchange_rate().set(BigUint::from(1u64)) };
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
        let caller = self.blockchain().get_caller();
        let value = self.call_value().egld_value();

        require!(&value > &0, "Stake value must be bigger than 0");

        // mint tokens based on exchange rate
        let exchange_rate = self.exchange_rate().get();
        let st_egld_id = self.staked_egld_id().get();
        let st_egld_amount = exchange_rate * &value; 
        
        self.send().esdt_local_mint(&st_egld_id, 0, &st_egld_amount);
        self.send().direct_esdt(&caller, &st_egld_id, 0, &st_egld_amount);


        let current_total_supply = self.total_token_supply().get();
        self.total_token_supply().set(&current_total_supply + &st_egld_amount);

        self.delta_stake().set(self.delta_stake().get() + &value);
    }

    // Receives stEGLD

    #[payable("*")]
    #[endpoint]
    fn unstake(&self) {
        let caller = self.blockchain().get_caller();
        let (token, _, payment) = self.call_value().single_esdt().into_tuple();
        let st_egld_id = self.staked_egld_id().get();

        require!(&token == &st_egld_id, "Invalid token sent");
        
        self.send().esdt_local_burn(&st_egld_id, 0, &payment);

        self.total_token_supply().set(self.total_token_supply().get() - &payment);
        self.delta_stake().set(self.delta_stake().get() - &payment);

        // todo: keep track of unstaking, so user can withdraw later
    }

    /* 
        Endpoint for delegating EGLD to validators, claiming rewards and other maintenance actions.
        
        Could be replaced by a bot that provides that info, so contract itself won't have to make all the calls.
    */ 
    
    #[only_owner]
    #[endpoint]
    fn delegate(&self) {
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
    
}
