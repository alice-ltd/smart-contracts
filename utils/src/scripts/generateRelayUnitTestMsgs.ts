import { MnemonicKey } from '@terra-money/terra.js';

// for relay tests
// terra1x46rqay4d3cssq8gxxvqz8xt6nwlz4td20k38v

async function main() {
  const mk = new MnemonicKey({
    mnemonic:
      'notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius',
  });
  console.log('public key', mk.publicKey?.toString('hex'));
  console.log('raw address', mk.rawAddress?.toString('hex'));

  {
    console.log();
    let transferMsg = {
      contract: 'terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun',
      chain_id: 'terra-test',
      nonce: '1',
      msg: {
        transfer: {
          recipient: 'terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6',
          amount: '100',
        },
      },
    };
    let transferMsgStr = JSON.stringify(transferMsg);
    console.log(transferMsgStr);
    let depositMsgBuff = Buffer.from(transferMsgStr, 'ascii');
    let signature = await mk.sign(depositMsgBuff);

    console.log('transferMsg signature', signature.toString('hex'));
  }

  {
    console.log();
    let transferMsgWithTip = {
      contract: 'terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun',
      chain_id: 'terra-test',
      nonce: '1',
      msg: {
        transfer: {
          recipient: 'terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6',
          amount: '100',
        },
      },
      tip: '50',
    };
    let transferMsgStr = JSON.stringify(transferMsgWithTip);
    console.log(transferMsgStr);
    let depositMsgBuff = Buffer.from(transferMsgStr, 'ascii');
    let signature = await mk.sign(depositMsgBuff);

    console.log('transferMsgWithTip signature', signature.toString('hex'));
  }

  {
    console.log();
    let transferMsg = {
      contract: 'terra1k82qylhej6lgym9j3w0u4s62pgvyf9c8wypsm7',
      chain_id: 'terra-test',
      nonce: '1',
      msg: {
        transfer: {
          recipient: 'terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6',
          amount: '100',
        },
      },
    };
    let transferMsgStr = JSON.stringify(transferMsg);
    console.log(transferMsgStr);
    let depositMsgBuff = Buffer.from(transferMsgStr, 'ascii');
    let signature = await mk.sign(depositMsgBuff);

    console.log(
      'transferMsgWrongContract signature',
      signature.toString('hex')
    );
  }

  {
    console.log();
    let transferMsg = {
      contract: 'terra1dzhzukyezv0etz22ud940z7adyv7xgcjkahuun',
      chain_id: 'terra-test-5',
      nonce: '1',
      msg: {
        transfer: {
          recipient: 'terra12rusa506gu7f4xaxqucym48arl5q9ltn4ekuw6',
          amount: '100',
        },
      },
    };
    let transferMsgStr = JSON.stringify(transferMsg);
    console.log(transferMsgStr);
    let depositMsgBuff = Buffer.from(transferMsgStr, 'ascii');
    let signature = await mk.sign(depositMsgBuff);

    console.log('transferMsgWrongChain signature', signature.toString('hex'));
  }
}

main().catch((e) => console.error(e));
