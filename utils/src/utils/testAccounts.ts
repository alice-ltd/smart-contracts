import { MnemonicKey } from '@terra-money/terra.js';
import { bombay } from './terra';

// terra1pcl09xsxjmucljksm4xxx3c9hx5pk5qqd3vhsf
const mk1 = new MnemonicKey({
  mnemonic:
    'gasp setup insect party genre fantasy diet heavy sort name hip term urban library program focus table sure friend assist melody eight rice stock',
});
export const wallet1 = bombay.wallet(mk1);

// terra14t7mukq06ty7p64cr2xvhvkty56jfh7e70qnkg
const mk2 = new MnemonicKey({
  mnemonic:
    'rain stick dress ride fatigue loud wrist address title demise speed fabric manage ensure basic vote enough voice century betray music shuffle witness door',
});
export const wallet2 = bombay.wallet(mk2);
