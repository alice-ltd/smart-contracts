import {
  createDepositStableMsg,
  deployContract,
  uploadCode,
} from '../utils/contract';
import { bombay } from '../utils/terra';
import { isTxError, ModeInfo, MsgExecuteContract } from '@terra-money/terra.js';
import { wallet1 } from '../utils/testAccounts';

const TESTNET_MONEY_MARKET_ADDR =
  'terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal';
const TESTNET_ATERRA_TOKEN_ADDR =
  'terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl';

async function main() {
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

  const contractAddr = 'terra10zugkw7zu6t6l8cpkua2ujh3cmmet9nc5gak4p';

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

  const redeem = new MsgExecuteContract(
    'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf', // sender
    contractAddr, // contract account address
    {
      redeem_stable: {
        recipient: 'terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf',
        burn_amount: '86784044',
      },
    }, // handle msg
    {} // coins
  );

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

  const tx = await wallet1.createAndSignTx({
    msgs: [redeem],
    // fee: new Fee(1000000, {[Denom.USD]: 1000000}),
  });

  console.log(JSON.stringify(tx.toData(), null, 2));
  const result = await bombay.tx.broadcast(tx);
  if (isTxError(result)) {
    throw new Error('msg error: ' + result.code + ' ' + result.raw_log);
  }
  console.log(result);
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
