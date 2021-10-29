import 'HashUtils.dart';

class Tuple4<T0, T1, T2, T3> {
  T0 field0;
  T1 field1;
  T2 field2;
  T3 field3;

  Tuple4(T0 f0, T1 f1, T2 f2, T3 f3) {
    assert(f0 != null);
    assert(f1 != null);
    assert(f2 != null);
    assert(f3 != null);
    this.field0 = f0;
    this.field1 = f1;
    this.field2 = f2;
    this.field3 = f3;
  }

  @override
  bool operator ==(covariant Tuple4 other) {
    if (this == other) return true;
    if (other == null) return false;
    if (this.field0 == other.field0 &&
        this.field1 == other.field1 &&
        this.field2 == other.field2 &&
        this.field3 == other.field3) {
      return true;
    } else {
      return false;
    }
  }

  @override
  int get hashCode => $jf($jc(
      $jc($jc(this.field0.hashCode, this.field1.hashCode),
          this.field2.hashCode),
      this.field3.hashCode));
}
