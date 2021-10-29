part of serde;

abstract class BinaryDeserializer {
  ByteData input;
  int offset = 0;

  BinaryDeserializer(Uint8List input) : input = ByteData.view(input.buffer);

  bool deserialize_bool() {
    var result = this.input.getUint8(offset) != 0;
    this.offset += 1;
    return result;
  }

  Unit deserialize_unit() {
    return new Unit();
  }

  int deserialize_u8() {
    var result = this.input.getUint8(offset);
    this.offset += 1;
    return result;
  }

  int deserialize_u16() {
    var result = this.input.getUint16(offset, Endian.little);
    this.offset += 2;
    return result;
  }

  int deserialize_u32() {
    var result = this.input.getUint32(offset, Endian.little);
    this.offset += 4;
    return result;
  }

  int deserialize_u64() {
    var result = this.input.getUint64(offset, Endian.little);
    this.offset += 8;
    return result;
  }

  int deserialize_i8() {
    var result = this.input.getInt8(offset);
    this.offset += 1;
    return result;
  }

  int deserialize_i16() {
    var result = this.input.getInt16(offset, Endian.little);
    this.offset += 2;
    return result;
  }

  int deserialize_i32() {
    var result = this.input.getInt32(offset, Endian.little);
    this.offset += 4;
    return result;
  }

  int deserialize_i64() {
    var result = this.input.getInt64(offset, Endian.little);
    this.offset += 8;
    return result;
  }

  Bytes deserialize_bytes() {
    return new Bytes(deserialize_uint8list());
  }

  Uint8List deserialize_uint8list() {
    int len = deserialize_len();
    if (len < 0 || len > maxInt) {
      throw new Exception("The length of a array cannot exceed MAXINT");
    }
    Uint8List content = new Uint8List(len);
    for (int i = 0; i < len; i++) {
      content[i] = deserialize_u8();
    }
    return content;
  }

  bool deserialize_option_tag() {
    return deserialize_bool();
  }

  int deserialize_variant_index();

  String deserialize_str() {
    Uint8List value = deserialize_uint8list();
    return new String.fromCharCodes(value);
  }

  int get_buffer_offset() {
    return offset;
  }

  int deserialize_len();

  Int128 deserialize_u128() {
    var low = this.deserialize_u64();
    var high = this.deserialize_u64();
    return Int128(high, low);
  }

  int getUint8() {
    return deserialize_u8();
  }
}
