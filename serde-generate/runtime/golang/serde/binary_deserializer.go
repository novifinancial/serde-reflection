// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde

import (
	"bytes"
	"errors"
	"fmt"
)

// `BinaryDeserializer` is a partial implementation of the `Deserializer` interface.
// It is used as an embedded struct by the Bincode and LCS deserializers.
type BinaryDeserializer struct {
	Buffer *bytes.Buffer
	Input  []byte
}

func NewBinaryDeserializer(input []byte) *BinaryDeserializer {
	return &BinaryDeserializer{
		Buffer: bytes.NewBuffer(input),
		Input:  input,
	}
}

// `deserializeLen` to be provided by the extending struct.
func (d *BinaryDeserializer) DeserializeBytes(deserializeLen func() (uint64, error)) ([]byte, error) {
	len, err := deserializeLen()
	if err != nil {
		return nil, err
	}
	ret := make([]byte, len)
	_, err = d.Buffer.Read(ret)
	return ret, err
}

// `deserializeLen` to be provided by the extending struct.
func (d *BinaryDeserializer) DeserializeStr(deserializeLen func() (uint64, error)) (string, error) {
	bytes, err := d.DeserializeBytes(deserializeLen)
	return string(bytes), err
}

func (d *BinaryDeserializer) DeserializeBool() (bool, error) {
	ret, err := d.Buffer.ReadByte()
	if err != nil {
		return false, err
	}
	switch ret {
	case 0:
		return false, nil
	case 1:
		return true, nil
	default:
		return false, fmt.Errorf("invalid bool byte: expected 0 / 1, but got %d", ret)
	}
}

func (d *BinaryDeserializer) DeserializeUnit() (struct{}, error) {
	return struct{}{}, nil
}

// DeserializeChar is unimplemented.
func (d *BinaryDeserializer) DeserializeChar() (rune, error) {
	return 0, errors.New("unimplemented")
}

// DeserializeF32 is unimplemented.
func (d *BinaryDeserializer) DeserializeF32() (float32, error) {
	return 0, errors.New("unimplemented")
}

// DeserializeF64 is unimplemented.
func (d *BinaryDeserializer) DeserializeF64() (float64, error) {
	return 0, errors.New("unimplemented")
}

func (d *BinaryDeserializer) DeserializeU8() (uint8, error) {
	ret, err := d.Buffer.ReadByte()
	return uint8(ret), err
}

func (d *BinaryDeserializer) DeserializeU16() (uint16, error) {
	var ret uint16
	for i := 0; i < 8*2; i += 8 {
		b, err := d.Buffer.ReadByte()
		if err != nil {
			return 0, err
		}
		ret = ret | uint16(b)<<i
	}
	return ret, nil
}

func (d *BinaryDeserializer) DeserializeU32() (uint32, error) {
	var ret uint32
	for i := 0; i < 8*4; i += 8 {
		b, err := d.Buffer.ReadByte()
		if err != nil {
			return 0, err
		}
		ret = ret | uint32(b)<<i
	}
	return ret, nil
}

func (d *BinaryDeserializer) DeserializeU64() (uint64, error) {
	var ret uint64
	for i := 0; i < 8*8; i += 8 {
		b, err := d.Buffer.ReadByte()
		if err != nil {
			return 0, err
		}
		ret = ret | uint64(b)<<i
	}
	return ret, nil
}

func (d *BinaryDeserializer) DeserializeU128() (Uint128, error) {
	low, err := d.DeserializeU64()
	if err != nil {
		return Uint128{}, err
	}
	high, err := d.DeserializeU64()
	if err != nil {
		return Uint128{}, err
	}
	return Uint128{High: high, Low: low}, nil
}

func (d *BinaryDeserializer) DeserializeI8() (int8, error) {
	ret, err := d.DeserializeU8()
	return int8(ret), err
}

func (d *BinaryDeserializer) DeserializeI16() (int16, error) {
	ret, err := d.DeserializeU16()
	return int16(ret), err
}

func (d *BinaryDeserializer) DeserializeI32() (int32, error) {
	ret, err := d.DeserializeU32()
	return int32(ret), err
}

func (d *BinaryDeserializer) DeserializeI64() (int64, error) {
	ret, err := d.DeserializeU64()
	return int64(ret), err
}

func (d *BinaryDeserializer) DeserializeI128() (Int128, error) {
	low, err := d.DeserializeU64()
	if err != nil {
		return Int128{}, err
	}
	high, err := d.DeserializeI64()
	if err != nil {
		return Int128{}, err
	}
	return Int128{High: high, Low: low}, nil
}

func (d *BinaryDeserializer) DeserializeOptionTag() (bool, error) {
	return d.DeserializeBool()
}

func (d *BinaryDeserializer) GetBufferOffset() uint64 {
	return uint64(len(d.Input)) - uint64(d.Buffer.Len())
}
