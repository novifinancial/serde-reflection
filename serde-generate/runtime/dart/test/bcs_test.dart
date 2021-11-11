// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

import 'dart:typed_data';
import 'package:test/test.dart';
import '../lib/src/bcs/bcs.dart';

void main() {
  test('serializeUint32', () {
    BcsSerializer serializer = BcsSerializer();
    serializer.serializeUint32(1);
    expect(serializer.bytes, Uint8List.fromList([1, 0, 0, 0]));
  });

  test('deserializeUint32', () {
    BcsDeserializer serializer =
        BcsDeserializer(Uint8List.fromList([1, 0, 0, 0]));
    int result = serializer.deserializeUint32();
    expect(result, 1);
  });

  test('slice', () {
    BcsSerializer serializer = BcsSerializer();
    serializer.serializeUint8(-1);
    serializer.serializeUint32(1);
    serializer.serializeUint32(1);
    serializer.serializeUint32(2);
    expect(
        serializer.bytes,
        Uint8List.fromList([
          -1,
          /**/ 1,
          /**/ 0,
          0,
          /**/ 0,
          1,
          0,
          /**/ 0,
          /**/ 0,
          /**/ 2,
          0,
          0,
          0
        ]));
  });
}
