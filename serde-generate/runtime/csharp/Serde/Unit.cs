
using System;

namespace Serde
{
    ///<summary>
    /// Analogous to Rust's Unit type `()`.
    ///</summary>
    public readonly struct Unit : IEquatable<Unit>
    {
        public override bool Equals(object obj) => obj is Unit unit;

        public bool Equals(Unit other) => true;

        public static bool operator==(Unit l, Unit r) => true;

        public static bool operator!=(Unit l, Unit r) => false;

        public override int GetHashCode() => 793253941;
    }
}
