#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetJobIdResponse, InstantiateMsg, Metadata, PalomaMsg, QueryMsg};
use crate::state::{State, STATE};
use cosmwasm_std::CosmosMsg;
use ethabi::{Contract, Function, Param, ParamType, StateMutability, Token, Uint};
use std::collections::BTreeMap;
use std::str::FromStr;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io::llamalend-aave-interest-rate-prediction-bot-cw";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        retry_delay: msg.retry_delay,
        job_id: msg.job_id.clone(),
        owner: info.sender.clone(),
        metadata: Metadata {
            creator: msg.creator,
            signers: msg.signers,
        },
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("job_id", msg.job_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::SetPaloma {} => execute::set_paloma(deps, info),
        ExecuteMsg::UpdateCompass { new_compass } => {
            execute::update_compass(deps, info, new_compass)
        }
        ExecuteMsg::SetWinnerList { winner_infos } => {
            execute::set_winner_list(deps, env, info, winner_infos)
        }
    }
}

pub mod execute {
    use super::*;
    use crate::msg::WinnerInfo;
    use crate::state::WITHDRAW_TIMESTAMP;
    use crate::ContractError::{Unauthorized};
    use ethabi::Address;

    pub fn set_paloma(
        deps: DepsMut,
        info: MessageInfo,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "set_paloma".to_string(),
                vec![Function {
                    name: "set_paloma".to_string(),
                    inputs: vec![],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg {
                job_id: state.job_id,
                payload: Binary(
                    contract
                        .function("set_paloma")
                        .unwrap()
                        .encode_input(&[])
                        .unwrap(),
                ),
                metadata: state.metadata,
            }))
            .add_attribute("action", "set_paloma"))
    }

    pub fn update_compass(
        deps: DepsMut,
        info: MessageInfo,
        new_compass: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        let new_compass_address: Address = Address::from_str(new_compass.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_compass".to_string(),
                vec![Function {
                    name: "update_compass".to_string(),
                    inputs: vec![Param {
                        name: "_new_compass".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg {
                job_id: state.job_id,
                payload: Binary(
                    contract
                        .function("update_compass")
                        .unwrap()
                        .encode_input(&[Token::Address(new_compass_address)])
                        .unwrap(),
                ),
                metadata: state.metadata,
            }))
            .add_attribute("action", "update_compass"))
    }

    #[allow(clippy::vec_init_then_push)]
    pub fn set_winner_list(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        winner_infos: Vec<WinnerInfo>,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if state.owner != info.sender {
            return Err(Unauthorized {});
        }
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "set_winner_list".to_string(),
                vec![Function {
                    name: "set_winner_list".to_string(),
                    inputs: vec![Param {
                        name: "_winner_infos".to_string(),
                        kind: ParamType::Array(Box::new(ParamType::Tuple(vec![
                            ParamType::Address,
                            ParamType::Uint(256),
                        ]))),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };

        let mut token_winner_info: Vec<Token> = vec![];
        let retry_delay: u64 = state.retry_delay;

        for winner_info in winner_infos {
            if let Some(timestamp) = WITHDRAW_TIMESTAMP.may_load(
                deps.storage,
                (winner_info.winner.to_owned(), "set_winner".to_string()),
            )? {
                if timestamp.plus_seconds(retry_delay).lt(&env.block.time) {
                    let mut token_winner_info_element: Vec<Token> = vec![];
                    token_winner_info_element.push(Token::Address(
                        Address::from_str(winner_info.winner.as_str()).unwrap(),
                    ));
                    token_winner_info_element.push(Token::Uint(Uint::from_big_endian(
                        &winner_info.claimable_amount.to_be_bytes(),
                    )));
                    token_winner_info.push(Token::Tuple(token_winner_info_element));
                    WITHDRAW_TIMESTAMP.save(
                        deps.storage,
                        (winner_info.winner.to_owned(), "set_winner".to_string()),
                        &env.block.time,
                    )?;
                }
            } else {
                let mut token_winner_info_element: Vec<Token> = vec![];
                token_winner_info_element.push(Token::Address(
                    Address::from_str(winner_info.winner.as_str()).unwrap(),
                ));
                token_winner_info_element.push(Token::Uint(Uint::from_big_endian(
                    &winner_info.claimable_amount.to_be_bytes(),
                )));
                token_winner_info.push(Token::Tuple(token_winner_info_element));
                WITHDRAW_TIMESTAMP.save(
                    deps.storage,
                    (winner_info.winner.to_owned(), "set_winner".to_string()),
                    &env.block.time,
                )?;
            }
        }
        let token_winners_info: Vec<Token> = vec![Token::Array(token_winner_info)];

        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg {
                job_id: state.job_id,
                payload: Binary(
                    contract
                        .function("set_winner_list")
                        .unwrap()
                        .encode_input(token_winners_info.as_slice())
                        .unwrap(),
                ),
                metadata: state.metadata,
            }))
            .add_attribute("action", "set_winner_list"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetJobId {} => to_json_binary(&query::get_job_id(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn get_job_id(deps: Deps) -> StdResult<GetJobIdResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetJobIdResponse {
            job_id: state.job_id,
        })
    }
}
