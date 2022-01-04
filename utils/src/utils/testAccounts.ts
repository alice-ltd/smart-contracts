import { MnemonicKey } from '@terra-money/terra.js';
import { bombay } from './terra';

// terra1u3lex2u3dnz6wvte0wjukqe0rphansptxatv32
const mk1 = new MnemonicKey({
  mnemonic:
    'license hidden roast ensure make topple cloud mad offer crumble defy tuna sweet spray ten gas health about same chunk force about fabric rose',
});
export const wallet1 = bombay.wallet(mk1);

// terra14t7mukq06ty7p64cr2xvhvkty56jfh7e70qnkg
const mk2 = new MnemonicKey({
  mnemonic:
    'rain stick dress ride fatigue loud wrist address title demise speed fabric manage ensure basic vote enough voice century betray music shuffle witness door',
});
export const wallet2 = bombay.wallet(mk2);
