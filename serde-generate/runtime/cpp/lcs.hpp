// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#pragma once

#include <algorithm>

#include "serde.hpp"

// Maximum length supported for LCS sequences and maps.
constexpr size_t LCS_MAX_LENGTH = 1ull << 31;

class LcsSerializer {
    std::vector<uint8_t> bytes_;

    void serialize_u32_as_uleb128(uint32_t);

  public:
    LcsSerializer() {}

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

    static constexpr bool enforce_strict_map_ordering = true;
    size_t get_buffer_offset();
    void sort_last_entries(std::vector<size_t> offsets);

    std::vector<uint8_t> bytes() && { return std::move(bytes_); }
};

class LcsDeserializer {
    std::vector<uint8_t> bytes_;
    size_t pos_;

    uint8_t read_byte();
    uint32_t deserialize_uleb128_as_u32();

  public:
    LcsDeserializer(std::vector<uint8_t> bytes) {
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

    static constexpr bool enforce_strict_map_ordering = true;
    size_t get_buffer_offset();
    void check_that_key_slices_are_increasing(std::tuple<size_t, size_t> key1,
                                              std::tuple<size_t, size_t> key2);
};

inline void LcsSerializer::serialize_u32_as_uleb128(uint32_t value) {
    while (value >= 0x80) {
        bytes_.push_back((uint8_t)value & 0x7F);
        value = value >> 7;
    }
    bytes_.push_back((uint8_t)value);
}

inline void LcsSerializer::serialize_str(const std::string &value) {
    serialize_len(value.size());
    for (auto c : value) {
        bytes_.push_back(c);
    }
}

inline void LcsSerializer::serialize_unit() {}

inline void LcsSerializer::serialize_f32(float) { throw "not implemented"; }

inline void LcsSerializer::serialize_f64(double) { throw "not implemented"; }

inline void LcsSerializer::serialize_char(char32_t) { throw "not implemented"; }

inline void LcsSerializer::serialize_bool(bool value) {
    bytes_.push_back((uint8_t)value);
}

inline void LcsSerializer::serialize_u8(uint8_t value) {
    bytes_.push_back(value);
}

inline void LcsSerializer::serialize_u16(uint16_t value) {
    bytes_.push_back((uint8_t)value);
    bytes_.push_back((uint8_t)(value >> 8));
}

inline void LcsSerializer::serialize_u32(uint32_t value) {
    bytes_.push_back((uint8_t)value);
    bytes_.push_back((uint8_t)(value >> 8));
    bytes_.push_back((uint8_t)(value >> 16));
    bytes_.push_back((uint8_t)(value >> 24));
}

inline void LcsSerializer::serialize_u64(uint64_t value) {
    bytes_.push_back((uint8_t)value);
    bytes_.push_back((uint8_t)(value >> 8));
    bytes_.push_back((uint8_t)(value >> 16));
    bytes_.push_back((uint8_t)(value >> 24));
    bytes_.push_back((uint8_t)(value >> 32));
    bytes_.push_back((uint8_t)(value >> 40));
    bytes_.push_back((uint8_t)(value >> 48));
    bytes_.push_back((uint8_t)(value >> 56));
}

inline void LcsSerializer::serialize_u128(const uint128_t &value) {
    serialize_u64(value.low);
    serialize_u64(value.high);
}

inline void LcsSerializer::serialize_i8(int8_t value) {
    serialize_u8((uint8_t)value);
}

inline void LcsSerializer::serialize_i16(int16_t value) {
    serialize_u16((uint16_t)value);
}

inline void LcsSerializer::serialize_i32(int32_t value) {
    serialize_u32((uint32_t)value);
}

inline void LcsSerializer::serialize_i64(int64_t value) {
    serialize_u64((uint64_t)value);
}

inline void LcsSerializer::serialize_i128(const int128_t &value) {
    serialize_u64(value.low);
    serialize_i64(value.high);
}

inline void LcsSerializer::serialize_len(size_t value) {
    if (value > LCS_MAX_LENGTH) {
        throw "Length is too large";
    }
    serialize_u32_as_uleb128((uint32_t)value);
}

inline void LcsSerializer::serialize_variant_index(size_t value) {
    serialize_u32_as_uleb128((uint32_t)value);
}

inline void LcsSerializer::serialize_option_tag(bool value) {
    serialize_bool(value);
}

inline size_t LcsSerializer::get_buffer_offset() { return bytes_.size(); }

inline void LcsSerializer::sort_last_entries(std::vector<size_t> offsets) {
    if (offsets.size() <= 1) {
        return;
    }
    offsets.push_back(bytes_.size());

    std::vector<std::vector<uint8_t>> slices;
    for (auto i = 1; i < offsets.size(); i++) {
        auto start = bytes_.cbegin() + offsets[i - 1];
        auto end = bytes_.cbegin() + offsets[i];
        slices.emplace_back(start, end);
    }

    std::sort(slices.begin(), slices.end(), [](auto &s1, auto &s2) {
        return std::lexicographical_compare(s1.begin(), s1.end(), s2.begin(),
                                            s2.end());
    });

    bytes_.resize(offsets[0]);
    for (auto slice : slices) {
        bytes_.insert(bytes_.end(), slice.begin(), slice.end());
    }
    assert(offsets.back() == bytes_.size());
}

inline uint8_t LcsDeserializer::read_byte() { return bytes_.at(pos_++); }

inline uint32_t LcsDeserializer::deserialize_uleb128_as_u32() {
    uint64_t value = 0;
    for (int shift = 0; shift < 32; shift += 7) {
        auto byte = read_byte();
        auto digit = byte & 0x7F;
        value |= digit << shift;
        if (value > std::numeric_limits<uint32_t>::max()) {
            throw "Overflow while parsing uleb128-encoded uint32 value";
        }
        if (digit == byte) {
            if (shift > 0 && digit == 0) {
                throw "Invalid uleb128 number (unexpected zero digit)";
            }
            return (uint32_t)value;
        }
    }
    throw "Overflow while parsing uleb128-encoded uint32 value";
}

inline std::string LcsDeserializer::deserialize_str() {
    auto len = deserialize_len();
    std::string result;
    result.reserve(len);
    for (size_t i = 0; i < len; i++) {
        result.push_back(read_byte());
    }
    return result;
}

inline void LcsDeserializer::deserialize_unit() {}

inline float LcsDeserializer::deserialize_f32() { throw "not implemented"; }

inline double LcsDeserializer::deserialize_f64() { throw "not implemented"; }

inline char32_t LcsDeserializer::deserialize_char() { throw "not implemented"; }

inline bool LcsDeserializer::deserialize_bool() { return (bool)read_byte(); }

inline uint8_t LcsDeserializer::deserialize_u8() { return read_byte(); }

inline uint16_t LcsDeserializer::deserialize_u16() {
    uint16_t val = 0;
    val |= (uint16_t)read_byte();
    val |= (uint16_t)read_byte() << 8;
    return val;
}

inline uint32_t LcsDeserializer::deserialize_u32() {
    uint32_t val = 0;
    val |= (uint32_t)read_byte();
    val |= (uint32_t)read_byte() << 8;
    val |= (uint32_t)read_byte() << 16;
    val |= (uint32_t)read_byte() << 24;
    return val;
}

inline uint64_t LcsDeserializer::deserialize_u64() {
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

inline uint128_t LcsDeserializer::deserialize_u128() {
    uint128_t result;
    result.low = deserialize_u64();
    result.high = deserialize_u64();
    return result;
}

inline int8_t LcsDeserializer::deserialize_i8() {
    return (int8_t)deserialize_u8();
}

inline int16_t LcsDeserializer::deserialize_i16() {
    return (int16_t)deserialize_u16();
}

inline int32_t LcsDeserializer::deserialize_i32() {
    return (int32_t)deserialize_u32();
}

inline int64_t LcsDeserializer::deserialize_i64() {
    return (int64_t)deserialize_u64();
}

inline int128_t LcsDeserializer::deserialize_i128() {
    int128_t result;
    result.low = deserialize_u64();
    result.high = deserialize_i64();
    return result;
}

inline size_t LcsDeserializer::deserialize_len() {
    auto value = deserialize_uleb128_as_u32();
    if (value > LCS_MAX_LENGTH) {
        throw "Length is too large";
    }
    return (size_t)value;
}

inline size_t LcsDeserializer::deserialize_variant_index() {
    return (size_t)deserialize_uleb128_as_u32();
}

inline bool LcsDeserializer::deserialize_option_tag() {
    return deserialize_bool();
}

inline size_t LcsDeserializer::get_buffer_offset() { return pos_; }

inline void LcsDeserializer::check_that_key_slices_are_increasing(
    std::tuple<size_t, size_t> key1, std::tuple<size_t, size_t> key2) {
    if (!std::lexicographical_compare(bytes_.cbegin() + std::get<0>(key1),
                                      bytes_.cbegin() + std::get<1>(key1),
                                      bytes_.cbegin() + std::get<0>(key2),
                                      bytes_.cbegin() + std::get<1>(key2))) {
        throw "Error while decoding map: keys are not serialized in the "
              "expected order";
    }
}
