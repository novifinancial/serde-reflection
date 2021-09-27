// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
part of serde;

/**
 * Immutable wrapper class around byte[].
 *
 * Enforces value-semantice for `equals` and `hashCode`.
 */
@immutable
class Bytes {
  const Bytes(this.content);

  factory Bytes.fromJson(String json) {
    return Bytes(Uint8List.fromList(HEX.decode(json)));
  }

  final Uint8List content;

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is Bytes && listEquals(content, other.content);
  }

  @override
  int get hashCode => content.hashCode;

  String toJson() => HEX.encode(content);
}
