import { Int64LE, Uint64LE } from 'int64-buffer';
import { BigNumber } from '@ethersproject/bignumber';

export interface Deserializer {
  deserializeStr(): string;

  deserializeToHexString(): string;

  deserializeBytes(): Uint8Array;

  deserializeBool(): boolean;

  deserializeUnit(): any;

  deserializeChar(): string;

  deserializeF32(): number;

  deserializeF64(): number;

  deserializeU8(): number;

  deserializeU16(): number;

  deserializeU32(): number;

  deserializeU64(): Uint64LE;

  deserializeU128(): BigNumber;

  deserializeI8(): number;

  deserializeI16(): number;

  deserializeI32(): number;

  deserializeI64(): Int64LE;

  deserializeI128(): BigNumber;

  deserializeLen(): number;

  deserializeVariantIndex(): number;

  deserializeOptionTag(): boolean;

  getBufferOffset(): number;

  checkThatKeySlicesAreIncreasing(key1:[number, number], key2:[number, number]): void;
}
