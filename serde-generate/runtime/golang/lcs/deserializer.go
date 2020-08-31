// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package lcs

import (
	"bytes"
	"errors"

	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/serde"
)

// MaxSequenceLength is max length allowed for sequence.
const MaxSequenceLength = (1 << 31) - 1

const maxUint32 = uint64(^uint32(0))

// `deserializer` extends `serde.BinaryDeserializer` to implement `serde.Deserializer`.
type deserializer struct {
	serde.BinaryDeserializer
}

func NewDeserializer(input []byte) serde.Deserializer {
	return &deserializer{*serde.NewBinaryDeserializer(input)}
}

func (d *deserializer) DeserializeBytes() ([]byte, error) {
	return d.BinaryDeserializer.DeserializeBytes(d.DeserializeLen)
}

func (d *deserializer) DeserializeStr() (string, error) {
	return d.BinaryDeserializer.DeserializeStr(d.DeserializeLen)
}

func (d *deserializer) DeserializeLen() (uint64, error) {
	ret, err := d.deserializeUleb128AsU32()
	if ret > MaxSequenceLength {
		return 0, errors.New("length is too large")
	}
	return uint64(ret), err
}

func (d *deserializer) DeserializeVariantIndex() (uint32, error) {
	return d.deserializeUleb128AsU32()
}

func (d *deserializer) CheckThatKeySlicesAreIncreasing(key1, key2 serde.Slice) error {
	if bytes.Compare(d.Input[key1.Start:key1.End], d.Input[key2.Start:key2.End]) >= 0 {
		return errors.New("Error while decoding map: keys are not serialized in the expected order")
	}
	return nil
}

func (d *deserializer) deserializeUleb128AsU32() (uint32, error) {
	var value uint64
	for shift := 0; shift < 32; shift += 7 {
		byte, err := d.Buffer.ReadByte()
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
