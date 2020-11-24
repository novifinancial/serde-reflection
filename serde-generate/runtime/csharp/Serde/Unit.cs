
using System;

namespace Serde
{
    public sealed class Unit : IEquatable<Unit>
    {
        public Unit() { }

        public override bool Equals(object obj) => obj is Unit unit;

        public bool Equals(Unit other) => other != null;

        public override int GetHashCode() => 793253941;
    }
}
