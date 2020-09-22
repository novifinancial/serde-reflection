import {BigNumber} from '@ethersproject/bignumber';
import {Int64LE, Uint64LE} from 'int64-buffer';
import {Serializer} from './serializer';
import bytes from '@ethersproject/bytes';

export abstract class BinarySerializer implements Serializer {
    private output: Uint8Array = Buffer.alloc(0);

    abstract serializeLen(value: number): void ;

    abstract serializeVariantIndex(value: number): void;

    abstract sortMapEntries(offsets: number[]): void;

    public serializeStr(value: string): void {
        const stringUTF8Bytes = Buffer.from(value, 'utf8');
        this.serializeBytes(stringUTF8Bytes);
    }

    public serializeBytes(value: Uint8Array): void {
        this.serializeLen(value.length);
        this.concat(value);
    }

    public serializeBool(value: boolean): void {
        this.concat(Buffer.from([value ? 1 : 0]));
    }

    public serializeUnit(value: any): void {
    }

    public serializeU8(value: number): void {
        this.concat(new Uint8Array([value]));
    }

    public serializeU16(value: number): void {
        const u16 = new Uint16Array([value]);
        this.concat(Buffer.from(u16.buffer));
    }

    public serializeU32(value: number): void {
        const u32 = new Uint32Array([value]);
        this.concat(Buffer.from(u32.buffer));
    }

    public serializeU64(value: Uint64LE): void {
        const buffer = value.toArrayBuffer();
        this.concat(new Uint8Array(buffer));
    }

    public serializeU128(value: BigNumber): void {
        this.concat(bytes.arrayify(value));
    }

    public serializeI8(value: number): void {
        this.serializeU8(value);
    }

    public serializeI16(value: number): void {
        this.serializeU16(value);
    }

    public serializeI32(value: number): void {
        this.serializeU32(value);
    }

    public serializeI64(value: Int64LE): void {
        const buffer = value.toArrayBuffer();
        this.concat(new Uint8Array(buffer));
    }

    public serializeI128(value: BigNumber): void {
        this.concat(bytes.arrayify(value));
    }

    public serializeOptionTag(value: boolean): void {
        if (value) {
            this.concat(Buffer.from([1])); // True
        } else {
            this.concat(Buffer.from([0])); // False
        }
    }

    public getBufferOffset(): number {
        return this.output.length;
    }

    public getBytes(): Uint8Array {
        return this.output;
    }

    public concat(value: Uint8Array): void {
        this.output = BinarySerializer.concat(this.output, value);
    }

    public static concat(a: Uint8Array, b: Uint8Array): Uint8Array {
        return Buffer.concat([a, b], a.length + b.length);
    }

    public serializeChar(value: string): void {
        throw new Error('Method serializeChar not implemented.');
    }

    public serializeF32(value: number): void {
        throw new Error('Method serializeF32 not implemented.');
    }

    public serializeF64(value: number): void {
        throw new Error('Method serializeF64 not implemented.');
    }
}
