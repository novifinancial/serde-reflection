using System;
using System.Collections;
using System.Collections.Generic;

namespace Serde
{
    /// <summary>
    /// Immutable wrapper class around <see cref="Dictionary<K, V>"/>. Implements value semantics for
    /// <see cref="object.Equals(object)"/> and <see cref="object.GetHashCode"/>.
    /// </summary>
    public class ValueDictionary<K, V> : IEquatable<ValueDictionary<K, V>>, IReadOnlyDictionary<K, V>
    where K: IEquatable<K>
    where V: IEquatable<V>
    {
        private readonly Dictionary<K, V> dict;
        private int? hashCode;

        public int Count => dict.Count;

        public IEnumerable<K> Keys => dict.Keys;

        public IEnumerable<V> Values => dict.Values;

        public V this[K key] => dict[key];

        public ValueDictionary(Dictionary<K, V> dictionary) { 
            dict = dictionary ?? throw new ArgumentNullException(nameof(dictionary));
            hashCode = null;
        }

        public bool ContainsKey(K key) => dict.ContainsKey(key);

        public bool TryGetValue(K key, out V value) => dict.TryGetValue(key, out value);

        IEnumerator<KeyValuePair<K, V>> IEnumerable<KeyValuePair<K, V>>.GetEnumerator() => dict.GetEnumerator();

        public IEnumerator GetEnumerator() => ((IEnumerable)dict).GetEnumerator();

        public override bool Equals(object obj) => obj is ValueDictionary<K, V> bytes && Equals(bytes);

        public bool Equals(ValueDictionary<K, V> other) {
            if (other == null) return false;
            if (Count != other.Count) return false;
            foreach (var key in Keys)
            {
                if (!other.ContainsKey(key)) return false;
                if (!dict[key].Equals(other[key])) return false;
            }
            return true;
        }

        public static bool operator ==(ValueDictionary<K, V> left, ValueDictionary<K, V> right) => Equals(left, right);

        public static bool operator !=(ValueDictionary<K, V> left, ValueDictionary<K, V> right) => !Equals(left, right);


        public override int GetHashCode()
        {
            unchecked
            {
                if (hashCode.HasValue) return hashCode.Value;
                int code = 45053;
                foreach (var pair in dict)
                {
                    code = code * 31 + pair.Key.GetHashCode();
                    code = code * 31 + pair.Value.GetHashCode();
                }
                hashCode = code;
                return code;
            }
        }
    }
}
