part of bcs_test;

void runBincodeTests() {
  test('serializer u32 work', () {
    BincodeSerializer serializer = new BincodeSerializer();
    serializer.serialize_u32(1);
    expect(serializer.get_bytes(), Uint8List.fromList([1, 0, 0, 0]));
  });

  test('deserializer u32 work', () {
    BincodeDeserializer serializer =
        new BincodeDeserializer(Uint8List.fromList([1, 0, 0, 0]));
    int result = serializer.deserialize_u32();
    expect(result, 1);
  });

  test('test slice work', () {
    BincodeSerializer serializer = new BincodeSerializer();
    serializer.serialize_u8(-1);
    serializer.serialize_u32(1);
    serializer.serialize_u32(1);
    serializer.serialize_u32(2);
    expect(
        serializer.get_bytes(),
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
