using System;

namespace Serde
{
    public sealed class SerializationException : Exception
    {
        public SerializationException(string message) : base(message) { }
    }
}
