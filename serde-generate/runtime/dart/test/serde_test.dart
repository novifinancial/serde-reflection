part of bcs_test;

void runSerdeTests() {
  test('serializer u8list work', () {
    Uint8List list1 = new Uint8List.fromList([1, 1, 1, 1]);
    Uint8List list2 = new Uint8List(4);
    for (int i = 0; i < 4; i++) {
      list2[i] = 1;
    }
    expect(list1, list2);
  });
}
