import { BigNumber } from '@ethersproject/bignumber';
import { Int64LE, Uint64LE } from 'int64-buffer';
import { Deserializer } from './deserializer';
import { Readable } from 'stream';

export abstract class BinaryDeserializer implements Deserializer {
  public static readonly MAX_VALUE = 2147483647;
  public data: Readable;
  private readonly totalLength: number;

  private static makeReadable(data: Uint8Array): Readable {
    const r = new Readable();
    r.push(Buffer.from(data));
    r.push(null);
    return r;
  }

  protected constructor(data: Uint8Array) {
    this.data = BinaryDeserializer.makeReadable(data);
    this.totalLength = this.data.readableLength;
  }

  abstract deserializeLen(): number;

  abstract deserializeVariantIndex(): number;

  abstract checkThatKeySlicesAreIncreasing(
      key1: [number, number],
      key2: [number, number]
  ): void;

  public deserializeStr(): string {
    const value = this.deserializeBytes();
    return String.fromCharCode.apply(null, Array.from(value));
  }

  public deserializeBytes(): Uint8Array {
    const len = this.deserializeLen();
    if (len < 0 || len > BinaryDeserializer.MAX_VALUE) {
      throw new Error('The length of a JavaScript array cannot exceed MAXINT');
    }
    return this.data.read(len);
  }

  public deserializeBool(): boolean {
    const bool = this.data.read(1);
    return bool[0] == 1;
  }

  public deserializeUnit(): any {
    return;
  }

  public deserializeU8(): number {
    return Buffer.from(this.data.read(1)).readUInt8(0);
  }

  public deserializeU16(): number {
    return Buffer.from(this.data.read(2)).readUInt16LE(0);
  }

  public deserializeU32(): number {
    return Buffer.from(this.data.read(4)).readUInt32LE(0);
  }

  public deserializeU64(): Uint64LE {
    return new Uint64LE(Buffer.from(this.data.read(8)));
  }

  public deserializeU128(): BigNumber {
    return BigNumber.from(this.data.read(16));
  }

  public deserializeI8(): number {
    return Buffer.from(this.data.read(1)).readInt8(0);
  }

  public deserializeI16(): number {
    return Buffer.from(this.data.read(2)).readInt16LE(0);
  }

  public deserializeI32(): number {
    return Buffer.from(this.data.read(4)).readInt32LE(0);
  }

  public deserializeI64(): Int64LE {
    return new Int64LE(Buffer.from(this.data.read(8)));
  }

  public deserializeI128(): BigNumber {
    return BigNumber.from(this.data.read(16));
  }

  public deserializeOptionTag(): boolean {
    return this.deserializeBool();
  }

  public getBufferOffset(): number {
    return this.totalLength - this.data.readableLength;
  }

  public deserializeChar(): string {
    throw new Error('Method serializeChar not implemented.');
  }

  public deserializeF32(): number {
    throw new Error('Method serializeF32 not implemented.');
  }

  public deserializeF64(): number {
    throw new Error('Method serializeF64 not implemented.');
  }
}
