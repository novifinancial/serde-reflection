import leb from 'leb';
import {BinarySerializer} from '../serde/binarySerializer';

export class LcsSerializer extends BinarySerializer {
    public serializeU32AsUleb128(value: number): void {
        this.concat(leb.encodeUInt32(value));
    }

    serializeLen(value: number): void {
        this.serializeU32AsUleb128(value);
    }

    public serializeVariantIndex(value: number): void {
        this.serializeU32AsUleb128(value);
    }

    public sortMapEntries(offsets: number[]) {
        // leaving it empty for now, should be implemented soon
    }

    public static hexString(value: string): Uint8Array {
        const data = value.match(/.{1,2}/g)!.map((x) => parseInt(x, 16));
        return new Uint8Array(data);
    }

    public getBytesAsHex(): string {
        return Buffer.from(this.getBytes()).toString('hex');
    }


}
