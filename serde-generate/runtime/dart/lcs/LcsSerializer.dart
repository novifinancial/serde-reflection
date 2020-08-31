// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
part of lcs;

class LcsSerializer extends BinarySerializer {
  void serialize_u32_as_uleb128(int value) {
    while (((value & 0xFFFFFFFF) >> 7) != 0) {
      output.add((value & 0x7f) | 0x80);
      value = (value & 0xFFFFFFFF) >> 7;
    }
    output.add(value);
  }

  void serialize_len(int value) {
    serialize_u32_as_uleb128(value);
  }

  void serialize_variant_index(int value) {
    serialize_u32_as_uleb128(value);
  }

  void sort_map_entries(Uint8List offsets) {}
}
