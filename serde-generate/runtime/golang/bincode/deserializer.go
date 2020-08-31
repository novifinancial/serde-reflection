// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package bincode

import (
	"github.com/novifinancial/serde-reflection/serde-generate/runtime/golang/serde"
)

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
	return d.DeserializeU64()
}

func (d *deserializer) DeserializeVariantIndex() (uint32, error) {
	return d.DeserializeU32()
}

func (d *deserializer) CheckThatKeySlicesAreIncreasing(key1, key2 serde.Slice) error {
	// No need to check key ordering in Bincode.
	return nil
}
