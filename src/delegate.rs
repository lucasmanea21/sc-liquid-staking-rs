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
    
}