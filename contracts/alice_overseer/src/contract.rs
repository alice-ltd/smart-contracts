use crate::error::ContractError;

use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, MigrateTimelockResponse, QueryMsg,
    RegisteredContractsResponse,
};
use crate::state::{
    config_mut, config_read, contracts_mut, contracts_read, migrate_timelocks_mut,
    migrate_timelocks_read, Config, MigrateTimelock,
};
use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Pair,
    Response, StdResult, WasmMsg,
};
use cw0::{calc_range_start, maybe_addr, Duration, Expiration};
use cw2::set_contract_version;
use terra_cosmwasm::TerraQuerier;

const CONTRACT_NAME: &str = "crates.io:alice-overseer";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    config_mut(deps.storage).save(&Config {
        owner: deps.api.addr_validate(msg.owner.as_str())?,
        timelock_duration: msg.timelock_duration,
    })?;

    // initialize CW2
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register { contract_addr } => execute_register(deps, env, info, contract_addr),
        ExecuteMsg::InitiateMigrate {
            contract_addr,
            new_code_id,
            msg,
        } => execute_initiate_migrate(deps, env, info, contract_addr, new_code_id, msg),
        ExecuteMsg::Migrate { contract_addr } => execute_migrate(deps, env, info, contract_addr),
        ExecuteMsg::CancelMigrate { contract_addr } => {
            execute_cancel_migrate(deps, env, info, contract_addr)
        }
    }
}

fn execute_migrate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_addr: String,
) -> Result<Response, ContractError> {
    if info.sender != config_read(deps.storage).load()?.owner {
        return Err(ContractError::Unauthorized {});
    }

    let contract_addr_bytes = contract_addr.as_bytes();

    let maybe_timelock = migrate_timelocks_read(deps.storage).may_load(contract_addr_bytes)?;

    let timelock = maybe_timelock.ok_or(ContractError::TimelockNotFound())?;

    if !timelock.expiration.is_expired(&env.block) {
        return Err(ContractError::TimelockNotExpired());
    }

    migrate_timelocks_mut(deps.storage).remove(contract_addr_bytes);

    Ok(
        Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr,
            new_code_id: timelock.new_code_id,
            msg: timelock.msg,
        })),
    )
}

fn execute_initiate_migrate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_addr: String,
    new_code_id: u64,
    msg: Binary,
) -> Result<Response, ContractError> {
    let config = config_read(deps.storage).load()?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let contract_addr_bytes = contract_addr.as_bytes();
    if !contracts_read(deps.storage).load(contract_addr_bytes)? {
        return Err(ContractError::UnregisteredContract());
    }

    if migrate_timelocks_read(deps.storage).may_load(contract_addr_bytes)? != None {
        return Err(ContractError::ExistingTimelock());
    }

    let duration = config.timelock_duration;
    let expiration = match duration {
        Duration::Height(h) => Expiration::AtHeight(env.block.height + h),
        Duration::Time(t) => Expiration::AtTime(env.block.time.plus_seconds(t)),
    };

    migrate_timelocks_mut(deps.storage).save(
        contract_addr_bytes,
        &MigrateTimelock {
            expiration,
            new_code_id,
            msg,
        },
    )?;

    Ok(Response::default())
}

fn execute_cancel_migrate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
) -> Result<Response, ContractError> {
    if info.sender != config_read(deps.storage).load()?.owner {
        return Err(ContractError::Unauthorized {});
    }

    let contract_addr_bytes = contract_addr.as_bytes();
    migrate_timelocks_mut(deps.storage).remove(contract_addr_bytes);

    Ok(Response::default())
}

