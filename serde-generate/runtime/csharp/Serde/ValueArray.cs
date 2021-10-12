// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;

namespace Serde
{
    /// <summary>
    /// Immutable wrapper class around T[]. Implements value semantics for
    /// <see cref="object.Equals(object)"/> and <see cref="object.GetHashCode"/>.
    /// </summary>
    public class ValueArray<T> : IEquatable<ValueArray<T>>, IReadOnlyList<T>, IStructuralEquatable
    where T: IEquatable<T>
    {
        private readonly T[] array;
        private int? hashCode;

        public int Count => array.Length;

        public T this[int index] => array[index];

        public ValueArray(T[] data) { 
            array = data ?? throw new ArgumentNullException(nameof(data));
            hashCode = null;
        }

        public T[] ToArray() => array.ToArray();

        public static implicit operator ReadOnlySpan<T>(ValueArray<T> bytes) => bytes.array;

        public ReadOnlySpan<T> AsReadOnlySpan() => array;

        public override bool Equals(object obj) => obj is ValueArray<T> bytes && Equals(bytes);

        public bool Equals(ValueArray<T> other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (Count != other.Count) return false;
            for (int i = 0; i < Count; i++)
                if (!array[i].Equals(other[i])) return false;
            return true;
        }

        public static bool operator ==(ValueArray<T> left, ValueArray<T> right) => Equals(left, right);

        public static bool operator !=(ValueArray<T> left, ValueArray<T> right) => !Equals(left, right);

        public IEnumerator<T> GetEnumerator() => ((IEnumerable<T>)array).GetEnumerator();

        IEnumerator IEnumerable.GetEnumerator() => array.GetEnumerator();

        public override int GetHashCode()
        {
            unchecked
            {
                if (hashCode.HasValue) return hashCode.Value;
                int code = 1849862467;
                foreach (T elem in array)
                    code = code * 31 + elem.GetHashCode();
                hashCode = code;
                return code;
            }
        }

        public bool Equals(object other, IEqualityComparer comparer)
        {
            return ((IStructuralEquatable)array).Equals(other, comparer);
        }

        public int GetHashCode(IEqualityComparer comparer)
        {
            return ((IStructuralEquatable)array).GetHashCode(comparer);
        }
    }
}
