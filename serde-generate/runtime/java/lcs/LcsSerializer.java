// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

package lcs;

import java.lang.Exception;
import serde.Slice;
import serde.BinarySerializer;

public class LcsSerializer extends BinarySerializer {
    private void serialize_u32_as_uleb128(int value) {
        while ((value >>> 7) != 0) {
            output.write((value & 0x7f) | 0x80);
            value = value >>> 7;
        }
        output.write(value);
    }

    public void serialize_len(long value) throws Exception {
        serialize_u32_as_uleb128((int) value);
    }

    public void serialize_variant_index(int value) throws Exception {
        serialize_u32_as_uleb128(value);
    }

    public void sort_map_entries(int[] offsets) {
        if (offsets.length <= 1) {
            return;
        }
        int offset0 = offsets[0];
        byte[] content = output.getBuffer();
        Slice[] slices = new Slice[offsets.length];
        for (int i = 0; i < offsets.length - 1; i++) {
            slices[i] = new Slice(offsets[i], offsets[i + 1]);
        }
        slices[offsets.length - 1] = new Slice(offsets[offsets.length - 1], output.size());

        java.util.Arrays.sort(slices, new java.util.Comparator<Slice>() {
            @Override
            public int compare(Slice slice1, Slice slice2) {
                return Slice.compare_bytes(content, slice1, slice2);
            }
        });

        byte[] old_content = new byte[output.size() - offset0];
        System.arraycopy(content, offset0, old_content, 0, output.size() - offset0);

        int position = offset0;
        for (int i = 0; i < offsets.length; i++) {
            int start = slices[i].start;
            int end = slices[i].end;
            System.arraycopy(old_content, start - offset0, content, position, end - start);
            position += end - start;
        }
    }
}
