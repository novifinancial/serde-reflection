import 'dart:typed_data';

class Slice {
  int start;
  int end;

  Slice(int start, int end) {
    this.start = start;
    this.end = end;
  }

  // Lexicographic comparison between the (unsigned!) bytes referenced by `slice1` and `slice2`
  // into `content`.
  static int compare_bytes(Uint8List content, Slice slice1, Slice slice2) {
    int start1 = slice1.start;
    int end1 = slice1.end;
    int start2 = slice2.start;
    int end2 = slice2.end;
    for (int i = 0; i < end1 - start1; i++) {
      int byte1 = content[start1 + i] & 0xFF;
      if (start2 + i >= end2) {
        return 1;
      }
      int byte2 = content[start2 + i] & 0xFF;
      if (byte1 > byte2) {
        return 1;
      }
      if (byte1 < byte2) {
        return -1;
      }
    }
    if (end2 - start2 > end1 - start1) {
      return -1;
    }
    return 0;
  }
}
