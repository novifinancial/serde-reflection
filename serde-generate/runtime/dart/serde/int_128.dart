part of serde;

@immutable
class Int128 {
  const Int128(this.high, this.low);

  factory Int128.fromBigInt(BigInt num) {
    final high = (num >> 64).toInt();
    final low = (num & BigInt.from(0xFFFFFFFFFFFFFFFF)).toInt();
    return Int128(high, low);
  }

  factory Int128.fromJson(String json) {
    final num = BigInt.parse(json);
    final high = (num >> 64).toInt();
    final low = (num & BigInt.from(0xFFFFFFFFFFFFFFFF)).toInt();
    return Int128(high, low);
  }

  final int high;
  final int low;

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;

    return other is Int128 && high == other.high && low == other.low;
  }

  @override
  int get hashCode => Object.hash(
        high,
        low,
      );

  @override
  String toString() {
    return "$high$low";
  }

  BigInt toBigInt() => (BigInt.from(high) << 64) + BigInt.from(low);

  String toJson() => toBigInt().toString();
}
