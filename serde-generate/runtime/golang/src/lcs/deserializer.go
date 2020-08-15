// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package lcs

import (
	"bytes"
	"errors"

	"github.com/facebookincubator/serde-reflection/serde"
)

// MaxSequenceLength is max length allowed for sequence
const MaxSequenceLength = (1 << 31) - 1

const maxUint32 = uint64(^uint32(0))

// Deserializer implements `serde.Deserializer` interface for deserializing LCS serialized bytes
type Deserializer struct {
	buf *bytes.Buffer
}

// NewDeserializer creates a new `serde.Deserializer`
func NewDeserializer(input []byte) serde.Deserializer {
	return &Deserializer{buf: bytes.NewBuffer(input)}
}

func (d *Deserializer) DeserializeBytes() ([]byte, error) {
	len, err := d.DeserializeLen()
	if err != nil {
		return nil, err
	}
	ret := make([]byte, len)
	_, err = d.buf.Read(ret)
	return ret, err
}

func (d *Deserializer) DeserializeLen() (int, error) {
	ret, err := d.deserializeUleb128AsU32()
	if ret > MaxSequenceLength {
		return 0, errors.New("length is too large")
	}
	return int(ret), err
}

func (d *Deserializer) DeserializeStr() (string, error) {
	bytes, err := d.DeserializeBytes()
	return string(bytes), err
}

func (d *Deserializer) DeserializeBool() (bool, error) {
	ret, err := d.buf.ReadByte()
	return ret == 1, err
}

func (d *Deserializer) DeserializeUnit() (struct{}, error) {
	return struct{}{}, nil
}

// DeserializeChar is unimplemented
func (d *Deserializer) DeserializeChar() (rune, error) {
	panic("unimplemented")
}

// DeserializeF32 is unimplemented
func (d *Deserializer) DeserializeF32() (float32, error) {
	panic("unimplemented")
}

// DeserializeF64 is unimplemented
func (d *Deserializer) DeserializeF64() (float64, error) {
	panic("unimplemented")
}

func (d *Deserializer) DeserializeU8() (uint8, error) {
	ret, err := d.buf.ReadByte()
	return uint8(ret), err
}

func (d *Deserializer) DeserializeU16() (uint16, error) {
	var ret uint16
	for i := 0; i < 8*2; i += 8 {
		b, err := d.buf.ReadByte()
		if err != nil {
			return 0, err
		}
		ret = ret | uint16(b)<<i
	}
	return ret, nil
}

func (d *Deserializer) DeserializeU32() (uint32, error) {
	var ret uint32
	for i := 0; i < 8*4; i += 8 {
		b, err := d.buf.ReadByte()
		if err != nil {
			return 0, err
		}
		ret = ret | uint32(b)<<i
	}
	return ret, nil
}

func (d *Deserializer) DeserializeU64() (uint64, error) {
	var ret uint64
	for i := 0; i < 8*8; i += 8 {
		b, err := d.buf.ReadByte()
		if err != nil {
			return 0, err
		}
		ret = ret | uint64(b)<<i
	}
	return ret, nil
}

func (d *Deserializer) DeserializeU128() (serde.Uint128, error) {
	low, err := d.DeserializeU64()
	if err != nil {
		return serde.Uint128{}, err
	}
	high, err := d.DeserializeU64()
	if err != nil {
		return serde.Uint128{}, err
	}
	return serde.Uint128{High: high, Low: low}, nil
}

func (d *Deserializer) DeserializeI8() (int8, error) {
	ret, err := d.DeserializeU8()
	return int8(ret), err
}

func (d *Deserializer) DeserializeI16() (int16, error) {
	ret, err := d.DeserializeU16()
	return int16(ret), err
}

func (d *Deserializer) DeserializeI32() (int32, error) {
	ret, err := d.DeserializeU32()
	return int32(ret), err
}

func (d *Deserializer) DeserializeI64() (int64, error) {
	ret, err := d.DeserializeU64()
	return int64(ret), err
}

func (d *Deserializer) DeserializeI128() (serde.Int128, error) {
	low, err := d.DeserializeU64()
	if err != nil {
		return serde.Int128{}, err
	}
	high, err := d.DeserializeI64()
	if err != nil {
		return serde.Int128{}, err
	}
	return serde.Int128{High: high, Low: low}, nil
}

func (d *Deserializer) DeserializeVariantIndex() (uint32, error) {
	return d.deserializeUleb128AsU32()
}

func (d *Deserializer) DeserializeOptionTag() (bool, error) {
	return d.DeserializeBool()
}

func (d *Deserializer) GetBufferOffset() int {
	return d.buf.Len()
}

// CheckThatKeySlicesAreIncreasing is unimplemented yet
func (d *Deserializer) CheckThatKeySlicesAreIncreasing(key1, key2 serde.Slice) error {
	panic("unimplemented")
}

func (d *Deserializer) deserializeUleb128AsU32() (uint32, error) {
	value := uint64(0)
	for shift := 0; shift < 32; shift += 7 {
		byte, err := d.buf.ReadByte()
		if err != nil {
			return 0, err
		}
		digit := byte & 0x7F
		value = value | (uint64(digit) << shift)

		if value > maxUint32 {
			return 0, errors.New("overflow while parsing uleb128-encoded uint32 value")
		}
		if digit == byte {
			if shift > 0 && digit == 0 {
				return 0, errors.New("invalid uleb128 number (unexpected zero digit)")
			}
			return uint32(value), nil
		}
	}
	return 0, errors.New("overflow while parsing uleb128-encoded uint32 value")
}
