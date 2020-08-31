// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0
part of serde;

class Unit {
  Unit() {}

  @override
  bool operator ==(covariant Unit other) {
    if (this == other) return true;
    if (other == null) return false;
    return true;
  }

  @override
  int get hashCode => 7;
}
