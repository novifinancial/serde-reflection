part of serde;

abstract class BinaryDeserializer {
  BinaryDeserializer(Uint8List input): input = ByteData.view(input.buffer);

  final ByteData input;
  int offset = 0;

  bool deserialize_bool() {
    final result = input.getUint8(offset) != 0;
    offset += 1;
    return result;
  }

  Unit deserialize_unit() {
    return new Unit();
  }

  int deserialize_u8() {
    final result = input.getUint8(offset);
    offset += 1;
    return result;
  }

  int deserialize_u16() {
    final result = input.getUint16(offset, Endian.little);
    offset += 2;
    return result;
  }

  int deserialize_u32() {
    final result = input.getUint32(offset, Endian.little);
    offset += 4;
    return result;
  }

  int deserialize_u64() {
    final result = input.getUint64(offset, Endian.little);
    offset += 8;
    return result;
  }

  int deserialize_i8() {
    final result = input.getInt8(offset);
    offset += 1;
    return result;
  }

  int deserialize_i16() {
    final result = input.getInt16(offset, Endian.little);
    offset += 2;
    return result;
  }

  int deserialize_i32() {
    final result = input.getInt32(offset, Endian.little);
    offset += 4;
    return result;
  }

  int deserialize_i64() {
    final result = input.getInt64(offset, Endian.little);
    offset += 8;
    return result;
  }

  Bytes deserialize_bytes() {
    return Bytes(deserialize_uint8list());
  }

  Uint8List deserialize_uint8list() {
    final len = deserialize_len();
    if (len < 0 || len > maxInt) {
      throw Exception('The length of an array cannot exceed MAXINT');
    }
    final content = Uint8List(len);
    for (var i = 0; i < len; i++) {
      content[i] = deserialize_u8();
    }
    return content;
  }

  bool deserialize_option_tag() {
    return deserialize_bool();
  }

  int deserialize_variant_index();

  String deserialize_str() {
    return String.fromCharCodes(deserialize_uint8list());
  }

  int get_buffer_offset() {
    return offset;
  }

  int deserialize_len();

  Int128 deserialize_u128() {
    final low = this.deserialize_u64();
    final high = this.deserialize_u64();
    return Int128(high, low);
  }

  int getUint8() {
    return deserialize_u8();
  }
}
