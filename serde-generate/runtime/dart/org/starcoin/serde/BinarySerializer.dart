import 'dart:typed_data';
import 'Unit.dart';

class BinarySerializer {
  List<int> output;

  BinarySerializer() {
    output = new List<int>();
  }

  Uint8List getBytes() {
    return Uint8List.fromList(this.output);
  }

  void serialize_bytes(Uint8List val) {
    var bdata = new ByteData(4);
    bdata.setUint32(0, val.length, Endian.little);
    this.output.addAll(bdata.buffer.asUint8List());
    this.output.addAll(val);
  }

  void serialize_bool(bool val) {
    this.output.addAll(Uint8List.fromList([val ? 1 : 0]));
  }

  void serialize_u8(int val) {
    this.output.addAll(Uint8List.fromList([val]));
  }

  void serialize_u16(int val) {
    var bdata = new ByteData(2);
    bdata.setUint16(0, val, Endian.little);
    this.output.addAll(bdata.buffer.asUint8List());
  }

  void serialize_u32(int val) {
    var bdata = new ByteData(4);
    bdata.setUint32(0, val, Endian.little);
    this.output.addAll(bdata.buffer.asUint8List());
  }

  void serialize_u64(int val) {
    var bdata = new ByteData(8);
    bdata.setUint64(0, val, Endian.little);
    this.output.addAll(bdata.buffer.asUint8List());
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
    this.output.addAll(Uint8List.fromList([value ? 1 : 0]));
  }

  void serialize_unit(Unit value) {}

  int get_buffer_offset() {
    return output.length;
  }
}
