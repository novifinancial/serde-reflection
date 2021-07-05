import {BinarySerializer} from "../serde/binarySerializer";

export class BincodeSerializer extends BinarySerializer {
    serializeLen(value: bigint): void {
        this.serializeU64(value);
    }

    public serializeVariantIndex(value: number): void {
        this.serializeU32(value);
    }

    public sortMapEntries(offsets: number[]): void {
        return;
    }
}