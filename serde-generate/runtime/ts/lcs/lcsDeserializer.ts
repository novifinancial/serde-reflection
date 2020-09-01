import leb from 'leb';
import { BinaryDeserializer } from '../serde/binaryDeserializer';

export class LcsDeserializer extends BinaryDeserializer {
  constructor(data: Buffer) {
    super(data);
  }

  public deserializeUleb128AsU32(): number {
    const buffer: number[] = [];
    let byte = 0xff;
    while (byte >= 0x80) {
      byte = this.deserializeU8();
      buffer.push(byte);
    }

    return leb.decodeUInt32(Buffer.from(buffer)).value;
  }

  deserializeLen(): number {
    return this.deserializeUleb128AsU32();
  }

  public deserializeVariantIndex(): number {
    return this.deserializeUleb128AsU32();
  }

  public deserializeToHexString(): string {
    const bytes = this.deserializeBytes();
    return Buffer.from(bytes).toString('hex');
  }

  public checkThatKeySlicesAreIncreasing(key1:[number, number], key2:[number, number]): void {

  }
}
