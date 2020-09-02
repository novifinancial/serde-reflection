import 'HashUtils.dart';

class Tuple2<T0, T1> {
  T0 field0;
  T1 field1;

  Tuple2(T0 f0, T1 f1) {
    assert(f0 != null);
    assert(f1 != null);
    this.field0 = f0;
    this.field1 = f1;
  }

  @override
  bool operator ==(covariant Tuple2 other) {
    if (other == null) return false;
    if (this.field0 == other.field0 && this.field1 == other.field1) {
      return true;
    } else {
      return false;
    }
  }

  @override
  int get hashCode => $jf($jc(this.field0.hashCode, this.field1.hashCode));
}
