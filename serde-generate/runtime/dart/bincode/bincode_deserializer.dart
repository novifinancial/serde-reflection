// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
part of bincode;

class BincodeDeserializer extends BinaryDeserializer {
  BincodeDeserializer(Uint8List input) : super(input);

  @override
  int deserializeLength() {
    return deserializeUint64();
  }

  @override
  int deserializeVariantIndex() {
    return deserializeUint32();
  }

  void checkThatKeySlicesAreIncreasing(Slice key1, Slice key2) {
    // Not required by the format.
  }
}
