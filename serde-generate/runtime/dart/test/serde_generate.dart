import 'dart:typed_data';
import 'package:example/example.dart';
import 'package:test/test.dart';
import 'package:tuple/tuple.dart';

void runSerdeGenerateTests() {
  test('Struct', () {
    final struct = Struct(x: 100, y: Uint64.parse('200000'));
    expect(
        Struct.bincodeDeserialize(struct.bincodeSerialize()), equals(struct));
  });

  test('C Enum', () {
    final val = CStyleEnum.a;

    expect(
        CStyleEnumExtension.bincodeDeserialize(CStyleEnum.a.bincodeSerialize()),
        equals(val));
  });

  test('UnitStruct', () {
    final val = UnitStruct();

    expect(UnitStruct.bincodeDeserialize(val.bincodeSerialize()), equals(val));
  });

  test('TupleStruct', () {
    final val = TupleStruct(field0: 10, field1: Uint64.parse('20'));

    expect(TupleStruct.bincodeDeserialize(val.bincodeSerialize()), equals(val));
  });

  test('SimpleList (recursion)', () {
    final val = SimpleList(value: SimpleList());

    expect(SimpleList.bincodeDeserialize(val.bincodeSerialize()), equals(val));
  });

  test('Primitive Types', () {
    final val = PrimitiveTypes(
      fBool: true,
      fU8: 255,
      fU16: 300,
      fU32: 3000000,
      fU64: Uint64.parse('18446744073709551615'),
      fU128: Uint128.parse('340282366920938463463374607431768211455'),
      fI8: -128,
      fI16: -400,
      fI32: -30000000,
      fI64: 9223372036854775807,
      fI128: Int128.parse('170141183460469231731687303715884105727'),
      fF32: 623929.125,
      fF64: 9223372036854775807.21,
      fChar: 20,
    );

    expect(
        PrimitiveTypes.bincodeDeserialize(val.bincodeSerialize()), equals(val));
  });

  test('Other Types', () {
    final val = OtherTypes(
        fString: "this is a string",
        fBytes: Bytes(Uint8List.fromList([1, 2, 3])),
        fOption: null,
        fUnit: Unit(),
        fSeq: List.filled(2, Struct(x: 5, y: Uint64.parse('100'))),
        fTuple: Tuple2(100, 300),
        fStringmap: {
          'key': 2000
        },
        fIntset: {
          Uint64.parse('500'): Unit()
        },
        fNestedSeq: [
          [Struct(x: 1, y: Uint64.parse('3'))]
        ]);

    expect(OtherTypes.bincodeDeserialize(val.bincodeSerialize()), equals(val));
  });
}
