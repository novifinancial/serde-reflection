import {BinarySerializer} from "../serde/binarySerializer";
import {Uint64LE} from "int64-buffer";

export abstract class BincodeSerializer extends BinarySerializer {
    serializeLen(value: number): void {
        this.serializeU64(new Uint64LE(value));
    }

    public serializeVariantIndex(value: number): void {
        this.serializeU32(value);
    }
}