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
    if (other == null) return false;
    return isUint8ListsEqual(this.content, other.content);
  }

  @override
  int get hashCode => this.content.hashCode;

  Bytes.fromJson(Map<String, dynamic> json)
      : content = Uint8List.fromList(List<int>.from(json['content']));

  Map<String, dynamic> toJson() => {
    "content": content,
  };
}

