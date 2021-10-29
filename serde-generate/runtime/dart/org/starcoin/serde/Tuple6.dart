import 'HashUtils.dart';

class Tuple6<T0, T1, T2, T3, T4, T5> {
  T0 field0;
  T1 field1;
  T2 field2;
  T3 field3;
  T4 field4;
  T5 field5;

  Tuple6(T0 f0, T1 f1, T2 f2, T3 f3, T4 f4, T5 f5) {
    assert(f0 != null);
    assert(f1 != null);
    assert(f2 != null);
    assert(f3 != null);
    assert(f4 != null);
    assert(f5 != null);
    this.field0 = f0;
    this.field1 = f1;
    this.field2 = f2;
    this.field3 = f3;
    this.field4 = f4;
    this.field5 = f5;
  }

  @override
  bool operator ==(covariant Tuple6 other) {
    if (this == other) return true;
    if (other == null) return false;
    if (this.field0 == other.field0 &&
        this.field1 == other.field1 &&
        this.field2 == other.field2 &&
        this.field3 == other.field3 &&
        this.field4 == other.field4 &&
        this.field5 == other.field5) {
      return true;
    } else {
      return false;
    }
  }

  @override
  int get hashCode => $jf($jc(
      $jc(
          $jc(
              $jc($jc(this.field0.hashCode, this.field1.hashCode),
                  this.field2.hashCode),
              this.field3.hashCode),
          this.field4.hashCode),
      this.field5.hashCode));
}
