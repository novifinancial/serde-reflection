import 'dart:typed_data';
import 'package:test/test.dart';
import '../../lib/src/bincode/bincode.dart';

void runBincodeTests() {
  test('serializeUint32', () {
    BincodeSerializer serializer = BincodeSerializer();
    serializer.serializeUint32(1);
    expect(serializer.bytes, Uint8List.fromList([1, 0, 0, 0]));
  });

  test('deserializeUint32', () {
    BincodeDeserializer serializer =
        BincodeDeserializer(Uint8List.fromList([1, 0, 0, 0]));
    int result = serializer.deserializeUint32();
    expect(result, 1);
  });

  test('slice', () {
    BincodeSerializer serializer = BincodeSerializer();
    serializer.serializeUint8(-1);
    serializer.serializeUint32(1);
    serializer.serializeUint32(1);
    serializer.serializeUint32(2);
    expect(
        serializer.bytes,
        Uint8List.fromList([
          -1,
          /**/ 1,
          /**/ 0,
          0,
          /**/ 0,
          1,
          0,
          /**/ 0,
          /**/ 0,
          /**/ 2,
          0,
          0,
          0
        ]));
  });

  test('serializeUint8', () {
    final serializer = BincodeSerializer();
    serializer.serializeUint8(255);
    final deserializer = BincodeDeserializer(serializer.bytes);
    expect(deserializer.deserializeUint8(), 255);
    expect(() => serializer.serializeUint8(256),
        throwsA('The integer literal 256 can\'t be represented in 8 bits.'));
  });

  test('serializeUint16', () {
    final serializer = BincodeSerializer();
    serializer.serializeUint16(65535);
    final deserializer = BincodeDeserializer(serializer.bytes);
    expect(deserializer.deserializeUint16(), 65535);
    expect(() => serializer.serializeUint16(65536),
        throwsA('The integer literal 65536 can\'t be represented in 16 bits.'));
  });

  test('serializeUint32', () {
    final serializer = BincodeSerializer();
    serializer.serializeUint32(4294967295);
    final deserializer = BincodeDeserializer(serializer.bytes);
    expect(deserializer.deserializeUint32(), 4294967295);
    expect(
        () => serializer.serializeUint32(4294967296),
        throwsA(
            'The integer literal 4294967296 can\'t be represented in 32 bits.'));
  });

  test('serializeInt8', () {
    final serializer = BincodeSerializer();
    serializer.serializeInt8(127);
    final deserializer = BincodeDeserializer(serializer.bytes);
    expect(deserializer.deserializeInt8(), 127);
    expect(() => serializer.serializeInt8(128),
        throwsA('The integer literal 128 can\'t be represented in 8 bits.'));
    expect(() => serializer.serializeInt8(-129),
        throwsA('The integer literal -129 can\'t be represented in 8 bits.'));
  });

  test('serializeInt16', () {
    final serializer = BincodeSerializer();
    serializer.serializeInt16(32767);
    final deserializer = BincodeDeserializer(serializer.bytes);
    expect(deserializer.deserializeInt16(), 32767);
    expect(() => serializer.serializeInt16(32768),
        throwsA('The integer literal 32768 can\'t be represented in 16 bits.'));
    expect(
        () => serializer.serializeInt16(-32769),
        throwsA(
            'The integer literal -32769 can\'t be represented in 16 bits.'));
  });

  test('serializeInt32', () {
    final serializer = BincodeSerializer();
    serializer.serializeInt32(2147483647);
    final deserializer = BincodeDeserializer(serializer.bytes);
    expect(deserializer.deserializeInt32(), 2147483647);
    expect(
        () => serializer.serializeInt32(2147483648),
        throwsA(
            'The integer literal 2147483648 can\'t be represented in 32 bits.'));
    expect(
        () => serializer.serializeInt32(-2147483649),
        throwsA(
            'The integer literal -2147483649 can\'t be represented in 32 bits.'));
  });
}
