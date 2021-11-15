// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

part of bcs;

class BcsSerializer extends BinarySerializer {
  void serializeUint32AsUleb128(int value) {
    while (((value & 0xFFFFFFFF) >> 7) != 0) {
      output.add((value & 0x7f) | 0x80);
      value = (value & 0xFFFFFFFF) >> 7;
    }
    output.add(value);
  }

  @override
  void serializeLength(int value) {
    serializeUint32AsUleb128(value);
  }

  @override
  void serializeVariantIndex(int value) {
    serializeUint32AsUleb128(value);
  }

  void sortMapEntries(Uint8List offsets) {
    // TODO(#120)
  }
}
