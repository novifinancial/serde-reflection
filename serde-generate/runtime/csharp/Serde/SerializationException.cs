// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;

namespace Serde
{
    public sealed class SerializationException : Exception
    {
        public SerializationException(string message) : base(message) { }
    }
}
