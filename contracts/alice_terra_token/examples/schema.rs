use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw20::{BalanceResponse, TokenInfoResponse};

use alice_terra_token::msg::{
    ExchangeRateResponse, ExecuteMsg, InstantiateMsg, MetaTx, MigrateMsg, QueryMsg,
    RelayNonceResponse,
};
use alice_terra_token::state::Config;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);

    export_schema(&schema_for!(MetaTx), &out_dir);

    export_schema(&schema_for!(BalanceResponse), &out_dir);
    export_schema(&schema_for!(TokenInfoResponse), &out_dir);
    export_schema(&schema_for!(ExchangeRateResponse), &out_dir);
    export_schema(&schema_for!(RelayNonceResponse), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
}
