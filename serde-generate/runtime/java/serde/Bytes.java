// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde;

import java.util.Arrays;

/**
 * Wrapper around byte[] so that `equals` and `hashCode` compose as expected.
 */
public class Bytes {
    public byte[] content;

    public Bytes() {
    }

    public Bytes(byte[] content) {
        this.content = content;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Bytes other = (Bytes) obj;
        return Arrays.equals(this.content, other.content);
    }

    public int hashCode() {
        return Arrays.hashCode(content);
    }

}
