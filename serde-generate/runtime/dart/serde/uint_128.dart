part of serde;

///
/// A Dart type to represent the Rust u128 type.
///
/// Warning: this currently clamps at a max value of 34028236692093846346337460743176821 while Rust is 340282366920938463463374607431768211455
@immutable
class Uint128 {
  Uint128(this.high, this.low);

  factory Uint128.parse(String num, {int? radix}) {
    return Uint128.fromBigInt(BigInt.parse(num, radix: radix));
  }

  factory Uint128.fromBigInt(BigInt num) {
    final input = num.toUnsigned(128);
    final high = (input >> 64).toUnsigned(64);
    final low = (input & BigInt.from(0xFFFFFFFFFFFFFFFF)).toUnsigned(64);
    return Uint128(high.toInt(), low.toInt());
  }

  final int high;
  final int low;

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is Uint128 && high == other.high && low == other.low;
  }

  @override
  int get hashCode => Object.hash(
        high,
        low,
      );

  @override
  String toString() {
    return toBigInt().toString();
  }

  BigInt toBigInt() =>
      (BigInt.from(high).toUnsigned(64) << 64) +
      BigInt.from(low).toUnsigned(64);
}
