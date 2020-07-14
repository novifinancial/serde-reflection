// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde;

public class Tuple2<T0, T1> {
    public T0 field0;
    public T1 field1;

    public Tuple2() {}

    public Tuple2(T0 f0, T1 f1) {
        this.field0 = f0;
        this.field1 = f1;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Tuple2 other = (Tuple2) obj;
        if (!this.field0.equals(other.field0)) { return false; }
        if (!this.field1.equals(other.field1)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + this.field0.hashCode();
        value = 31 * value + this.field1.hashCode();
        return value;
    }

}
