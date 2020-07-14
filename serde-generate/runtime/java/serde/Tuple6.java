// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package serde;

public class Tuple6<T0, T1, T2, T3, T4, T5> {
    public T0 field0;
    public T1 field1;
    public T2 field2;
    public T3 field3;
    public T4 field4;
    public T5 field5;

    public Tuple6() {}

    public Tuple6(T0 f0, T1 f1, T2 f2, T3 f3, T4 f4, T5 f5) {
        this.field0 = f0;
        this.field1 = f1;
        this.field2 = f2;
        this.field3 = f3;
        this.field4 = f4;
        this.field5 = f5;
    }

    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null) return false;
        if (getClass() != obj.getClass()) return false;
        Tuple6 other = (Tuple6) obj;
        if (!this.field0.equals(other.field0)) { return false; }
        if (!this.field1.equals(other.field1)) { return false; }
        if (!this.field2.equals(other.field2)) { return false; }
        if (!this.field3.equals(other.field3)) { return false; }
        if (!this.field4.equals(other.field4)) { return false; }
        if (!this.field5.equals(other.field5)) { return false; }
        return true;
    }

    public int hashCode() {
        int value = 7;
        value = 31 * value + this.field0.hashCode();
        value = 31 * value + this.field1.hashCode();
        value = 31 * value + this.field2.hashCode();
        value = 31 * value + this.field3.hashCode();
        value = 31 * value + this.field4.hashCode();
        value = 31 * value + this.field5.hashCode();
        return value;
    }
}
