// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde

import (
	"bytes"
	"errors"
)

// `BinarySerializer` is a partial implementation of the `Serializer` interface.
// It is used as an embedded struct by the Bincode and LCS serializers.
type BinarySerializer struct {
	Buffer bytes.Buffer
}

func NewBinarySerializer() *BinarySerializer {
	return new(BinarySerializer)
}

// `serializeLen` to be provided by the extending struct.
func (s *BinarySerializer) SerializeBytes(value []byte, serializeLen func(uint64) error) error {
	serializeLen(uint64(len(value)))
	s.Buffer.Write(value)
	return nil
}

// `serializeLen` to be provided by the extending struct.
func (s *BinarySerializer) SerializeStr(value string, serializeLen func(uint64) error) error {
	return s.SerializeBytes([]byte(value), serializeLen)
}

func (s *BinarySerializer) SerializeBool(value bool) error {
	if value {
		return s.Buffer.WriteByte(1)
	}
	return s.Buffer.WriteByte(0)
}

func (s *BinarySerializer) SerializeUnit(value struct{}) error {
	return nil
}

// SerializeChar is unimplemented.
func (s *BinarySerializer) SerializeChar(value rune) error {
	return errors.New("unimplemented")
}

// SerializeF32 is unimplemented
func (s *BinarySerializer) SerializeF32(value float32) error {
	return errors.New("unimplemented")
}

// SerializeF64 is unimplemented
func (s *BinarySerializer) SerializeF64(value float64) error {
	return errors.New("unimplemented")
}

func (s *BinarySerializer) SerializeU8(value uint8) error {
	s.Buffer.WriteByte(byte(value))
	return nil
}

func (s *BinarySerializer) SerializeU16(value uint16) error {
	s.Buffer.WriteByte(byte(value))
	s.Buffer.WriteByte(byte(value >> 8))
	return nil
}

func (s *BinarySerializer) SerializeU32(value uint32) error {
	s.Buffer.WriteByte(byte(value))
	s.Buffer.WriteByte(byte(value >> 8))
	s.Buffer.WriteByte(byte(value >> 16))
	s.Buffer.WriteByte(byte(value >> 24))
	return nil
}

func (s *BinarySerializer) SerializeU64(value uint64) error {
	s.Buffer.WriteByte(byte(value))
	s.Buffer.WriteByte(byte(value >> 8))
	s.Buffer.WriteByte(byte(value >> 16))
	s.Buffer.WriteByte(byte(value >> 24))
	s.Buffer.WriteByte(byte(value >> 32))
	s.Buffer.WriteByte(byte(value >> 40))
	s.Buffer.WriteByte(byte(value >> 48))
	s.Buffer.WriteByte(byte(value >> 56))
	return nil
}

func (s *BinarySerializer) SerializeU128(value Uint128) error {
	s.SerializeU64(value.Low)
	s.SerializeU64(value.High)
	return nil
}

func (s *BinarySerializer) SerializeI8(value int8) error {
	s.SerializeU8(uint8(value))
	return nil
}

func (s *BinarySerializer) SerializeI16(value int16) error {
	s.SerializeU16(uint16(value))
	return nil
}

func (s *BinarySerializer) SerializeI32(value int32) error {
	s.SerializeU32(uint32(value))
	return nil
}

func (s *BinarySerializer) SerializeI64(value int64) error {
	s.SerializeU64(uint64(value))
	return nil
}

func (s *BinarySerializer) SerializeI128(value Int128) error {
	s.SerializeU64(value.Low)
	s.SerializeI64(value.High)
	return nil
}

func (s *BinarySerializer) SerializeOptionTag(value bool) error {
	return s.SerializeBool(value)
}

func (s *BinarySerializer) GetBufferOffset() uint64 {
	return uint64(s.Buffer.Len())
}

func (s *BinarySerializer) GetBytes() []byte {
	return s.Buffer.Bytes()
}
