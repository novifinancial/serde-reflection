import 'HashUtils.dart';

class Tuple3<T0, T1, T2> {
  T0 field0;
  T1 field1;
  T2 field2;

  Tuple3(T0 f0, T1 f1, T2 f2) {
    assert(f0 != null);
    assert(f1 != null);
    assert(f2 != null);
    this.field0 = f0;
    this.field1 = f1;
    this.field2 = f2;
  }

  @override
  bool operator ==(covariant Tuple3 other) {
    if (this == other) return true;
    if (other == null) return false;
    if (this.field0 == other.field0 &&
        this.field1 == other.field1 &&
        this.field2 == other.field2) {
      return true;
    } else {
      return false;
    }
  }

  @override
  int get hashCode => $jf($jc(
      $jc(this.field0.hashCode, this.field1.hashCode), this.field2.hashCode));
}
