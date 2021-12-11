import { deployContract, uploadCode } from '../utils/contract';
import { broadcastSingleMsg, columbus } from '../utils/terra';
import {
  MsgExecuteContract,
  MsgUpdateContractAdmin,
} from '@terra-money/terra.js';
import { CLIKey } from '@terra-money/terra.js/dist/key/CLIKey';

const MONEY_MARKET_ADDR = 'terra1sepfj7s0aeg5967uxnfk4thzlerrsktkpelm5s';
const ATERRA_TOKEN_ADDR = 'terra1hzh9vpxhsk8253se0vv5jj6etdvxu3nv8z07zu';

const contractOwnerKey = new CLIKey({ keyName: 'contractowner' });
const wallet = columbus.wallet(contractOwnerKey);

async function main() {
  let sequence1 = await wallet.sequence();

  await instantiate_all_contracts();

  async function instantiate_all_contracts() {
    console.log('\n\n## MAINNET: instantiate_all_contracts ##');

    // Instantiate overseer & register self
    let overseerCodeId;
    ({ codeId: overseerCodeId, sequence: sequence1 } = await uploadCode(
      wallet,
      '../artifacts/alice_overseer.wasm',
      sequence1++
    ));
    const overseerAddr = await deployContract(
      wallet,
      overseerCodeId,
      {
        owner: wallet.key.accAddress,
        timelock_duration: {
          time: 60, // seconds
        },
      },
      sequence1++,
      wallet.key.accAddress
    );

    const updateAdminToSelf = new MsgUpdateContractAdmin(
      wallet.key.accAddress,
      overseerAddr,
      overseerAddr
    );
    await broadcastSingleMsg(wallet, updateAdminToSelf, sequence1++);

    const registerSelf = new MsgExecuteContract(
      wallet.key.accAddress,
      overseerAddr,
      {
        register: {
          contract_addr: overseerAddr,
        },
      }
    );
    await broadcastSingleMsg(wallet, registerSelf, sequence1++);

    // Instantiate token & register with overseer
    let tokenCodeId;
    ({ codeId: tokenCodeId, sequence: sequence1 } = await uploadCode(
      wallet,
      '../artifacts/alice_terra_token.wasm',
      sequence1++
    ));
    const tokenContractAddr = await deployContract(
      wallet,
      tokenCodeId,
      {
        owner: wallet.key.accAddress,
        name: 'Alice aUST Wrapper',
        decimals: 6,
        symbol: 'aliceUST',
        stable_denom: 'uusd',
        money_market_addr: MONEY_MARKET_ADDR,
        aterra_token_addr: ATERRA_TOKEN_ADDR,
      },
      sequence1++,
      overseerAddr
    );
    const registerToken = new MsgExecuteContract(
      wallet.key.accAddress,
      overseerAddr,
      {
        register: {
          contract_addr: tokenContractAddr,
        },
      }
    );
    await broadcastSingleMsg(wallet, registerToken, sequence1++);
  }
}
main().catch((e) => {
  console.error(e);
  throw e;
});
