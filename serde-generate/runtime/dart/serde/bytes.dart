// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
part of serde;

/**
 * Immutable wrapper class around byte[].
 *
 * Enforces value-semantice for `equals` and `hashCode`.
 */
class Bytes {
  Uint8List content;

  Bytes(Uint8List content) : content = content;

  Uint8List getContent() {
    return this.content;
  }

  @override
  bool operator ==(covariant Bytes other) {
    return isUint8ListsEqual(this.content, other.content);
  }

  @override
  int get hashCode => this.content.hashCode;

  Bytes.fromJson(String json) : content = Uint8List.fromList(HEX.decode(json));

  String toJson() => HEX.encode(content);
}
