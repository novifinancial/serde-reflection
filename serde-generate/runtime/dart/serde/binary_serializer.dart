part of serde;

abstract class BinarySerializer {
  final List<int> output = List<int>.empty(growable: true);

  Uint8List get_bytes() {
    return Uint8List.fromList(output);
  }

  void serialize_uint8list(Uint8List val) {
    serialize_len(val.length);
    output.addAll(val);
  }

  void serialize_bytes(Bytes val) {
    serialize_len(val.content.length);
    output.addAll(val.content);
  }

  void serialize_bool(bool val) {
    output.addAll(Uint8List.fromList([val ? 1 : 0]));
  }

  void serialize_u8(int val) {
    output.addAll(Uint8List.fromList([val]));
  }

  void serialize_u16(int val) {
    final bdata = ByteData(2)..setUint16(0, val, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serialize_u32(int val) {
    final bdata = ByteData(4)..setUint32(0, val, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serialize_u64(int val) {
    final bdata = ByteData(8)..setUint64(0, val, Endian.little);
    output.addAll(bdata.buffer.asUint8List());
  }

  void serialize_i8(int value) {
    serialize_u8(value);
  }

  void serialize_i16(int value) {
    serialize_u16(value);
  }

  void serialize_i32(int value) {
    serialize_u32(value);
  }

  void serialize_i64(int value) {
    serialize_u64(value);
  }

  void serialize_option_tag(bool value) {
    output.addAll(Uint8List.fromList([value ? 1 : 0]));
  }

  void serialize_unit(Unit value) {}

  void serialize_variant_index(int index);

  void serialize_str(String str) {
    serialize_uint8list(Uint8List.fromList(str.codeUnits));
  }

  void serialize_len(int len);

  void serialize_u128(Int128 value) {
    serialize_u64(value.low);
    serialize_u64(value.high);
  }

  int get_buffer_offset() {
    return output.length;
  }
}
