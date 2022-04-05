import { MnemonicKey } from '@terra-money/terra.js';
import { bombay } from './terra';

const mk1 = new MnemonicKey({
  mnemonic: process.env.TEST_MNEMONIC_1
});
export const wallet1 = bombay.wallet(mk1);

const mk2 = new MnemonicKey({
  mnemonic: process.env.TEST_MNEMONIC_2,
});
export const wallet2 = bombay.wallet(mk2);
