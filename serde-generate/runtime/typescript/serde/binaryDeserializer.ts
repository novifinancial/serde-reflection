import { Deserializer } from './deserializer';

export abstract class BinaryDeserializer implements Deserializer {
  private static readonly BIG_32 = BigInt(32);
  private static readonly BIG_64 = BigInt(64);
  private static readonly textDecoder: TextDecoder = new TextDecoder();
  public buffer: ArrayBuffer;
  public offset: number;

  protected constructor(data: Uint8Array) {
    this.buffer = data;
    this.offset = 0;
  }

  private read(length: number): Uint8Array {
    const bytes = new Uint8Array(
        this.buffer.slice(this.offset, this.offset + length)
    );
    this.offset += length;
    return bytes;
  }

  abstract deserializeLen(): number;

  abstract deserializeVariantIndex(): number;

  abstract checkThatKeySlicesAreIncreasing(
      key1: [number, number],
      key2: [number, number]
  ): void;

  public deserializeStr(): string {
    const value = this.deserializeBytes();
    return BinaryDeserializer.textDecoder.decode(value);
  }

  public deserializeBytes(): Uint8Array {
    const len = this.deserializeLen();
    if (len < 0) {
      throw new Error("Length of a bytes array can't be negative");
    }
    return this.read(len);
  }

  public deserializeBool(): boolean {
    const bool = this.read(1)[0];
    return bool == 1;
  }

  public deserializeUnit(): undefined {
    return;
  }

  public deserializeU8(): number {
    return new DataView(this.read(1)).getUint8(0);
  }

  public deserializeU16(): number {
    return new DataView(this.read(2)).getUint16(0, true);
  }

  public deserializeU32(): number {
    return new DataView(this.read(4)).getUint32(0, true);
  }

  public deserializeU64(): BigInt {
    const left = this.deserializeU32();
    const right = this.deserializeU32();

    // combine the two 32-bit values and return (right contain the high bits)
    return (BigInt(right) << BinaryDeserializer.BIG_32) | BigInt(left);
  }

  public deserializeU128(): BigInt {
    const left = this.deserializeU64();
    const right = this.deserializeU64();

    // combine the two 64-bit values and return (right contain the high bits)
    return (BigInt(right) << BinaryDeserializer.BIG_64) | BigInt(left);
  }

  public deserializeI8(): number {
    return new DataView(this.read(1)).getInt8(0);
  }

  public deserializeI16(): number {
    return new DataView(this.read(2)).getInt16(0, true);
  }

  public deserializeI32(): number {
    return new DataView(this.read(4)).getInt32(0, true);
  }

  public deserializeI64(): BigInt {
    const left = this.deserializeI32();
    const right = this.deserializeI32();

    // combine the two 32-bit values and return (right contain the high bits)
    return (BigInt(right) << BinaryDeserializer.BIG_32) | BigInt(left);
  }

  public deserializeI128(): BigInt {
    const left = this.deserializeI64();
    const right = this.deserializeI64();

    // combine the two 64-bit values and return (right contain the high bits)
    return (BigInt(right) << BinaryDeserializer.BIG_64) | BigInt(left);
  }

  public deserializeOptionTag(): boolean {
    return this.deserializeBool();
  }

  public getBufferOffset(): number {
    return this.offset;
  }

  public deserializeChar(): string {
    throw new Error('Method deserializeChar not implemented.');
  }

  public deserializeF32(): number {
    return new DataView(this.read(4)).getFloat32(0, true);
  }

  public deserializeF64(): number {
    return new DataView(this.read(8)).getFloat64(0, true);
  }
}
