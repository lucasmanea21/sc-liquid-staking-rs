ALICE="./wallets/new-wallet.pem"
MY_ADDRESS=erd1epacy29dkrkqaeju3k59z45rdq5c9a2dv4qs0t0992d32prx623slv5fq5
ADDRESS=$(erdpy data load --key=address-testnet)
ADDRESS_HEX="$(erdpy wallet bech32 --decode ${ADDRESS})"

DELEGATION_ADDRESS=erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxhllllssz7sl7
DELEGATION_ADDRESS_HEX="$(erdpy wallet bech32 --decode ${DELEGATION_ADDRESS})"

DEPLOY_TRANSACTION=$(erdpy data load --key=deployTransaction-testnet)

PROXY=https://devnet-api.elrond.com
CHAIN_ID=D

NEW_TOKEN_NAME="STEGLD"
NEW_TOKEN_NAME_HEX="$(echo -n ${NEW_TOKEN_NAME} | xxd -p -u | tr -d '\n')"
TOKEN_ID="STEGLD-7294c4"
TOKEN_ID_HEX="$(echo -n ${TOKEN_ID} | xxd -p -u | tr -d '\n')"

UNDELEGATED_TOKEN_NAME="UEGLD"
UNDELEGATED_TOKEN_NAME_HEX="$(echo -n ${UNDELEGATED_TOKEN_NAME} | xxd -p -u | tr -d '\n')"

UNDELEGATED_TOKEN_ID="STEGLD-e5139f"
UNDELEGATED_TOKEN_ID_HEX="$(echo -n ${UNDELEGATED_TOKEN_ID} | xxd -p -u | tr -d '\n')"

deploy() {
    erdpy --verbose contract deploy --project=${PROJECT} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --send --outfile="deploy-testnet.interaction.json" --proxy=${PROXY} --metadata-payable --chain=${CHAIN_ID} || return

    TRANSACTION=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['emittedTransactionHash']")
    ADDRESS=$(erdpy data parse --file="deploy-testnet.interaction.json" --expression="data['contractAddress']")

    erdpy data store --key=address-testnet --value=${ADDRESS}
    erdpy data store --key=deployTransaction-testnet --value=${TRANSACTION}

    echo ""
    echo "Smart contract address: ${ADDRESS}"
}

upgrade() {
    erdpy --verbose contract upgrade ${ADDRESS} --project=${PROJECT} --recall-nonce --pem=${ALICE} --send --outfile="upgrade.json" --proxy=${PROXY} --chain=${CHAIN_ID} \
        --metadata-payable \
        --gas-limit=100000000
}

