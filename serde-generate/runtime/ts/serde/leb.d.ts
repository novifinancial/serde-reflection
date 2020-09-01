declare module 'leb' {
  declare function decodeInt32(
    buffer: Buffer,
    index?: number
  ): { value: number; nextIndex: number };
  declare function decodeInt64(
    buffer: Buffer,
    index?: number
  ): { value: number; nextIndex: number; lossy: boolean };
  declare function decodeIntBuffer(
    encodedBuffer: Buffer,
    index?: number
  ): { value: Buffer; nextIndex: number };
  declare function decodeUInt32(
    buffer: Buffer,
    index?: number
  ): { value: number; nextIndex: number };
  declare function decodeUInt64(
    buffer: Buffer,
    index?: number
  ): { value: number; nextIndex: number; lossy: boolean };
  declare function decodeUIntBuffer(
    encodedBuffer: Buffer,
    index?: number
  ): { value: Buffer; nextIndex: number };
  declare function encodeInt32(num?: number): Buffer;
  declare function encodeInt64(num?: number): Buffer;
  declare function encodeIntBuffer(buffer: Buffer): Buffer;
  declare function encodeUInt32(num?: number): Buffer;
  declare function encodeUInt64(num?: number): Buffer;
  declare function encodeUIntBuffer(buffer: Buffer): Buffer;
}
