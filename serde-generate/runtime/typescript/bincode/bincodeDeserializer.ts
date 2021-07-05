import {BinaryDeserializer} from "../serde/binaryDeserializer";

export class BincodeDeserializer extends BinaryDeserializer {
    deserializeLen(): bigint {
        return this.deserializeU64();
    }

    public deserializeVariantIndex(): number {
        return this.deserializeU32();
    }

    checkThatKeySlicesAreIncreasing(key1: [number, number], key2: [number, number]): void {
        return;
    }

}