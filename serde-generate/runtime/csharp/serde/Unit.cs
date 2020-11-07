
namespace Serde {
    public sealed class Unit {
        public Unit() {}

        public override bool Equals(object obj)
        {
            if (obj == null || GetType() != obj.GetType())
                return false;
            return true;
        }
        
        public override int GetHashCode() => 7;
    }
}
