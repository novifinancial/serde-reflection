using System;

namespace Serde {
    public sealed class DeserializationException : Exception {
        public DeserializationException(string message) : base(message) {}
    }
}
