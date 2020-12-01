using System;

namespace Serde
{
    public static class Verification
    {
        /// <summary>
        /// Returns an integer corresponding to the lexicographic ordering of the two input byte strings.
        /// </summary>
        public static int CompareLexicographic(ReadOnlySpan<byte> key1, ReadOnlySpan<byte> key2)
        {
            for (int i = 0; i < key1.Length; i++)
            {
                var byte1 = key1[i];
                if (i >= key2.Length)
                {
                    return 1;
                }
                var byte2 = key2[i];
                if (byte1 > byte2)
                {
                    return 1;
                }
                if (byte1 < byte2)
                {
                    return -1;
                }
            }
            if (key2.Length > key1.Length)
            {
                return -1;
            }
            return 0;
        }
    }
}
