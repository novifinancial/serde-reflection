// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#pragma once

#include "serde.hpp"
#include "binary.hpp"

// Maximum length supported in practice (e.g. Java).
constexpr size_t BINCODE_MAX_LENGTH = (1ull << 31) - 1;

namespace serde {

class BincodeSerializer : public BinarySerializer<BincodeSerializer> {
    using Parent = BinarySerializer<BincodeSerializer>;

  public:
    BincodeSerializer() : Parent() {}

    void serialize_len(size_t value);
    void serialize_variant_index(uint32_t value);

    static constexpr bool enforce_strict_map_ordering = false;
};

class BincodeDeserializer : public BinaryDeserializer<BincodeDeserializer> {
    using Parent = BinaryDeserializer<BincodeDeserializer>;

  public:
    BincodeDeserializer(std::vector<uint8_t> bytes)
        : Parent(std::move(bytes)) {}

    size_t deserialize_len();
    uint32_t deserialize_variant_index();

    static constexpr bool enforce_strict_map_ordering = false;
};

inline void BincodeSerializer::serialize_len(size_t value) {
    if (value > BINCODE_MAX_LENGTH) {
        throw serde::serialization_error("Length is too large");
    }
    Parent::serialize_u64((uint64_t)value);
}

inline void BincodeSerializer::serialize_variant_index(uint32_t value) {
    Parent::serialize_u32((uint32_t)value);
}

inline size_t BincodeDeserializer::deserialize_len() {
    auto value = (size_t)Parent::deserialize_u64();
    if (value > BINCODE_MAX_LENGTH) {
        throw serde::deserialization_error("Length is too large");
    }
    return (size_t)value;
}

inline uint32_t BincodeDeserializer::deserialize_variant_index() {
    return Parent::deserialize_u32();
}

} // end of namespace serde
