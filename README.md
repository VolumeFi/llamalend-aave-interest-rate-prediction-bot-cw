# Llamalend AAVE interest rate prediction bot CosmWasm smart contract

This is the Cosmwasm smart contract to manage AAVE interest rate prediction game vyper contract on EVM chain (ARB) written in Vyper.

## ExecuteMsg

### SetPaloma

Run `set_paloma` function on CompetitionArb Vyper smart contract to register this contract address data in the Vyper contract.

| Key | Type | Description |
|-----|------|-------------|
| -   | -    | -           |

### UpdateCompass

Run `update_compass` function on CompetitionArb Vyper smart contract to update the EVM-compass address.

| Key         | Type   | Description                                               |
|-------------|--------|-----------------------------------------------------------|
| new_compass | String | New evm-compass address for competitionArb vyper contract |

### SetWinnerList

Run `set_winner_list` function on CompetitionArb Vyper smart contract to set the winner list of the current epoch.

| Key          | Type            | Description                |
|--------------|-----------------|----------------------------|
| winner_infos | Vec<WinnerInfo> | Array of WinnerInfo struct |

## QueryMsg

### GetJobId

Get `job_id` of Paloma message to run functions on a CompetitionArb Vyper smart contract.

| Key | Type | Description |
|-----|------|-------------|
| -   | -    | -           |

#### Response

| Key    | Type   | Description      |
|--------|--------|------------------|
| job_id | String | Job Id on Paloma |

## Structs

### WinnerInfo

| Key              | Type    | Description                  |
|------------------|---------|------------------------------|
| winner           | String  | winner address               |
| claimable_amount | Uint256 | Reward amount of this winner |