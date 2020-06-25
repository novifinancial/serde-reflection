// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#pragma once

#include "serde.hpp"

namespace serde {

class BincodeSerializer {
    std::vector<uint8_t> bytes_;

  public:
    BincodeSerializer() {}

    void serialize_str(const std::string &value);

    void serialize_bool(bool value);
    void serialize_unit();
    void serialize_char(char32_t value);
    void serialize_f32(float value);
    void serialize_f64(double value);

    void serialize_u8(uint8_t value);
    void serialize_u16(uint16_t value);
    void serialize_u32(uint32_t value);
    void serialize_u64(uint64_t value);
    void serialize_u128(const uint128_t &value);

    void serialize_i8(int8_t value);
    void serialize_i16(int16_t value);
    void serialize_i32(int32_t value);
    void serialize_i64(int64_t value);
    void serialize_i128(const int128_t &value);

    void serialize_len(size_t value);
    void serialize_variant_index(size_t value);
    void serialize_option_tag(bool value);

    static constexpr bool enforce_strict_map_ordering = false;

    std::vector<uint8_t> bytes() && { return std::move(bytes_); }
};

class BincodeDeserializer {
    std::vector<uint8_t> bytes_;
    size_t pos_;

    uint8_t read_byte();

  public:
    BincodeDeserializer(std::vector<uint8_t> bytes) {
        bytes_ = std::move(bytes);
        pos_ = 0;
    }

    std::string deserialize_str();

    bool deserialize_bool();
    void deserialize_unit();
    char32_t deserialize_char();
    float deserialize_f32();
    double deserialize_f64();

    uint8_t deserialize_u8();
    uint16_t deserialize_u16();
    uint32_t deserialize_u32();
    uint64_t deserialize_u64();
    uint128_t deserialize_u128();

    int8_t deserialize_i8();
    int16_t deserialize_i16();
    int32_t deserialize_i32();
    int64_t deserialize_i64();
    int128_t deserialize_i128();

    size_t deserialize_len();
    size_t deserialize_variant_index();
    bool deserialize_option_tag();

