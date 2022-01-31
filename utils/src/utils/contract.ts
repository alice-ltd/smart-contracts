import * as fs from 'fs';
import {
  Int,
  isTxError,
  LCDClient,
  MsgExecuteContract,
  MsgInstantiateContract,
  MsgStoreCode,
  Wallet,
} from '@terra-money/terra.js';
import * as crypto from 'crypto';
import * as path from 'path';
import { broadcastSingleMsg } from './terra';

export async function uploadCode(
  wallet: Wallet,
  contractPath: string,
  sequence: number,
  ignoreCache?: boolean
) {
  const contractCode = fs.readFileSync(contractPath);

  const hashSum = crypto.createHash('sha256');
  hashSum.update(contractCode);
  const hashHex = hashSum.digest('hex');

  let codeId = -1;
  let codeIds: Record<string, Record<string, number>> = {};

  const cacheDir = path.join(__dirname, '../../cache/');
  if (!fs.existsSync(cacheDir)) {
    fs.mkdirSync(cacheDir);
  }

  const chainID = wallet.lcd.config.chainID;
  try {
    const codeIdsJson = fs.readFileSync(path.join(cacheDir, 'code_ids.json'));
    codeIds = JSON.parse(codeIdsJson.toString());
    if (codeIds?.[chainID]?.[hashHex] && !ignoreCache) {
      codeId = codeIds[chainID][hashHex];
      console.log('Cached Code ID', codeId);
    }
  } catch (e) {
    console.log('No cache/code_ids.json found');
  }

  if (codeId === -1) {
    const upload = new MsgStoreCode(
      wallet.key.accAddress,
      contractCode.toString('base64')
    );
    const result = await broadcastSingleMsg(wallet, upload, sequence, false);

    codeId = parseInt(
      result.logs[0].events
        .filter((e) => e.type === 'store_code')[0]
        .attributes.filter((attr) => attr.key === 'code_id')[0].value
    );

    console.log('Uploaded Code ID', codeId);
    sequence += 1;
  }

  fs.writeFileSync(
    path.join(cacheDir, 'code_ids.json'),
    JSON.stringify({
      ...codeIds,
      [chainID]: {
        ...codeIds[chainID],
        [hashHex]: codeId,
      },
    })
  );

  return { codeId, sequence };
}

export async function deployContract(
  wallet: Wallet,
  codeId: number,
  initMsg: object,
  sequence?: number,
  admin?: string
) {
  const instantiate = new MsgInstantiateContract(
    wallet.key.accAddress,
    admin,
    codeId,
    initMsg
  );

  const result = await broadcastSingleMsg(wallet, instantiate, sequence, false);
  if (isTxError(result)) {
    throw new Error(
      'instantiate contract error: ' + result.code + ' ' + result.raw_log
    );
  }

  const contractAddress =
    result.logs[0].eventsByType['instantiate_contract']['contract_address'][0];
  console.log('Deployed contract', contractAddress);

  return contractAddress;
}

export async function createDepositStableMsg({
  contractAddr,
  sender,
  recipient,
  uusd_amount,
}: {
  contractAddr: string;
  sender: string;
  recipient: string;
  uusd_amount: number;
}) {
  return new MsgExecuteContract(
    sender, // sender
    contractAddr, // contract account address
    { deposit_stable: { recipient } }, // handle msg
    { uusd: uusd_amount } // coins
  );
}

export async function createRedeemStableMsg({
  contractAddr,
  sender,
  recipient,
  burn_amount,
}: {
  contractAddr: string;
  sender: string;
  recipient: string;
  burn_amount: string;
}) {
  return new MsgExecuteContract(
    sender, // sender
    contractAddr, // contract account address
    { redeem_stable: { recipient, burn_amount } }, // handle msg
    {} // coins
  );
}

export async function queryCW20Balance({
  lcd,
  contractAddr,
  address,
}: {
  lcd: LCDClient;
  contractAddr: string;
  address: string;
}) {
  const query = await lcd.wasm.contractQuery<{ balance: string }>(
    contractAddr,
    {
      balance: { address: address },
    }
  );

  return new Int(query.balance);
}

export async function createTransferMsg({
  sender,
  contractAddr,
  recipient,
  aliceUusdAmount,
}: {
  sender: string;
  contractAddr: string;
  recipient: string;
  aliceUusdAmount: number;
}) {
  return new MsgExecuteContract(
    sender, // sender
    contractAddr, // contract account address
    {
      transfer: {
        recipient,
        amount: aliceUusdAmount.toString(),
      },
    }, // handle msg
    {} // coins
  );
}
