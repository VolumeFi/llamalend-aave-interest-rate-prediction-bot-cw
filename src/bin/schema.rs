use cosmwasm_schema::write_api;

use llamalend_aave_interest_rate_prediction_bot_cw::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
