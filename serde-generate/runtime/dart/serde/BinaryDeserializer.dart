part of serde;


class BinaryDeserializer {
  ByteData input;
  int offset = 0;

  BinaryDeserializer(Uint8List input) {
    this.input = ByteData.view(input.buffer);
  }

  Uint8List deserialize_Uint8List() {
    var length = this.deserialize_u32();
    var result = this.input.buffer.asUint8List(this.offset, length);
    this.offset += length;
    return result;
  }

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
    this.offset += 1;
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

  Uint8List deserialize_bytes() {
    return null;
  }

  bool deserialize_option_tag() {
    return deserialize_bool();
  }

  int deserialize_variant_index(){
    return 0;
  }

  String deserialize_str(){
    return null;
  }

  int get_buffer_offset() {
    return offset;
  }

  int deserialize_len(){
    return 0;
  }

  int deserialize_u128(){
    return 0;
  }

  int getUint8() {
    int result = this.input.getUint8(offset);
    this.offset++;
    return result;
  }
}
