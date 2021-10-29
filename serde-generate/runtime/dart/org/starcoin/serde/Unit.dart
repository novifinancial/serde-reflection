// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

class Unit {
  Unit() {}

  @override
  bool operator ==(covariant Unit other) {
    if (other == null) return false;
    return true;
  }

  @override
  int get hashCode => 7;
}
