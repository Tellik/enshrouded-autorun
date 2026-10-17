function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function write16(view, offset, value) { view.setUint16(offset, value, true); }

function write32(view, offset, value) { view.setUint32(offset, value, true); }

export async function makeModZip(manifest, extended, uploadedIcon) {
  const encoder = new TextEncoder();
  const entries = [
    { name: "mod.json", data: encoder.encode(manifest) },
    { name: "extended.mod.json", data: encoder.encode(extended) }
  ];
  if (uploadedIcon) entries.push({ name: uploadedIcon.file.name, data: new Uint8Array(await uploadedIcon.file.arrayBuffer()) });
  const localParts = [], centralParts = [];
  let localOffset = 0;
  for (const entry of entries) {
    const name = encoder.encode(entry.name), crc = crc32(entry.data);
    const local = new Uint8Array(30 + name.length + entry.data.length), localView = new DataView(local.buffer);
    write32(localView, 0, 0x04034b50); write16(localView, 4, 20); write16(localView, 6, 0x0800); write16(localView, 8, 0); write16(localView, 12, 0x21);
    write32(localView, 14, crc); write32(localView, 18, entry.data.length); write32(localView, 22, entry.data.length); write16(localView, 26, name.length);
    local.set(name, 30); local.set(entry.data, 30 + name.length); localParts.push(local);
    const central = new Uint8Array(46 + name.length), centralView = new DataView(central.buffer);
    write32(centralView, 0, 0x02014b50); write16(centralView, 4, 20); write16(centralView, 6, 20); write16(centralView, 8, 0x0800); write16(centralView, 10, 0); write16(centralView, 14, 0x21);
    write32(centralView, 16, crc); write32(centralView, 20, entry.data.length); write32(centralView, 24, entry.data.length); write16(centralView, 28, name.length); write32(centralView, 42, localOffset);
    central.set(name, 46); centralParts.push(central); localOffset += local.length;
  }
  const centralSize = centralParts.reduce((sum, part) => sum + part.length, 0);
  const end = new Uint8Array(22), endView = new DataView(end.buffer);
  write32(endView, 0, 0x06054b50); write16(endView, 8, entries.length); write16(endView, 10, entries.length); write32(endView, 12, centralSize); write32(endView, 16, localOffset);
  return new Blob([...localParts, ...centralParts, end], { type: "application/zip" });
}

export function downloadJson(text, filename) {
  const file = new Blob([text], { type: 'application/json' }); const href = URL.createObjectURL(file);
  const link = document.createElement('a'); link.href = href; link.download = filename; link.click(); URL.revokeObjectURL(href);
}
