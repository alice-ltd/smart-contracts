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

async function broadcastTx(wallet: Wallet, tx: Tx) {
  const result = await wallet.lcd.tx.broadcastSync(tx);
  for (let i = 0; i < 50; i++) {
    // query txhash
    const data = await wallet.lcd.tx.txInfo(result.txhash).catch(() => {});
    // if hash is onchain return data
    if (data) return data;
    // else wait 250ms and then repeat
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error('Transaction is not onchain');
}

export async function broadcastSingleMsg(
  wallet: Wallet,
  msg: Msg,
  sequence?: number,
  logMsg: boolean = true
) {
  const tx = await wallet.createAndSignTx({
    msgs: [msg],
    sequence,
  });

  // console.log(JSON.stringify(tx.toData(), null, 2));
  const result = await broadcastTx(wallet, tx);
  if (isTxError(result)) {
    throw new Error('msg error: ' + result.code + ' ' + result.raw_log);
  }

  console.log('success: ', result.txhash);
  if (logMsg) {
    console.log(msg);
  }
  return result;
}

export async function queryUusdBalance(lcd: LCDClient, address: string) {
  const [coins] = await lcd.bank.balance(address);
  return coins.get('uusd')!.amount;
}

export const bombay = new LCDClient(BOMBAY);
export const columbus = new LCDClient(COLUMBUS);
