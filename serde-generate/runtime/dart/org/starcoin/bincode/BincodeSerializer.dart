// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

import 'dart:typed_data';

import '../serde/BinarySerializer.dart';

class BincodeSerializer extends BinarySerializer {
  void serialize_len(int value) {
    serialize_u64(value);
  }

  void serialize_variant_index(int value) {
    serialize_u32(value);
  }

  void sort_map_entries(Int32List offsets) {
    // Not required by the format.
  }
}
