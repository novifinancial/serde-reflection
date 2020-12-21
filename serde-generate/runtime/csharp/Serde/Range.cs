using System;

namespace Serde
{
    public struct Range : IEquatable<Range>
    {
        public int Start { get; }
        public int End { get; }
        public int Length => End - Start;

        public Range(int start, int end)
        {
            Start = start;
            End = end;
        }

        public override int GetHashCode()
        {
            var hashCode = -1676728671;
            hashCode = hashCode * -1521134295 + Start.GetHashCode();
            hashCode = hashCode * -1521134295 + End.GetHashCode();
            return hashCode;
        }

        public override bool Equals(object obj) => obj is Range range && Equals(range);

        public bool Equals(Range other) => Start == other.Start && End == other.End;

        public static bool operator ==(Range range1, Range range2) => range1.Equals(range2);

        public static bool operator !=(Range range1, Range range2) => !(range1 == range2);
    }

    public static class RangeExtensions
    {
        public static Span<T> Slice<T>(this T[] array, Range range) =>
            new Span<T>(array, range.Start, range.Length);
        public static Span<T> Slice<T>(this ArraySegment<T> array, Range range) =>
            new ArraySegment<T>(array.Array, array.Offset + range.Start, range.Length);
    }
}