    static constexpr bool enforce_strict_map_ordering = false;
};

inline void BincodeSerializer::serialize_str(const std::string &value) {
    serialize_len(value.size());
    for (auto c : value) {
        bytes_.push_back(c);
    }
}

inline void BincodeSerializer::serialize_unit() {}

inline void BincodeSerializer::serialize_f32(float) { throw "not implemented"; }

inline void BincodeSerializer::serialize_f64(double) {
    throw "not implemented";
}

inline void BincodeSerializer::serialize_char(char32_t) {
    throw "not implemented";
}

inline void BincodeSerializer::serialize_bool(bool value) {
    bytes_.push_back((uint8_t)value);
}

inline void BincodeSerializer::serialize_u8(uint8_t value) {
    bytes_.push_back(value);
}

inline void BincodeSerializer::serialize_u16(uint16_t value) {
    bytes_.push_back((uint8_t)value);
    bytes_.push_back((uint8_t)(value >> 8));
}

inline void BincodeSerializer::serialize_u32(uint32_t value) {
    bytes_.push_back((uint8_t)value);
    bytes_.push_back((uint8_t)(value >> 8));
    bytes_.push_back((uint8_t)(value >> 16));
    bytes_.push_back((uint8_t)(value >> 24));
}

inline void BincodeSerializer::serialize_u64(uint64_t value) {
    bytes_.push_back((uint8_t)value);
    bytes_.push_back((uint8_t)(value >> 8));
    bytes_.push_back((uint8_t)(value >> 16));
    bytes_.push_back((uint8_t)(value >> 24));
    bytes_.push_back((uint8_t)(value >> 32));
    bytes_.push_back((uint8_t)(value >> 40));
    bytes_.push_back((uint8_t)(value >> 48));
    bytes_.push_back((uint8_t)(value >> 56));
}

inline void BincodeSerializer::serialize_u128(const uint128_t &value) {
    serialize_u64(value.low);
    serialize_u64(value.high);
}

inline void BincodeSerializer::serialize_i8(int8_t value) {
    serialize_u8((uint8_t)value);
}

inline void BincodeSerializer::serialize_i16(int16_t value) {
    serialize_u16((uint16_t)value);
}

inline void BincodeSerializer::serialize_i32(int32_t value) {
    serialize_u32((uint32_t)value);
}

inline void BincodeSerializer::serialize_i64(int64_t value) {
    serialize_u64((uint64_t)value);
}

inline void BincodeSerializer::serialize_i128(const int128_t &value) {
    serialize_u64(value.low);
    serialize_i64(value.high);
}

inline void BincodeSerializer::serialize_len(size_t value) {
    serialize_u64((uint64_t)value);
}

inline void BincodeSerializer::serialize_variant_index(size_t value) {
    serialize_u32((uint32_t)value);
}

inline void BincodeSerializer::serialize_option_tag(bool value) {
    serialize_bool(value);
}

inline std::string BincodeDeserializer::deserialize_str() {
    auto len = deserialize_len();
    std::string result;
    result.reserve(len);
    for (size_t i = 0; i < len; i++) {
        result.push_back(read_byte());
    }
    return result;
}

inline uint8_t BincodeDeserializer::read_byte() { return bytes_.at(pos_++); }

inline void BincodeDeserializer::deserialize_unit() {}

inline float BincodeDeserializer::deserialize_f32() { throw "not implemented"; }

inline double BincodeDeserializer::deserialize_f64() {
    throw "not implemented";
}

inline char32_t BincodeDeserializer::deserialize_char() {
    throw "not implemented";
}

inline bool BincodeDeserializer::deserialize_bool() {
    return (bool)read_byte();
}

inline uint8_t BincodeDeserializer::deserialize_u8() { return read_byte(); }

inline uint16_t BincodeDeserializer::deserialize_u16() {
    uint16_t val = 0;
    val |= (uint16_t)read_byte();
    val |= (uint16_t)read_byte() << 8;
    return val;
}

inline uint32_t BincodeDeserializer::deserialize_u32() {
    uint32_t val = 0;
    val |= (uint32_t)read_byte();
    val |= (uint32_t)read_byte() << 8;
    val |= (uint32_t)read_byte() << 16;
    val |= (uint32_t)read_byte() << 24;
    return val;
}

inline uint64_t BincodeDeserializer::deserialize_u64() {
    uint64_t val = 0;
    val |= (uint64_t)read_byte();
    val |= (uint64_t)read_byte() << 8;
    val |= (uint64_t)read_byte() << 16;
    val |= (uint64_t)read_byte() << 24;
    val |= (uint64_t)read_byte() << 32;
    val |= (uint64_t)read_byte() << 40;
    val |= (uint64_t)read_byte() << 48;
    val |= (uint64_t)read_byte() << 56;
    return val;
}

inline uint128_t BincodeDeserializer::deserialize_u128() {
    uint128_t result;
    result.low = deserialize_u64();
    result.high = deserialize_u64();
    return result;
}

inline int8_t BincodeDeserializer::deserialize_i8() {
    return (int8_t)deserialize_u8();
}

inline int16_t BincodeDeserializer::deserialize_i16() {
    return (int16_t)deserialize_u16();
}

inline int32_t BincodeDeserializer::deserialize_i32() {
    return (int32_t)deserialize_u32();
}

inline int64_t BincodeDeserializer::deserialize_i64() {
    return (int64_t)deserialize_u64();
}

inline int128_t BincodeDeserializer::deserialize_i128() {
    int128_t result;
    result.low = deserialize_u64();
    result.high = deserialize_i64();
    return result;
}

inline size_t BincodeDeserializer::deserialize_len() {
    return (size_t)deserialize_u64();
}

inline size_t BincodeDeserializer::deserialize_variant_index() {
    return (size_t)deserialize_u32();
}

inline bool BincodeDeserializer::deserialize_option_tag() {
    return deserialize_bool();
}

} // end of namespace serde
