// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#pragma once

#include <array>
#include <cstdint>
#include <functional>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <tuple>
#include <type_traits>
#include <variant>
#include <vector>

// Placeholder types for 128-bit integers.
using uint128_t = std::tuple<uint64_t, uint64_t>;
using int128_t = std::tuple<int64_t, uint64_t>;

// Trait to enable serialization of values of type T.
// This is similar to the `serde::Serialize` trait in Rust.
template <typename T>
struct Serializable {
    template <typename Serializer>
    static void serialize(const T &value, Serializer &serializer);
};

// Trait to enable deserialization of values of type T.
// This is similar to the `serde::Deserialize` trait in Rust.
template <typename T>
struct Deserializable {
    template <typename Deserializer>
    static T deserialize(Deserializer &deserializer);
};

