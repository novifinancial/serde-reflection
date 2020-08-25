// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package lcs

import (
	"bytes"
	"errors"
	"sort"

	"github.com/facebookincubator/serde-reflection/serde-generate/runtime/golang/serde"
)

// Serializer implements `serde.Serializer` interface for serializing LCS bytes.
type Serializer struct {
	buf bytes.Buffer
}

// NewSerializer creates a new `serde.Serializer`.
func NewSerializer() serde.Serializer {
	return new(Serializer)
}

func (s *Serializer) GetBytes() []byte {
	return s.buf.Bytes()
}

func (s *Serializer) SerializeLen(value uint64) error {
	if value > MaxSequenceLength {
		return errors.New("length is too large")
	}
	s.serializeU32AsUleb128(uint32(value))
	return nil
}

func (s *Serializer) SerializeBytes(value []byte) error {
	s.SerializeLen(uint64(len(value)))
	s.buf.Write(value)
	return nil
}

func (s *Serializer) SerializeStr(value string) error {
	return s.SerializeBytes([]byte(value))
}

func (s *Serializer) SerializeBool(value bool) error {
	if value {
		return s.buf.WriteByte(1)
	}
	return s.buf.WriteByte(0)
}

func (s *Serializer) SerializeUnit(value struct{}) error {
	return nil
}

// SerializeChar is unimplemented.
func (s *Serializer) SerializeChar(value rune) error {
	return errors.New("unimplemented")
}

// SerializeF32 is unimplemented
func (s *Serializer) SerializeF32(value float32) error {
	return errors.New("unimplemented")
}

// SerializeF64 is unimplemented
func (s *Serializer) SerializeF64(value float64) error {
	return errors.New("unimplemented")
}

func (s *Serializer) SerializeU8(value uint8) error {
	s.buf.WriteByte(byte(value))
	return nil
}

func (s *Serializer) SerializeU16(value uint16) error {
	s.buf.WriteByte(byte(value))
	s.buf.WriteByte(byte(value >> 8))
	return nil
}

func (s *Serializer) SerializeU32(value uint32) error {
	s.buf.WriteByte(byte(value))
	s.buf.WriteByte(byte(value >> 8))
	s.buf.WriteByte(byte(value >> 16))
	s.buf.WriteByte(byte(value >> 24))
	return nil
}

func (s *Serializer) SerializeU64(value uint64) error {
	s.buf.WriteByte(byte(value))
	s.buf.WriteByte(byte(value >> 8))
	s.buf.WriteByte(byte(value >> 16))
	s.buf.WriteByte(byte(value >> 24))
	s.buf.WriteByte(byte(value >> 32))
	s.buf.WriteByte(byte(value >> 40))
	s.buf.WriteByte(byte(value >> 48))
	s.buf.WriteByte(byte(value >> 56))
	return nil
}

func (s *Serializer) SerializeU128(value serde.Uint128) error {
	s.SerializeU64(value.Low)
	s.SerializeU64(value.High)
	return nil
}

func (s *Serializer) SerializeI8(value int8) error {
	s.SerializeU8(uint8(value))
	return nil
}

func (s *Serializer) SerializeI16(value int16) error {
	s.SerializeU16(uint16(value))
	return nil
}

func (s *Serializer) SerializeI32(value int32) error {
	s.SerializeU32(uint32(value))
	return nil
}

func (s *Serializer) SerializeI64(value int64) error {
	s.SerializeU64(uint64(value))
	return nil
}

func (s *Serializer) SerializeI128(value serde.Int128) error {
	s.SerializeU64(value.Low)
	s.SerializeI64(value.High)
	return nil
}

func (s *Serializer) SerializeVariantIndex(value uint32) error {
	s.serializeU32AsUleb128(value)
	return nil
}

func (s *Serializer) SerializeOptionTag(value bool) error {
	return s.SerializeBool(value)
}

func (s *Serializer) GetBufferOffset() uint64 {
	return uint64(s.buf.Len())
}

func (s *Serializer) SortMapEntries(offsets []uint64) {
	if len(offsets) <= 1 {
		return
	}
	data := s.GetBytes()
	slices := make([]serde.Slice, len(offsets))
	for i, v := range offsets {
		var w uint64
		if i + 1 < len(offsets) {
			w = offsets[i+1]
		} else {
			w = uint64(len(data))
		}
		slices[i] = serde.Slice{Start: v, End: w}
	}
	entries := map_entries{data, slices}
	sort.Sort(entries)
	buffer := make([]byte, len(data)-int(offsets[0]))
	current := buffer[0:0]
	for _, slice := range entries.slices {
		current = append(current, data[slice.Start:slice.End]...)
	}
	copy(data[offsets[0]:], current)
}

func (s *Serializer) serializeU32AsUleb128(value uint32) {
	for value >= 0x80 {
		b := byte((value & 0x7f) | 0x80)
		_ = s.buf.WriteByte(b)
		value = value >> 7
	}
	_ = s.buf.WriteByte(byte(value))
}

type map_entries struct {
	data   []byte
	slices []serde.Slice
}

func (a map_entries) Len() int { return len(a.slices) }

func (a map_entries) Less(i, j int) bool {
	slice_i := a.data[a.slices[i].Start:a.slices[i].End]
	slice_j := a.data[a.slices[j].Start:a.slices[j].End]
	return bytes.Compare(slice_i, slice_j) < 0
}

func (a map_entries) Swap(i, j int) { a.slices[i], a.slices[j] = a.slices[j], a.slices[i] }