issueToken() {
    erdpy --verbose contract call ${ADDRESS} \
        --recall-nonce --pem=${ALICE} \
        --gas-limit=600000000 \
        --value=50000000000000000 \
        --arguments 0x${NEW_TOKEN_NAME_HEX} 0x${NEW_TOKEN_NAME_HEX} --function="issueToken" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

issueUndelegatedToken() {
    erdpy --verbose contract call ${ADDRESS} \
        --recall-nonce --pem=${ALICE} \
        --gas-limit=600000000 \
        --value=50000000000000000 \
        --arguments 0x${UNDELEGATED_TOKEN_NAME_HEX} 0x${UNDELEGATED_TOKEN_NAME_HEX} 0x12 --function="issueUndelegatedToken" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setLocalRoles() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
        --gas-limit=500000000 --function="setLocalRoles" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

stake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} \
        --gas-limit=50000000 --value=1000000000000000000 --function="stake" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

unstake() {
    erdpy --verbose tx new --receiver=${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 \
        --send \
        --proxy=${PROXY} \
        --data="ESDTTransfer@${TOKEN_ID_HEX}@b1a2bc2ec50000@756e7374616b65" \
        --chain=${CHAIN_ID}
    # --data="ESDTTransfer@${TOKEN_ID_HEX}@075bb48691a6650ff30904000000@756e7374616b65" \
}

claim() {
    erdpy --verbose tx new --receiver=${MY_ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 \
        --send \
        --proxy=${PROXY} \
        --data="ESDTNFTTransfer@${UNDELEGATED_TOKEN_ID_HEX}@01@b1a2bc2ec50000@${ADDRESS_HEX}@636c61696d" \
        --chain=${CHAIN_ID}
    # --data="ESDTTransfer@${TOKEN_ID_HEX}@b1a2bc2ec50000@756e7374616b65" \
}

setTotalStaked() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="set_total_staked" \
        --arguments=0x8ac7230489e80000 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

redelegateAdmin() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=60000000 --function="redelegateAdmin" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setDeltaStake() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="setDeltaStake" \
        --arguments=0x00 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

# 0xDE0B6B3A7640000 = 1 egld
# 0x8ac7230489e80000 = 10 egld
# 0xF21F494C589C0000 = -1
getStakeAdmin() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="getStakeAdmin" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getRewardsAdmin() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="getRewardsAdmin" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearValidators() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="clearValidators" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

updateExchangeRate() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="updateExchangeRate" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

dailyDelegation() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="dailyDelegation" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

withdrawAdmin() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="withdrawAdmin" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

push_validators() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="push_validators" --arguments 0x${DELEGATION_ADDRESS_HEX} --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setMappingIndex() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="setMappingIndex" --arguments=2 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setRewardsMappingIndex() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="setRewardsMappingIndex" --arguments=2 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearRewardsAmounts() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="clearRewardsAmounts" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearRewardsStarted() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="clearRewardsStarted" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearWithdrawStarted() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="clearWithdrawStarted" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

clearRewardsFinished() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="clearRewardsFinished" --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setServiceFee() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="setServiceFee" \
        --arguments=0x4b --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

setValidatorStakeAmount() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="setValidatorStakeAmount" \
        --arguments 0x${DELEGATION_ADDRESS_HEX} 0x8ac7230489e80000 --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

distributeProtocolRevenue() {
    erdpy --verbose contract call ${ADDRESS} --recall-nonce --pem=${ALICE} --gas-limit=50000000 --function="distributeProtocolRevenue" \
        --send --proxy=${PROXY} --chain=${CHAIN_ID}
}

getValidators() {
    erdpy --verbose contract query ${ADDRESS} --function="getValidators" --proxy=${PROXY}
}

getWithdrawMappingIndex() {
    erdpy --verbose contract query ${ADDRESS} --function="getWithdrawMappingIndex" --proxy=${PROXY}
}

getStakeValue() {
    erdpy --verbose contract query ${ADDRESS} --function="getStakeValue" --proxy=${PROXY}
}

getProtocolRevenue() {
    erdpy --verbose contract query ${ADDRESS} --function="getProtocolRevenue" --proxy=${PROXY}
}

getRedelegateStarted() {
    erdpy --verbose contract query ${ADDRESS} --function="getRedelegateStarted" --proxy=${PROXY}
}

getRedelegateFinished() {
    erdpy --verbose contract query ${ADDRESS} --function="getRedelegateFinished" --proxy=${PROXY}
}

getValidatorStakeAmount() {
    erdpy --verbose contract query ${ADDRESS} --function="getValidatorStakeAmount" --proxy=${PROXY}
}

getValidatorStakeAmountClone() {
    erdpy --verbose contract query ${ADDRESS} --function="getValidatorStakeAmountClone" --proxy=${PROXY}
}

getRewardsInfoFinished() {
    erdpy --verbose contract query ${ADDRESS} --function="getRewardsInfoFinished" --proxy=${PROXY}
}

getStakeInfoFinished() {
    erdpy --verbose contract query ${ADDRESS} --function="getStakeInfoFinished" --proxy=${PROXY}
}

getWithdrawFinished() {
    erdpy --verbose contract query ${ADDRESS} --function="getWithdrawFinished" --proxy=${PROXY}
}

getRewardsAmounts() {
    erdpy --verbose contract query ${ADDRESS} --function="getRewardsAmounts" --proxy=${PROXY}
}

getValidatorsCount() {
    erdpy --verbose contract query ${ADDRESS} --function="getValidatorsCount" --proxy=${PROXY}
}

getFilteredStakeAmountsLength() {
    erdpy --verbose contract query ${ADDRESS} --function="getFilteredStakeAmountsLength" --proxy=${PROXY}
}

getFilteredStakeAmounts() {
    erdpy --verbose contract query ${ADDRESS} --function="getFilteredStakeAmounts" --proxy=${PROXY}
}

getStakeAmounts() {
    erdpy --verbose contract query ${ADDRESS} --function="getStakeAmounts" --proxy=${PROXY}
}

getServiceFee() {
    erdpy --verbose contract query ${ADDRESS} --function="getServiceFee" --proxy=${PROXY}
}

getCallbackResult() {
    erdpy --verbose contract query ${ADDRESS} --function="getCallbackResult" --proxy=${PROXY}
}

getMappingIndex() {
    erdpy --verbose contract query ${ADDRESS} --function="getMappingIndex" --proxy=${PROXY}
}

getRedelegateMappingIndex() {
    erdpy --verbose contract query ${ADDRESS} --function="getRedelegateMappingIndex" --proxy=${PROXY}
}

getRewardsMappingIndex() {
    erdpy --verbose contract query ${ADDRESS} --function="getRewardsMappingIndex" --proxy=${PROXY}
}

getTotalStaked() {
    erdpy --verbose contract query ${ADDRESS} --function="getTotalStaked" --proxy=${PROXY}

}

getDeltaStake() {
    erdpy --verbose contract query ${ADDRESS} --function="getDeltaStake" --proxy=${PROXY}
}

getTotalTokenSupply() {
    erdpy --verbose contract query ${ADDRESS} --function="getTotalTokenSupply" --proxy=${PROXY}
}

clearValidatorStakeAmounts() {
    erdpy --verbose contract query ${ADDRESS} --function="clearValidatorStakeAmounts" --proxy=${PROXY}
}

getExchangeRate() {
    erdpy --verbose contract query ${ADDRESS} --function="getExchangeRate" --proxy=${PROXY}
}

getExchangeRateMultiplier() {
    erdpy --verbose contract query ${ADDRESS} --function="getExchangeRateMultiplier" --proxy=${PROXY}
}

getDailyDelegationFinished() {
    erdpy --verbose contract query ${ADDRESS} --function="getDailyDelegationFinished" --proxy=${PROXY}
}

getUEgldId() {
    erdpy --verbose contract query ${ADDRESS} --function="getUEgldId" --proxy=${PROXY}
}

getFlag() {
    erdpy --verbose contract query ${ADDRESS} --function="getFlag" --proxy=${PROXY}
}
