import {
  BlockTxBroadcastResult,
  isTxError,
  LCDClient,
  Msg,
  Tx,
  Wallet,
} from '@terra-money/terra.js';

const COLUMBUS = {
  URL: 'https://lcd.terra.dev',
  chainID: 'columbus-5',
  gasPrices: {
    uusd: 0.15,
  },
};

const BOMBAY = {
  URL: process.env.BOMBAY_LCD ?? 'https://bombay.stakesystems.io',
  chainID: 'bombay-12',
  gasPrices: {
    uusd: 0.15,
  },
};

async function broadcastTx(wallet: Wallet, tx: Tx, retries = 3) {
  let result;
  for (let i = 0; i < retries; i++) {
    try {
      result = await wallet.lcd.tx.broadcast(tx);
      break;
    } catch (e: any) {
      if (
        i < retries - 1 &&
        e?.response?.data?.message ===
          'timed out waiting for tx to be included in a block'
      ) {
        console.log(e);
        continue;
      }
      throw e;
    }
  }
  return result as BlockTxBroadcastResult;
}

export async function broadcastSingleMsg(
  wallet: Wallet,
  msg: Msg,
  sequence: number
) {
  const tx = await wallet.createAndSignTx({
    msgs: [msg],
    sequence,
  });

  // console.log(JSON.stringify(tx.toData(), null, 2));
  const result = await broadcastTx(wallet, tx, 3);
  if (isTxError(result)) {
    throw new Error('msg error: ' + result.code + ' ' + result.raw_log);
  }
  // console.log(result);

  console.log('success: ', result.txhash, msg);

  return result;
}

export async function queryUusdBalance(lcd: LCDClient, address: string) {
  const [coins] = await lcd.bank.balance(address);
  return coins.get('uusd')!.amount;
}

export const bombay = new LCDClient(BOMBAY);
export const columbus = new LCDClient(COLUMBUS);
