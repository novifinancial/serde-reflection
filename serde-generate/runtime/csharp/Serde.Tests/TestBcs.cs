using System;
using System.Numerics;
using NUnit.Framework;
using Bcs;

namespace Serde.Tests
{
    [TestFixture]
    public class TestBcs
    {
        [Test]
        public void TestSerializeU128()
        {
            BcsSerializer serializer = new BcsSerializer();
            serializer.serialize_u128((BigInteger.One << 128) - 1);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 });

            serializer = new BcsSerializer();
            serializer.serialize_u128(BigInteger.One);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 });

            serializer = new BcsSerializer();
            serializer.serialize_u128(BigInteger.Zero);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 });

            Assert.Throws<SerializationException>(() => serializer.serialize_u128(BigInteger.MinusOne));

            Assert.Throws<SerializationException>(() => serializer.serialize_u128((BigInteger.One << 128) + 1));
        }

        [Test]
        public void TestSerializeI128()
        {
            BcsSerializer serializer = new BcsSerializer();
            serializer.serialize_i128(BigInteger.MinusOne);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255 });

            serializer = new BcsSerializer();
            serializer.serialize_i128(BigInteger.One);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 });

            serializer = new BcsSerializer();
            serializer.serialize_i128((BigInteger.One << 127) - 1);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 127 });

            serializer = new BcsSerializer();
            serializer.serialize_i128(-(BigInteger.One << 127));
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x80 });

            Assert.Throws<SerializationException>(() => serializer.serialize_i128(BigInteger.One << 127));

            Assert.Throws<SerializationException>(() => serializer.serialize_i128(-((BigInteger.One << 127) + 1)));
        }

        [Test]
        public void TestSliceOrdering()
        {
            BcsSerializer serializer = new BcsSerializer();
            serializer.serialize_u8(255);
            serializer.serialize_u32(1);
            serializer.serialize_u32(1);
            serializer.serialize_u32(2);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, /**/ 1, /**/ 0, 0, /**/ 0, 1, 0, /**/ 0, /**/ 0, /**/ 2, 0, 0, 0 });

            int[] offsets = { 1, 2, 4, 7, 8, 9 };
            serializer.sort_map_entries(offsets);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 255, /**/ 0, /**/ 0, /**/ 0, 0, /**/ 0, 1, 0,  /**/ 1, /**/ 2, 0, 0, 0 });
        }

        [Test]
        public void TestULEB128Encoding()
        {
            BcsSerializer serializer = new BcsSerializer();
            serializer.serialize_len(0);
            serializer.serialize_len(1);
            serializer.serialize_len(127);
            serializer.serialize_len(128);
            serializer.serialize_len(3000);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 0, 1, 127, 128, 1, 184, 23 });
        }
    }
}
