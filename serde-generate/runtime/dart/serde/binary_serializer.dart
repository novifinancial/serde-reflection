part of serde;

abstract class BinarySerializer {
  final List<int> output = List<int>.empty(growable: true);

  Uint8List get bytes {
    return Uint8List.fromList(output);
  }

  int get offset {
    return output.length;
  }

  void serializeUint8List(Uint8List val) {
    serializeLength(val.length);
    output.addAll(val);
  }

  void serializeBytes(Bytes val) {
    serializeLength(val.content.length);
    output.addAll(val.content);
  }

  void serializeBool(bool val) {
    output.addAll(Uint8List.fromList([val ? 1 : 0]));
  }

  void serializeUint8(int val) {
    output.addAll(Uint8List.fromList([val]));
  }

  void serializeUint16(int val) {
    final bdata = ByteData(2)..setUint16(0, val, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeUint32(int val) {
    final bdata = ByteData(4)..setUint32(0, val, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeUint64(Uint64 val) {
    BigInt number = val.toBigInt();
    final _byteMask = BigInt.from(0xFF);
    int bytes = 8;
    var bdata = Uint8List(bytes);
    for (int i = 0; i < bytes; i++) {
      // big endian
      // bdata[bytes - i - 1] = (number & _byteMask).toInt();

      // little endian
      bdata[i] = (number & _byteMask).toInt();
      number = number >> 8;
    }

    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeInt8(int value) {
    final bdata = ByteData(1)..setInt8(0, value);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeInt16(int value) {
    final bdata = ByteData(2)..setInt16(0, value, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeInt32(int value) {
    final bdata = ByteData(4)..setInt32(0, value, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeInt64(int value) {
    final bdata = ByteData(8)..setInt64(0, value, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeFloat32(double value) {
    final bdata = ByteData(4)..setFloat32(0, value, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeFloat64(double value) {
    final bdata = ByteData(8)..setFloat64(0, value, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serializeOptionTag(bool value) {
    output.addAll(Uint8List.fromList([value ? 1 : 0]));
  }

  void serializeUintnit(Unit value) {}

  void serializeVariantIndex(int index);

  void serializeString(String str) {
    serializeUint8List(Uint8List.fromList(str.codeUnits));
  }

  void serializeLength(int len);

  void serializeInt128(Int128 value) {
    serializeInt64(value.low.toInt());
    serializeInt64(value.high.toInt());
  }

  void serializeUint128(Uint128 value) {
    serializeUint64(Uint64(value.low));
    serializeUint64(Uint64(value.high));
  }
}
