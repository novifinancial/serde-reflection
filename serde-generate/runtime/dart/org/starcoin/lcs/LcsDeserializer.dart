// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
import 'dart:typed_data';

import '../serde//BinaryDeserializer.dart';
import '../serde/Slice.dart';

const int maxInt = 4294967296;

class LcsDeserializer extends BinaryDeserializer {
  LcsDeserializer(Uint8List input) : super(input) {}

  int deserialize_uleb128_as_u32() {
    int value = 0;
    for (int shift = 0; shift < 32; shift += 7) {
      int x = super.getUint8();
      int digit = (x & 0x7F);
      value = value | (digit << shift);
      if (value > maxInt) {
        throw new Exception(
            "Overflow while parsing uleb128-encoded uint32 value");
      }
      if (digit == x) {
        if (shift > 0 && digit == 0) {
          throw new Exception("Invalid uleb128 number (unexpected zero digit)");
        }
        return value;
      }
    }
    throw new Exception("Overflow while parsing uleb128-encoded uint32 value");
  }

  int deserialize_len() {
    return deserialize_uleb128_as_u32();
  }

  int deserialize_variant_index() {
    return deserialize_uleb128_as_u32();
  }

  void check_that_key_slices_are_increasing(Slice key1, Slice key2) {
    if (Slice.compare_bytes(input.buffer.asUint8List(), key1, key2) >= 0) {
      throw new Exception(
          "Error while decoding map: keys are not serialized in the expected order");
    }
  }
}
