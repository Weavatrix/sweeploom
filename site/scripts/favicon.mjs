// Writes public/favicon.ico (16, 32, 48 px) using the same per-pixel test as
// apps/sweeploom-gui/src/mark.rs, supersampled 4x4 for smooth edges.
// Run from site/: `node scripts/favicon.mjs`. Output is committed.
import { deflateSync } from "node:zlib";
import { writeFileSync } from "node:fs";

const GOLD = [196, 140, 64];
const INK = [92, 62, 24];

function sample(u, v) {
  const r = Math.hypot(u, v);
  const ring = Math.abs(r - 0.68) < 0.1 && r < 0.92;
  const warp = Math.abs(u - v * 0.15) < 0.09 && r < 0.62;
  const weft = Math.abs(v + u * 0.12) < 0.09 && r < 0.62;
  const hub = r < 0.12;
  if (hub) return INK;
  if (ring || warp || weft) return GOLD;
  return null;
}

function rgba(n) {
  const S = 4;
  const px = Buffer.alloc(n * n * 4);
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) {
      let r = 0, g = 0, b = 0, a = 0;
      for (let sy = 0; sy < S; sy++) {
        for (let sx = 0; sx < S; sx++) {
          const u = ((x + (sx + 0.5) / S) / n) * 2 - 1;
          const v = ((y + (sy + 0.5) / S) / n) * 2 - 1;
          const c = sample(u, v);
          if (c) { r += c[0]; g += c[1]; b += c[2]; a += 1; }
        }
      }
      const i = (y * n + x) * 4;
      if (a) { px[i] = r / a; px[i + 1] = g / a; px[i + 2] = b / a; }
      px[i + 3] = Math.round((a / (S * S)) * 255);
    }
  }
  return px;
}

const CRC = new Int32Array(256).map((_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c;
});
function crc32(buf) {
  let c = -1;
  for (const byte of buf) c = CRC[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}
function chunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type), data]);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(td));
  return Buffer.concat([len, td, crc]);
}
function png(n) {
  const raw = rgba(n);
  const rows = Buffer.alloc((n * 4 + 1) * n);
  for (let y = 0; y < n; y++) {
    rows[y * (n * 4 + 1)] = 0;
    raw.copy(rows, y * (n * 4 + 1) + 1, y * n * 4, (y + 1) * n * 4);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(n, 0); ihdr.writeUInt32BE(n, 4);
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(rows)),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const sizes = [16, 32, 48];
const images = sizes.map(png);
const header = Buffer.alloc(6);
header.writeUInt16LE(0, 0); header.writeUInt16LE(1, 2); header.writeUInt16LE(sizes.length, 4);
let offset = 6 + 16 * sizes.length;
const dir = sizes.map((n, i) => {
  const e = Buffer.alloc(16);
  e[0] = n; e[1] = n; e[2] = 0; e[3] = 0;
  e.writeUInt16LE(1, 4); e.writeUInt16LE(32, 6);
  e.writeUInt32LE(images[i].length, 8); e.writeUInt32LE(offset, 12);
  offset += images[i].length;
  return e;
});
writeFileSync(new URL("../public/favicon.ico", import.meta.url), Buffer.concat([header, ...dir, ...images]));
writeFileSync(new URL("../public/mark-512.png", import.meta.url), png(512));
console.log("wrote public/favicon.ico and public/mark-512.png");
