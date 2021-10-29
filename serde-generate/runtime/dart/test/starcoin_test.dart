part of bcs_test;

void runStarcoinTests() {
  test('AccountAddress', () {
    AccountAddress accountAddress = new AccountAddress(List<int>.filled(16, 1));
    var expect_result = Uint8List.fromList([
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
      1,
    ]);
    expect(accountAddress.bcsSerialize(), expect_result);

    AccountAddress address = AccountAddress.bcsDeserialize(expect_result);
    expect(address, accountAddress);

    expect(AccountAddress.fromJson(jsonDecode(jsonEncode(accountAddress))),
        accountAddress);
  });

  test('AccessPath', () {
    AccountAddress accountAddress = new AccountAddress(List<int>.filled(16, 1));
    AccessPath accessPath1 =
        new AccessPath(accountAddress, DataPathCodeItem(Identifier("123")));
    var result = accessPath1.bcsSerialize();

    print(result);
    var expect_result = Uint8List.fromList(
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 3, 49, 50, 51]);
    expect(result, expect_result);

    AccessPath path = AccessPath.bcsDeserialize(expect_result);
    expect(accessPath1, path);

    print(jsonEncode(accessPath1));
    expect(
        AccessPath.fromJson(jsonDecode(jsonEncode(accessPath1))), accessPath1);
  });

  test('TransactionArgument', () {
    var u8ags = TransactionArgumentU8Item(1);
    var result = u8ags.bcsSerialize();
    var expect_result = Uint8List.fromList([0, 1]);
    expect(result, expect_result);

    var u8args_de = TransactionArgument.bcsDeserialize(expect_result);
    expect(u8args_de, u8ags);

    print(jsonEncode(u8args_de));

    expect(TransactionArgument.fromJson(jsonDecode(jsonEncode(u8args_de))),
        u8args_de);
  });

  test('TransactionPayload', () {
    var type_tag = List.filled(1, TypeTagU8Item());
    var u8ags = List.filled(1, TransactionArgumentU8Item(1));
    var code = Bytes(Uint8List.fromList([0, 1]));

    var script = Script(code, type_tag, u8ags);
    var t_script = TransactionPayloadScriptItem(script);

    var expect_result = Uint8List.fromList([0, 2, 0, 1, 1, 1, 1, 0, 1]);
    var result = t_script.bcsSerialize();

    expect(result, expect_result);

    var payload = TransactionPayload.bcsDeserialize(result);
    expect(payload, t_script);

    print(jsonEncode(payload));

    expect(
        TransactionPayload.fromJson(jsonDecode(jsonEncode(payload))), payload);
  });
}
