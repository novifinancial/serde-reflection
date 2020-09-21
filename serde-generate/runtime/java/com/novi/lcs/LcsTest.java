// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.novi.lcs;

import java.util.Arrays;
import java.lang.reflect.Method;
import java.math.BigInteger;
import java.lang.Runnable;

public class LcsTest {

    static void test_serialize_u128() throws Exception {
        LcsSerializer serializer = new LcsSerializer();
        serializer.serialize_u128(BigInteger.ONE.shiftLeft(128).subtract(BigInteger.ONE));
        assert Arrays.equals(serializer.get_bytes(), new byte[]{-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1});

        serializer = new LcsSerializer();
        serializer.serialize_u128(BigInteger.ONE);
        assert Arrays.equals(serializer.get_bytes(), new byte[]{1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0});

        serializer = new LcsSerializer();
        serializer.serialize_u128(BigInteger.ZERO);
        assert Arrays.equals(serializer.get_bytes(), new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0});

        try {
            serializer.serialize_u128(BigInteger.ONE.negate());
            assert false;
        } catch (java.lang.IllegalArgumentException e) { /* all good */  }

        try {
            serializer.serialize_u128(BigInteger.ONE.shiftLeft(128).add(BigInteger.ONE));
            assert false;
        } catch (java.lang.IllegalArgumentException e) { /* all good */  }
    }

    static void test_serialize_i128() throws Exception {
        LcsSerializer serializer = new LcsSerializer();
        serializer.serialize_i128(BigInteger.ONE.negate());
        assert Arrays.equals(serializer.get_bytes(), new byte[]{-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1});

        serializer = new LcsSerializer();
        serializer.serialize_i128(BigInteger.ONE);
        assert Arrays.equals(serializer.get_bytes(), new byte[]{1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0});

        serializer = new LcsSerializer();
        serializer.serialize_i128(BigInteger.ONE.shiftLeft(127).subtract(BigInteger.ONE));
        assert Arrays.equals(serializer.get_bytes(), new byte[]{-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 127});

        serializer = new LcsSerializer();
        serializer.serialize_i128(BigInteger.ONE.shiftLeft(127).negate());
        assert Arrays.equals(serializer.get_bytes(), new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -128});

        try {
            serializer.serialize_i128(BigInteger.ONE.shiftLeft(127));
            assert false;
        } catch (java.lang.IllegalArgumentException e) { /* all good */  }

        try {
            serializer.serialize_i128(BigInteger.ONE.shiftLeft(127).add(BigInteger.ONE).negate());
            assert false;
        } catch (java.lang.IllegalArgumentException e) { /* all good */  }
    }

    static void test_serializer_slice_ordering() throws Exception {
        LcsSerializer serializer = new LcsSerializer();

        serializer.serialize_u8((byte) -1);
        serializer.serialize_u32(1);
        serializer.serialize_u32(1);
        serializer.serialize_u32(2);
        assert Arrays.equals(serializer.get_bytes(), new byte[]{-1, /**/ 1, /**/ 0, 0, /**/ 0, 1, 0, /**/ 0, /**/ 0, /**/ 2, 0, 0, 0});

        int[] offsets = {1, 2, 4, 7, 8, 9};
        serializer.sort_map_entries(offsets);
        assert Arrays.equals(serializer.get_bytes(), new byte[]{-1, /**/ 0, /**/ 0, /**/ 0, 0, /**/ 0, 1, 0,  /**/ 1, /**/ 2, 0, 0, 0});
    }

    public static void main(String[] args) throws Exception {
        for (Method method : LcsTest.class.getDeclaredMethods()) {
            if (method.getName().startsWith("test_")) {
                method.invoke(null);
            }
        }
    }

}
