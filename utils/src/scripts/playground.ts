import {
  createDepositStableMsg,
  deployContract,
  uploadCode,
} from '../utils/contract';
import { bombay, broadcastSingleMsg } from '../utils/terra';
import {
  AuthorizationGrant,
  BasicAllowance, Fee, Int,
  isTxError,
  ModeInfo,
  MsgExecuteContract,
  MsgGrantAllowance, MsgGrantAuthorization, MsgRevokeAllowance,
  MsgSend, SendAuthorization, SignerData
} from '@terra-money/terra.js';
import { wallet1, wallet2, wallet3 } from '../utils/testAccounts';

const TESTNET_MONEY_MARKET_ADDR =
  'terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal';
const TESTNET_ATERRA_TOKEN_ADDR =
  'terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl';

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  // let sequence1 = await wallet1.sequence();

  // const codeId = await uploadCode('../artifacts/alice_terra_token.wasm');
  // const codeId = 19415;
  // const contractAddr = await deployContract(
  //   codeId,
  //   {
  //     owner: 'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf',
  //     name: 'Bob',
  //     decimals: 6,
  //     symbol: 'ubob',
  //     stable_denom: 'uusd',
  //     money_market_addr: TESTNET_MONEY_MARKET_ADDR,
  //     aterra_token_addr: TESTNET_ATERRA_TOKEN_ADDR,
  //   },
  //   "terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf"
  // );

  const contractAddr = 'terra1k25sfrksjwu8h33f4dpgjzwcwayf6rsg235guz';

  // console.log(wallet3.key.accAddress)

  // const [grants] = await wallet1.lcd.authz.grants(wallet1.key.accAddress, "terra198y6lrprq5f67wftwf92xksjumzvw7dk99nyzr");
  // console.log(grants)

  const feegrant = new MsgGrantAllowance(
    wallet1.key.accAddress,
    wallet3.key.accAddress,
    new BasicAllowance(
      { uusd: (0.2e6).toString() },
      new Date(Date.now() + 1000 * 60 * 60 * 24 * 365)
    )
  );
  // const feegrant = new MsgRevokeAllowance(
  //   wallet1.key.accAddress,
  //   "terra198y6lrprq5f67wftwf92xksjumzvw7dk99nyzr"
  // );
  // const authorize = new MsgGrantAuthorization(
  //   wallet2.key.accAddress,
  //   wallet3.key.accAddress,
  //   new AuthorizationGrant(
  //     new SendAuthorization({uusd: new Int(1e76)}),
  //     new Date(Date.now() + 1000 * 60 * 60 * 24 * 365 * 10)
  //   ),
  // );
  //
  //
  // const signerDatas: SignerData[] = [];
  // let sequenceNumber;
  // let publicKey;
  // const account = await wallet3.lcd.auth.accountInfo(wallet3.key.accAddress);
  // sequenceNumber = account.getSequenceNumber();
  // publicKey = account.getPublicKey();
  // signerDatas.push({
  //   sequenceNumber,
  //   publicKey,
  // });
  //
  // const fee = await wallet3.lcd.tx.estimateFee(signerDatas, {
  //   msgs: [authorize]
  // });
  //
  // const tx = await wallet3.createAndSignTx({
  //   msgs: [authorize],
  //   fee: Fee.fromData({
  //     ...fee.toData(),
  //     granter: wallet1.key.accAddress,
  //   })
  // });

  // const deposit = await createDepositStableMsg({
  //   contractAddr,
  //   sender: wallet1.key.accAddress,
  //   recipient: wallet3.key.accAddress,
  //   uusd_amount: 100_000_000,
  // });
  // await broadcastSingleMsg(wallet1, deposit, sequence1++);
  // console.log('sleep 5s...');
  // await sleep(5000);

  /*const msg1 = await createTransferMsg(
    'terra14t7mukq06ty7p64cr2xvhvkty56jfh7e70qnkg',
    contractAddr,
    'terra1qxg3zwwzzcp4k9pyzfz366qzlejq6j3y8skvuu',
    1_000_000);

  const fee = new Fee(1000000, [new Coin(Denom.USD, 1000000)],
    'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf');

  const {account_number: account_number2, sequence: sequence2} = await wallet2.accountNumberAndSequence();
  const {account_number, sequence} = await wallet.accountNumberAndSequence();

  const txBody = new TxBody([msg1], "", 0);
  const authInfo = new AuthInfo([
    new SignerInfo(
      new SimplePublicKey(wallet2.key.publicKey!.toString('base64')),
      sequence2,
      new ModeInfo(new ModeInfo.Single(SignMode.SIGN_MODE_DIRECT))
    ),
    new SignerInfo(
      new SimplePublicKey(wallet.key.publicKey!.toString('base64')),
      sequence,
      new ModeInfo(new ModeInfo.Single(SignMode.SIGN_MODE_DIRECT))
    )
  ], fee);
  let tx = new Tx(
    txBody,
    authInfo,
    []
  );

  const signDoc2 = new SignDoc(terra.config.chainID, account_number2, sequence2, authInfo, txBody);
  const sigBytes2 = (await wallet2.key.sign(Buffer.from(signDoc2.toBytes()))).toString(
    'base64'
  );

  const signDoc = new SignDoc(terra.config.chainID, account_number, sequence, authInfo, txBody);
  const sigBytes = (await wallet.key.sign(Buffer.from(signDoc.toBytes()))).toString(
    'base64'
  );

  tx.signatures.push(sigBytes2);
  tx.signatures.push(sigBytes);*/

  /*const date = new Date();
  date.setFullYear(date.getFullYear() + 1);
  const authorize = new MsgGrantAuthorization(
    'terra14t7mukq06ty7p64cr2xvhvkty56jfh7e70qnkg',
    contractAddr,
    new AuthorizationGrant(new SendAuthorization({[Denom.USD]: 100_000_000}), date)
  )
*/

  // const migrate = new MsgMigrateContract(
  //   'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf',
  //   contractAddr,
  //   codeId,
  //   {}
  // )

  // const deposit = await createDepositStableMsg(
  //   'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf',
  //   contractAddr,
  //   'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf',
  //   100_000_000
  // );
  // const deposit2 = await createDepositStableMsg(
  //   'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf',
  //   contractAddr,
  //   'terra14t7mukq06ty7p64cr2xvhvkty56jfh7e70qnkg',
  //   100_000_000
  // );
  // const deposit3 = await createDepositStableMsg(
  //   'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf',
  //   contractAddr,
  //   'terra1qxg3zwwzzcp4k9pyzfz366qzlejq6j3y8skvuu',
  //   100_000_000
  // );
  //

  // const redeem = new MsgExecuteContract(
  //   wallet1.key.accAddress, // sender
  //   contractAddr, // contract account address
  //   {
  //     redeem_stable: {
  //       recipient: wallet1.key.accAddress,
  //       burn_amount: '10000000',
  //     },
  //   }, // handle msg
  //   {} // coins
  // );

  // const msgSend = new MsgSend(
  //   wallet1.key.accAddress, // sender
  //   'terra1ph6qqd9khfxp7ak88nvep5ktz05ce2ke760d8f', // recipient
  //   { uusd: 1_000_000 } // amount
  // );

  // const authorizedDeposit = new MsgExecuteContract(
  //   'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf', // sender
  //   contractAddr, // contract account address
  //   {
  //     deposit_stable_authorized: {
  //       recipient: 'terra14t7mukq06ty7p64cr2xvhvkty56jfh7e70qnkg',
  //       amount: '100000000'
  //     }
  //   }, // handle msg
  //   {uusd: '1000000'} // coins
  // );

  // const send = new MsgSend('terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf', contractAddr, {uusd: '10000000'})

  // let {account_number, sequence} = await wallet1.accountNumberAndSequence()
  //
  // const tx = await wallet1.createAndSignTx({
  //   msgs: [msgSend],
  //   sequence,
  //   accountNumber: account_number,
  //   fee: new Fee(1000000, {uusd: 500000}),
  // });
  //
  // const tx2 = await wallet1.createAndSignTx({
  //   msgs: [msgSend],
  //   sequence: sequence + 1,
  //   accountNumber: account_number,
  //   fee: new Fee(1000000, {uusd: 500000}),
  // });
  //
  // console.log('broadcast 1')
  // const result1 = await bombay.tx.broadcastSync(tx);
  // console.log(result1);
  //
  // console.log('broadcast 2')
  // const result2 = await bombay.tx.broadcastSync(tx2);
  // console.log(result2);

  try {
    const tx = await wallet1.createAndSignTx({
      msgs: [feegrant],
    });
   // const res = await wallet1.lcd.tx.broadcastSync(tx);
  } catch(e: any) {
    console.log('data', e?.response?.data)
  }
}

// async function executeEpoch() {
//   const overseer = "terra1v6l6kqypskmel0vlnhuwewrhsppnpddh27afhe";
//   const msg = new MsgExecuteContract(
//     wallet.key.accAddress, // sender
//     overseer, // contract account address
//     {
//       execute_epoch_operations: {}
//     }, // handle msg
//     {} // coins
//   );
//   const tx = await wallet.createAndSignTx({
//     msgs: [msg]
//   });
//   const result = await terra.tx.broadcast(tx);
//   if (isTxError(result)) {
//     throw new Error('msg error: ' + result.code + ' ' + result.raw_log);
//   }
// }

main().catch((e) => console.error(e));
//executeEpoch().catch(e => console.error(e))
