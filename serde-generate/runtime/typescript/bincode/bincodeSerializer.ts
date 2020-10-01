import {BinarySerializer} from "../serde/binarySerializer";

export abstract class BincodeSerializer extends BinarySerializer {
    serializeLen(value: BigInt): void {
        this.serializeU64(value);
    }

    public serializeVariantIndex(value: number): void {
        this.serializeU32(value);
    }
}