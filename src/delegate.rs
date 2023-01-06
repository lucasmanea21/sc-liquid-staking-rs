elrond_wasm::imports!();

#[elrond_wasm::proxy]
pub trait Delegate {

    #[payable("EGLD")]
    #[endpoint(delegate)]
    fn delegate(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
    );

    #[endpoint(unDelegate)]
    fn unDelegate(
        &self,
        amount: BigUint
    );

    #[endpoint(reDelegateRewards)]
    fn reDelegateRewards(
        &self,
    );

    #[endpoint(getUserActiveStake)]
    fn getUserActiveStake(
        &self,
        address: &ManagedAddress
    );
    
    #[endpoint(getClaimableRewards)]
    fn getClaimableRewards(
        &self,
        address: &ManagedAddress
    );

    #[endpoint(withdraw)]
    fn withdraw(&self);
}