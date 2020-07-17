// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde;

import java.util.Arrays;

/**
 * Immutable wrapper class around byte[].
 *
 * Enforces value-semantice for `equals` and `hashCode`.
 */
public final class Bytes {
    private final byte[] content;

    public Bytes(byte[] content) {
        assert content != null;
        this.content = content.clone();
    }

    public byte[] content() {
        return this.content.clone();
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
