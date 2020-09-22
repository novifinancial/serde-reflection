import { Int64LE, Uint64LE } from 'int64-buffer';
import { BigNumber } from '@ethersproject/bignumber';

export interface Serializer {
  serializeStr(value: string): void;

  serializeBytes(value: Uint8Array): void;

  serializeBool(value: boolean): void;

  serializeUnit(value: any): void;

  serializeChar(value: string): void;

  serializeF32(value: number): void;

  serializeF64(value: number): void;

  serializeU8(value: number): void;

  serializeU16(value: number): void;

  serializeU32(value: number): void;

  serializeU64(value: Uint64LE): void;

  serializeU128(value: BigNumber): void;

  serializeI8(value: number): void;

  serializeI16(value: number): void;

  serializeI32(value: number): void;

  serializeI64(value: Int64LE): void;

  serializeI128(value: BigNumber): void;

  serializeLen(value: number): void;

  serializeVariantIndex(value: number): void;

  serializeOptionTag(value: boolean): void;

  concat(value: Uint8Array): void;

  getBufferOffset(): number;

  getBytes(): Uint8Array;

  sortMapEntries(offsets: number[]): void;
}
