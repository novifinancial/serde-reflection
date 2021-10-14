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
}