fn execute_register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract_addr: String,
) -> Result<Response, ContractError> {
    if info.sender != config_read(deps.storage).load()?.owner {
        return Err(ContractError::Unauthorized {});
    }

    let addr = deps.api.addr_validate(contract_addr.as_str())?;

    // Make sure the contract admin is the overseer
    let terra_querier = TerraQuerier::new(&deps.querier);
    let contract_info = terra_querier.query_contract_info(contract_addr.as_str())?;
    if contract_info.admin != Some(env.contract.address.to_string()) {
        return Err(ContractError::ContractAdminNotOverseer());
    }

    contracts_mut(deps.storage).save(addr.as_bytes(), &true)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::RegisteredContracts { start_after, limit } => {
            to_binary(&query_registered_contracts(deps, start_after, limit)?)
        }
        QueryMsg::MigrateTimelock { contract_addr } => {
            to_binary(&query_migrate_timelock(deps, contract_addr)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = config_read(deps.storage).load()?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        timelock_duration: config.timelock_duration,
    })
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_registered_contracts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<RegisteredContractsResponse> {
    // start_after, limit: Reference Anchor-Protocol/money-market-contracts

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(maybe_addr(deps.api, start_after)?);
    let contracts_state: Vec<Pair<_>> = contracts_read(deps.storage)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .collect::<Result<Vec<_>, _>>()?;

    // Filter value is true, convert key to String
    let contracts = contracts_state
        .into_iter()
        .map(|(k, _)| k)
        .map(String::from_utf8)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(RegisteredContractsResponse { contracts })
}

fn query_migrate_timelock(deps: Deps, contract_addr: String) -> StdResult<MigrateTimelockResponse> {
    let contract_addr_bytes = contract_addr.as_bytes();

    Ok(MigrateTimelockResponse {
        timelock: migrate_timelocks_read(deps.storage).may_load(contract_addr_bytes)?,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> StdResult<Response> {
    let mut config = config_read(deps.storage).load()?;

    if let Some(owner) = msg.owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    if let Some(timelock_duration) = msg.timelock_duration {
        config.timelock_duration = timelock_duration;
    }

    config_mut(deps.storage).save(&config)?;

    // Update CW2 version info
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::mock_querier::WasmMockQuerier;
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, MigrateTimelockResponse, QueryMsg, RegisteredContractsResponse,
    };
    use crate::state::MigrateTimelock;
    use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{
        from_binary, CosmosMsg, DepsMut, Env, OwnedDeps, Response, SubMsg, WasmMsg,
    };
    use cw0::{Duration, Expiration};
    use terra_cosmwasm::ContractInfoResponse;

    /// mock_dependencies is a drop-in replacement for cosmwasm_std::testing::mock_dependencies
    pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
        let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::new(&[]));

        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: custom_querier,
        }
    }

    fn instantiate_contract(deps: DepsMut, sender: &str) -> (Response, Env) {
        let msg = InstantiateMsg {
            owner: sender.to_string(),
            timelock_duration: Duration::Time(7 * 24 * 60 * 60), // 1 week
        };
        let env = mock_env();
        let info = mock_info(sender, &[]);

        let res = instantiate(deps, env.clone(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        (res, env)
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let (_res, env) = instantiate_contract(deps.as_mut(), "owner");

        // Verify status query
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::RegisteredContracts {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let value: RegisteredContractsResponse = from_binary(&res).unwrap();
        assert_eq!(value.contracts.len(), 0);
    }

    #[test]
    fn register_contract() {
        let mut deps = mock_dependencies();
        let (_res, env) = instantiate_contract(deps.as_mut(), "owner");
        let overseer_address = env.contract.address.to_string();

        // make the overseer contract the admin of a mock "contract1"
        deps.querier.with_contract_info(&[(
            &"contract1".to_string(),
            &ContractInfoResponse {
                address: "contract1".to_string(),
                creator: "creator1".to_string(),
                code_id: 123,
                admin: Some(overseer_address),
            },
        )]);

        let info = mock_info("owner", &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::Register {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 0);

        // Verify status query
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::RegisteredContracts {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let response: RegisteredContractsResponse = from_binary(&res).unwrap();
        assert_eq!(response.contracts, vec!["contract1".to_string()]);
    }

    #[test]
    fn register_contract_not_owner() {
        let mut deps = mock_dependencies();
        let (_res, env) = instantiate_contract(deps.as_mut(), "owner");
        let overseer_address = env.contract.address.to_string();

        // make the overseer contract the admin of a mock "contract1"
        deps.querier.with_contract_info(&[(
            &"contract1".to_string(),
            &ContractInfoResponse {
                address: "contract1".to_string(),
                creator: "creator1".to_string(),
                code_id: 123,
                admin: Some(overseer_address),
            },
        )]);

        // "random" sending register should error
        let info = mock_info("random", &[]);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::Register {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap_err();
    }

    #[test]
    fn initiate_migrate_unregistered_contract() {
        let mut deps = mock_dependencies();
        let (_res, env) = instantiate_contract(deps.as_mut(), "owner");

        // migrate an unregistered contract - should error
        let info = mock_info("owner", &[]);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::InitiateMigrate {
                contract_addr: "random".to_string(),
                new_code_id: 124,
                msg: Default::default(),
            },
        )
        .unwrap_err();
    }

    #[test]
    fn initiate_migrate_with_existing_timelock() {
        let mut deps = mock_dependencies();
        let (_res, env) = instantiate_contract(deps.as_mut(), "owner");
        let overseer_address = env.contract.address.to_string();

        // make the overseer contract the admin of a mock "contract1"
        deps.querier.with_contract_info(&[(
            &"contract1".to_string(),
            &ContractInfoResponse {
                address: "contract1".to_string(),
                creator: "creator1".to_string(),
                code_id: 123,
                admin: Some(overseer_address),
            },
        )]);

        // Register contract1
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::Register {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();

        // initiate migrate
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::InitiateMigrate {
                contract_addr: "contract1".to_string(),
                new_code_id: 124,
                msg: Default::default(),
            },
        )
        .unwrap();

        // initiate migrate again - should error
        execute(
            deps.as_mut(),
            env,
            mock_info("owner", &[]),
            ExecuteMsg::InitiateMigrate {
                contract_addr: "contract1".to_string(),
                new_code_id: 124,
                msg: Default::default(),
            },
        )
        .unwrap_err();
    }

    #[test]
    fn migrate_contract() {
        let mut deps = mock_dependencies();
        let (_res, mut env) = instantiate_contract(deps.as_mut(), "owner");
        let overseer_address = env.contract.address.to_string();

        // make the overseer contract the admin of a mock "contract1"
        deps.querier.with_contract_info(&[(
            &"contract1".to_string(),
            &ContractInfoResponse {
                address: "contract1".to_string(),
                creator: "creator1".to_string(),
                code_id: 123,
                admin: Some(overseer_address),
            },
        )]);

        // register contract1
        let info = mock_info("owner", &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::Register {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();

        // start migrate timelock
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::InitiateMigrate {
                contract_addr: "contract1".to_string(),
                new_code_id: 124,
                msg: Default::default(),
            },
        )
        .unwrap();

        env.block.time = env.block.time.plus_seconds(7 * 24 * 60 * 60); // 1 week

        // execute migrate
        let res = execute(
            deps.as_mut(),
            env,
            mock_info("owner", &[]),
            ExecuteMsg::Migrate {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();

        assert_eq!(
            res.messages,
            vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Migrate {
                contract_addr: "contract1".to_string(),
                new_code_id: 124,
                msg: Default::default(),
            }))]
        );
    }

    #[test]
    fn migrate_contract_timelock_not_expired() {
        let mut deps = mock_dependencies();
        let (_res, mut env) = instantiate_contract(deps.as_mut(), "owner");
        let overseer_address = env.contract.address.to_string();

        // make the overseer contract the admin of a mock "contract1"
        deps.querier.with_contract_info(&[(
            &"contract1".to_string(),
            &ContractInfoResponse {
                address: "contract1".to_string(),
                creator: "creator1".to_string(),
                code_id: 123,
                admin: Some(overseer_address),
            },
        )]);

        // register contract1
        let info = mock_info("owner", &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::Register {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();

        // start migrate timelock
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::InitiateMigrate {
                contract_addr: "contract1".to_string(),
                new_code_id: 124,
                msg: Default::default(),
            },
        )
        .unwrap();

        // not quite 1 week
        env.block.time = env.block.time.plus_seconds(7 * 24 * 60 * 59);

        // execute migrate - should error
        execute(
            deps.as_mut(),
            env,
            mock_info("owner", &[]),
            ExecuteMsg::Migrate {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap_err();
    }

    #[test]
    fn migrate_contract_no_timelock_initiated() {
        let mut deps = mock_dependencies();
        let (_res, mut env) = instantiate_contract(deps.as_mut(), "owner");
        let overseer_address = env.contract.address.to_string();

        // make the overseer contract the admin of a mock "contract1"
        deps.querier.with_contract_info(&[(
            &"contract1".to_string(),
            &ContractInfoResponse {
                address: "contract1".to_string(),
                creator: "creator1".to_string(),
                code_id: 123,
                admin: Some(overseer_address),
            },
        )]);

        // register contract1
        let info = mock_info("owner", &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::Register {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();

        env.block.time = env.block.time.plus_seconds(7 * 24 * 60 * 60); // 1 week

        // Did not start timelock with InitiateMigrate
        // execute migrate - should error
        execute(
            deps.as_mut(),
            env,
            mock_info("owner", &[]),
            ExecuteMsg::Migrate {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap_err();
    }

    #[test]
    fn cancel_migrate() {
        let mut deps = mock_dependencies();
        let (_res, env) = instantiate_contract(deps.as_mut(), "owner");
        let overseer_address = env.contract.address.to_string();

        // make the overseer contract the admin of a mock "contract1"
        deps.querier.with_contract_info(&[(
            &"contract1".to_string(),
            &ContractInfoResponse {
                address: "contract1".to_string(),
                creator: "creator1".to_string(),
                code_id: 123,
                admin: Some(overseer_address),
            },
        )]);

        // register contract1
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::Register {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();

        // initiate migrate timelock
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::InitiateMigrate {
                contract_addr: "contract1".to_string(),
                new_code_id: 124,
                msg: Default::default(),
            },
        )
        .unwrap();

        // check timelock set
        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::MigrateTimelock {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();
        let value: MigrateTimelockResponse = from_binary(&res).unwrap();
        assert_eq!(
            value.timelock,
            Some(MigrateTimelock {
                expiration: Expiration::AtTime(env.block.time.plus_seconds(7 * 24 * 60 * 60)),
                new_code_id: 124,
                msg: Default::default()
            })
        );

        // cancel migrate timelock
        execute(
            deps.as_mut(),
            env.clone(),
            mock_info("owner", &[]),
            ExecuteMsg::CancelMigrate {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();

        // check no timelock set
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::MigrateTimelock {
                contract_addr: "contract1".to_string(),
            },
        )
        .unwrap();
        let value: MigrateTimelockResponse = from_binary(&res).unwrap();
        assert_eq!(value.timelock, None);
    }
}
