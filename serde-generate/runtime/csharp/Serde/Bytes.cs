using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;

namespace Serde
{
    /// <summary>
    /// Immutable wrapper class around <see cref="byte[]"/>. Implements value semantics for
    /// <see cref="object.Equals(object)"/> and <see cref="object.GetHashCode"/>.
    /// </summary>
    /// <remarks>
    /// ImmutableArray is not used because it is implemented using a btree as it is
    /// optimized for making modified copies.
    /// </remarks>
    public readonly struct Bytes : IEquatable<Bytes>, IReadOnlyList<byte>
    {
        private readonly byte[] array;

        public int Count => array.Length;

        public byte this[int index] => throw new NotImplementedException();

        public Bytes(byte[] data) { 
            array = data ?? throw new ArgumentNullException(nameof(data));
        }

        public byte[] ToArray() => array.ToArray();

        public static implicit operator ReadOnlySpan<byte>(Bytes bytes) => bytes.array;

        public ReadOnlySpan<byte> AsReadOnlySpan() => array;

        public override bool Equals(object obj) => obj is Bytes bytes && Equals(bytes);

        public bool Equals(Bytes other) => Enumerable.SequenceEqual(array, other.array);

        public override int GetHashCode() => HashCode.Combine(array);

        public static bool operator ==(Bytes left, Bytes right) => Equals(left, right);

        public static bool operator !=(Bytes left, Bytes right) => !Equals(left, right);

        public IEnumerator<byte> GetEnumerator() => ((IEnumerable<byte>)array).GetEnumerator();

        IEnumerator IEnumerable.GetEnumerator() => array.GetEnumerator();
    }
}
