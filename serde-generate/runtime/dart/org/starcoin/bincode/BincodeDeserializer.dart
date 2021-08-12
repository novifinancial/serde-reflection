// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

import '../serde/BinaryDeserializer.dart';
import '../serde/Slice.dart';
import 'dart:typed_data';

class BincodeDeserializer extends BinaryDeserializer {
  BincodeDeserializer(Uint8List input) : super(input) {}

  int deserialize_len() {
    return deserialize_u64();
  }

  int deserialize_variant_index() {
    return deserialize_u64();
  }

  void check_that_key_slices_are_increasing(Slice key1, Slice key2) {
    // Not required by the format.
  }
}
