import {
  createDepositStableMsg,
  createRedeemStableMsg,
  createTransferMsg,
  deployContract,
  queryCW20Balance,
  uploadCode,
} from '../utils/contract';
import { broadcastSingleMsg, queryUusdBalance, bombay } from '../utils/terra';
import { wallet1, wallet2 } from '../utils/testAccounts';
import { strict as assert } from 'assert';
import {
  MsgExecuteContract,
  MsgUpdateContractAdmin,
} from '@terra-money/terra.js';

const TESTNET_MONEY_MARKET_ADDR =
  'terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal';
const TESTNET_ATERRA_TOKEN_ADDR =
  'terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl';

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// TODO: use a proper test framework
async function main() {
  let sequence1 = await wallet1.sequence();

  await instantiate_all_contracts();
  await basic_deposit_redeem();
  await migrate_token();
  await migrate_overseer();
  await basic_transfer();

  async function instantiate_all_contracts() {
    console.log('\n\n## TEST: instantiate_all_contracts ##');

    // Instantiate overseer & register self
    let overseerCodeId;
    ({ codeId: overseerCodeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_overseer.wasm',
      sequence1++
    ));
    const overseerAddr = await deployContract(
      wallet1,
      overseerCodeId,
      {
        owner: wallet1.key.accAddress,
        timelock_duration: {
          time: 60, // seconds
        },
      },
      sequence1++,
      wallet1.key.accAddress
    );

    const updateAdminToSelf = new MsgUpdateContractAdmin(
      wallet1.key.accAddress,
      overseerAddr,
      overseerAddr
    );
    await broadcastSingleMsg(wallet1, updateAdminToSelf, sequence1++);

    const registerSelf = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        register: {
          contract_addr: overseerAddr,
        },
      }
    );
    await broadcastSingleMsg(wallet1, registerSelf, sequence1++);

    // Instantiate token & register with overseer
    let tokenCodeId;
    ({ codeId: tokenCodeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_terra_token.wasm',
      sequence1++
    ));
    const tokenContractAddr = await deployContract(
      wallet1,
      tokenCodeId,
      {
        owner: wallet1.key.accAddress,
        name: 'Bob',
        decimals: 6,
        symbol: 'ubob',
        stable_denom: 'uusd',
        money_market_addr: TESTNET_MONEY_MARKET_ADDR,
        aterra_token_addr: TESTNET_ATERRA_TOKEN_ADDR,
        redeem_fee_ratio: '0.005',
        redeem_fee_cap: '25_000_000'
      },
      sequence1++,
      overseerAddr
    );
    const registerToken = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        register: {
          contract_addr: tokenContractAddr,
        },
      }
    );
    await broadcastSingleMsg(wallet1, registerToken, sequence1++);
  }

  async function basic_deposit_redeem() {
    console.log('\n\n## TEST: basic_deposit_redeem ##');

    let codeId;
    ({ codeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_terra_token.wasm',
      sequence1++
    ));
    const contractAddr = await deployContract(
      wallet1,
      codeId,
      {
        owner: wallet1.key.accAddress,
        name: 'Bob',
        decimals: 6,
        symbol: 'ubob',
        stable_denom: 'uusd',
        money_market_addr: TESTNET_MONEY_MARKET_ADDR,
        aterra_token_addr: TESTNET_ATERRA_TOKEN_ADDR,
        redeem_fee_ratio: '0.005',
        redeem_fee_cap: '25_000_000'
      },
      sequence1++,
      wallet1.key.accAddress
    );

    const deposit = await createDepositStableMsg({
      contractAddr,
      sender: wallet1.key.accAddress,
      recipient: wallet1.key.accAddress,
      uusd_amount: 10_000_000,
    });
    await broadcastSingleMsg(wallet1, deposit, sequence1++);
    console.log('sleep 5s...');
    await sleep(5000);

    let balance = await queryCW20Balance({
      lcd: bombay,
      contractAddr,
      address: wallet1.key.accAddress,
    });
    assert(balance.greaterThan(0));

    const uusdBalance = await queryUusdBalance(bombay, wallet1.key.accAddress);
    console.log('UST balance', uusdBalance.dividedBy(1e6));

    const redeem = await createRedeemStableMsg({
      contractAddr,
      sender: wallet1.key.accAddress,
      recipient: wallet1.key.accAddress,
      burn_amount: balance.toString(),
    });
    await broadcastSingleMsg(wallet1, redeem, sequence1++);
    console.log('sleep 5s...');
    await sleep(5000);

    const newUusdBalance = await queryUusdBalance(
      bombay,
      wallet1.key.accAddress
    );
    console.log('new UST balance', newUusdBalance.dividedBy(1e6));
    assert(newUusdBalance.greaterThan(uusdBalance));

    balance = await queryCW20Balance({
      lcd: bombay,
      contractAddr,
      address: wallet1.key.accAddress,
    });
    assert(balance.equals(0));
  }

  async function migrate_token() {
    console.log('\n\n## TEST: migrate_token ##');

    // Instantiate overseer
    let overseerCodeId;
    ({ codeId: overseerCodeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_overseer.wasm',
      sequence1++
    ));
    const overseerAddr = await deployContract(
      wallet1,
      overseerCodeId,
      {
        owner: wallet1.key.accAddress,
        timelock_duration: {
          time: 15, // seconds
        },
      },
      sequence1++,
      wallet1.key.accAddress
    );

    // Instantiate token & register with overseer
    let tokenCodeId;
    ({ codeId: tokenCodeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_terra_token.wasm',
      sequence1++
    ));
    const tokenContractAddr = await deployContract(
      wallet1,
      tokenCodeId,
      {
        owner: wallet1.key.accAddress,
        name: 'Bob',
        decimals: 6,
        symbol: 'ubob',
        stable_denom: 'uusd',
        money_market_addr: TESTNET_MONEY_MARKET_ADDR,
        aterra_token_addr: TESTNET_ATERRA_TOKEN_ADDR,
        redeem_fee_ratio: '0.005',
        redeem_fee_cap: '25_000_000'
      },
      sequence1++,
      overseerAddr
    );
    const registerToken = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        register: {
          contract_addr: tokenContractAddr,
        },
      }
    );
    await broadcastSingleMsg(wallet1, registerToken, sequence1++);

    // upload "new" token
    let newTokenCodeId;
    ({ codeId: newTokenCodeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_terra_token.wasm',
      sequence1++,
      true
    ));

    const initiateMigrate = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        initiate_migrate: {
          contract_addr: tokenContractAddr,
          msg: Buffer.from('{}').toString('base64'),
          new_code_id: newTokenCodeId,
        },
      }
    );
    await broadcastSingleMsg(wallet1, initiateMigrate, sequence1++);

    const migrate = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        migrate: {
          contract_addr: tokenContractAddr,
        },
      }
    );
    assert.rejects(() => broadcastSingleMsg(wallet1, migrate, sequence1));
    console.log('success: timelock not expired error ', migrate);

    // wait for timelock & try again
    console.log('sleeping 20s for timelock expiration...');
    await sleep(20 * 1000);
    await broadcastSingleMsg(wallet1, migrate, sequence1++);
  }

  async function migrate_overseer() {
    console.log('\n\n## TEST: migrate_overseer ##');

    // Instantiate overseer & register self
    let overseerCodeId;
    ({ codeId: overseerCodeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_overseer.wasm',
      sequence1++
    ));
    const overseerAddr = await deployContract(
      wallet1,
      overseerCodeId,
      {
        owner: wallet1.key.accAddress,
        timelock_duration: {
          time: 15, // seconds
        },
      },
      sequence1++,
      wallet1.key.accAddress
    );

    const updateAdminToSelf = new MsgUpdateContractAdmin(
      wallet1.key.accAddress,
      overseerAddr,
      overseerAddr
    );
    await broadcastSingleMsg(wallet1, updateAdminToSelf, sequence1++);

    const registerSelf = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        register: {
          contract_addr: overseerAddr,
        },
      }
    );
    await broadcastSingleMsg(wallet1, registerSelf, sequence1++);

    // upload "new" overseer
    let newOverseerCodeId;
    ({ codeId: newOverseerCodeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_overseer.wasm',
      sequence1++,
      true
    ));

    const initiateMigrate = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        initiate_migrate: {
          contract_addr: overseerAddr,
          msg: Buffer.from(
            JSON.stringify({ timelock_duration: { time: 60 } })
          ).toString('base64'),
          new_code_id: newOverseerCodeId,
        },
      }
    );
    await broadcastSingleMsg(wallet1, initiateMigrate, sequence1++);

    const migrate = new MsgExecuteContract(
      wallet1.key.accAddress,
      overseerAddr,
      {
        migrate: {
          contract_addr: overseerAddr,
        },
      }
    );
    assert.rejects(() => broadcastSingleMsg(wallet1, migrate, sequence1));
    console.log('success: timelock not expired error ', migrate);

    // wait for timelock & try again
    console.log('sleeping 20s for timelock expiration...');
    await sleep(20 * 1000);
    await broadcastSingleMsg(wallet1, migrate, sequence1++);
  }

  async function basic_transfer() {
    console.log('\n\n## TEST: basic_transfer ##');

    let codeId;
    ({ codeId, sequence: sequence1 } = await uploadCode(
      wallet1,
      '../artifacts/alice_terra_token.wasm',
      sequence1++
    ));
    const contractAddr = await deployContract(
      wallet1,
      codeId,
      {
        owner: wallet1.key.accAddress,
        name: 'Bob',
        decimals: 6,
        symbol: 'ubob',
        stable_denom: 'uusd',
        money_market_addr: TESTNET_MONEY_MARKET_ADDR,
        aterra_token_addr: TESTNET_ATERRA_TOKEN_ADDR,
        redeem_fee_ratio: '0.005',
        redeem_fee_cap: '25_000_000'
      },
      sequence1++,
      wallet1.key.accAddress
    );

    const deposit = await createDepositStableMsg({
      contractAddr,
      sender: wallet1.key.accAddress,
      recipient: wallet1.key.accAddress,
      uusd_amount: 10_000_000,
    });
    await broadcastSingleMsg(wallet1, deposit, sequence1++);
    console.log('sleep 5s...');
    await sleep(5000);

    const balance1 = await queryCW20Balance({
      lcd: bombay,
      contractAddr,
      address: wallet1.key.accAddress,
    });
    const balance2 = await queryCW20Balance({
      lcd: bombay,
      contractAddr,
      address: wallet2.key.accAddress,
    });

    const transfer = await createTransferMsg({
      contractAddr,
      sender: wallet1.key.accAddress,
      recipient: wallet2.key.accAddress,
      aliceUusdAmount: 5_000_000,
    });
    await broadcastSingleMsg(wallet1, transfer, sequence1++);
    console.log('sleep 5s...');
    await sleep(5000);

    const newBalance1 = await queryCW20Balance({
      lcd: bombay,
      contractAddr,
      address: wallet1.key.accAddress,
    });
    const newBalance2 = await queryCW20Balance({
      lcd: bombay,
      contractAddr,
      address: wallet2.key.accAddress,
    });

    assert(newBalance1.lessThan(balance1));
    assert(newBalance2.greaterThan(balance2));
  }
}
main().catch((e) => {
  console.error(e);
  process.exit(1);
});
