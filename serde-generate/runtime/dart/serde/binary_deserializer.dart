part of serde;

abstract class BinaryDeserializer {
  BinaryDeserializer(Uint8List input) : input = ByteData.view(input.buffer);

  @protected
  final ByteData input;
  int _offset = 0;

  int get offset {
    return _offset;
  }

  bool deserializeBool() {
    final result = input.getUint8(_offset) != 0;
    _offset += 1;
    return result;
  }

  Unit deserializeUnit() {
    return const Unit();
  }

  int deserializeUint8() {
    final result = input.getUint8(_offset);
    _offset += 1;
    return result;
  }

  int deserializeUint16() {
    final result = input.getUint16(_offset, Endian.little);
    _offset += 2;
    return result;
  }

  int deserializeUint32() {
    final result = input.getUint32(_offset, Endian.little);
    _offset += 4;
    return result;
  }

  int deserializeUint64() {
    final result = input.getUint64(_offset, Endian.little);
    _offset += 8;
    return result;
  }

  int deserializeInt8() {
    final result = input.getInt8(_offset);
    _offset += 1;
    return result;
  }

  int deserializeInt16() {
    final result = input.getInt16(_offset, Endian.little);
    _offset += 2;
    return result;
  }

  int deserializeInt32() {
    final result = input.getInt32(_offset, Endian.little);
    _offset += 4;
    return result;
  }

  int deserializeInt64() {
    final result = input.getInt64(_offset, Endian.little);
    _offset += 8;
    return result;
  }

  double deserializeFloat64() {
    final result = input.getFloat64(_offset, Endian.little);
    _offset += 8;
    return result;
  }

  Bytes deserializeBytes() {
    return Bytes(deserializeUint8List());
  }

  Uint8List deserializeUint8List() {
    final len = deserializeLength();
    if (len < 0 || len > maxInt) {
      throw Exception('The length of an array cannot exceed MAXINT');
    }
    final content = Uint8List(len);
    for (var i = 0; i < len; i++) {
      content[i] = deserializeUint8();
    }
    return content;
  }

  bool deserializeOptionTag() {
    return deserializeBool();
  }

  int deserializeVariantIndex();

  String deserializeString() {
    return String.fromCharCodes(deserializeUint8List());
  }

  int deserializeLength();

  Int128 deserializeUint128() {
    final low = deserializeUint64();
    final high = deserializeUint64();
    return Int128(high, low);
  }
}
