part of serde;

const maxInt = 4294967296;

bool listEquals<T>(List<T>? a, List<T>? b) {
  if (a == null)
    return b == null;
  if (b == null || a.length != b.length)
    return false;
  if (identical(a, b))
    return true;
  for (var index = 0; index < a.length; index++) {
    if (a[index] != b[index])
      return false;
  }
  return true;
}
