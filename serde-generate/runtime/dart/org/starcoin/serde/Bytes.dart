// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

import 'dart:typed_data';

/**
 * Immutable wrapper class around byte[].
 *
 * Enforces value-semantice for `equals` and `hashCode`.
 */
class Bytes {
  Uint8List content;

  Bytes(Uint8List content) {
    assert(content != null);
    this.content = content;
  }

  Uint8List getContent() {
    return this.content;
  }

  @override
  bool operator ==(covariant Bytes other) {
    if (this == other) return true;
    if (other == null) return false;
    return this.content == other.content;
  }

  @override
  int get hashCode => this.content.hashCode;
}
