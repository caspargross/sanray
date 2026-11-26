#!/usr/bin/env python3
"""Minimal parser for Delphi/Borland binary DFM resources (TPF0 format).

Grammar (reconstructed from community write-ups + the classic Delphi
TValueType enum; see docs/REVERSE_ENGINEERING.md for sources):

    File              := "TPF0" Object
    Object            := Symbol(class) Symbol(name) PropertyList NestedObjectList
    PropertyList      := (Symbol(propname) Value)* Symbol("")
    NestedObjectList  := (Object)* Symbol("")   -- empty class name terminates
    Symbol / String   := byte(length) + raw bytes (no terminator)

Value := tag:u8 followed by tag-specific payload. Known tags (TValueType):
    0  vaNull        (list/collection terminator; 0 bytes)
    1  vaList        repeated Value until a vaNull(0x00) tag
    2  vaInt8        1 byte, signed
    3  vaInt16       2 bytes, signed, LE
    4  vaInt32       4 bytes, signed, LE
    5  vaExtended    10 bytes (80-bit x87 extended float)
    6  vaString      byte(len) + bytes
    7  vaIdent       byte(len) + bytes (identifier/enum value)
    8  vaFalse       0 bytes
    9  vaTrue        0 bytes
    10 vaBinary      u32(len) + bytes
    11 vaSet         (byte(len)+bytes)* terminated by zero-length entry
    12 vaLString     u32(len) + bytes
    13 vaNil         0 bytes
    14 vaCollection  repeated collection items until 0x00; each item is
                     optionally prefixed by an index tag/value then a
                     PropertyList
    15 vaSingle      4 bytes float
    16 vaCurrency    8 bytes (int64, scaled)
    17 vaDate        8 bytes double (TDateTime)
    18 vaWString     u32(len) + utf-16le bytes
    19 vaInt64       8 bytes signed
    20 vaUTF8String  u32(len) + utf-8 bytes
    21 vaDouble      8 bytes double

This is reconstructed, not from official docs -- if parsing goes out of
sync, self_check() will report leftover/short bytes so we notice rather
than silently emitting garbage.
"""
import os
import struct
import sys
import json

DEBUG = os.environ.get("DFM_DEBUG") == "1"


class Reader:
    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0

    def eof(self):
        return self.pos >= len(self.data)

    def u8(self):
        v = self.data[self.pos]
        self.pos += 1
        return v

    def bytes(self, n):
        v = self.data[self.pos:self.pos + n]
        if len(v) != n:
            raise EOFError(f"wanted {n} bytes at {self.pos}, got {len(v)}")
        self.pos += n
        return v

    def i8(self):
        return struct.unpack("<b", self.bytes(1))[0]

    def i16(self):
        return struct.unpack("<h", self.bytes(2))[0]

    def i32(self):
        return struct.unpack("<i", self.bytes(4))[0]

    def i64(self):
        return struct.unpack("<q", self.bytes(8))[0]

    def u32(self):
        return struct.unpack("<I", self.bytes(4))[0]

    def f32(self):
        return struct.unpack("<f", self.bytes(4))[0]

    def f64(self):
        return struct.unpack("<d", self.bytes(8))[0]

    def shortstr(self):
        n = self.u8()
        return self.bytes(n).decode("latin1")


def read_value(r: Reader):
    tagpos = r.pos
    tag = r.u8()
    if DEBUG:
        print(f"    val @{tagpos}: tag={tag}", file=sys.stderr)
    if tag == 0:   # vaNull
        return None
    if tag == 1:   # vaList
        items = []
        while True:
            peek = r.data[r.pos]
            if peek == 0:
                r.pos += 1
                break
            items.append(read_value(r))
        return items
    if tag == 2:
        return r.i8()
    if tag == 3:
        return r.i16()
    if tag == 4:
        return r.i32()
    if tag == 5:
        raw = r.bytes(10)
        return f"<extended80 {raw.hex()}>"
    if tag == 6:
        return r.shortstr()
    if tag == 7:
        return ("ident", r.shortstr())
    if tag == 8:
        return False
    if tag == 9:
        return True
    if tag == 10:
        n = r.u32()
        return ("binary", r.bytes(n))
    if tag == 11:
        items = []
        while True:
            n = r.u8()
            if n == 0:
                break
            items.append(r.bytes(n).decode("latin1"))
        return ("set", items)
    if tag == 12:
        n = r.u32()
        return r.bytes(n).decode("latin1", errors="replace")
    if tag == 13:
        return None
    if tag == 14:
        # Each item is prefixed by a marker byte: 0x00 ends the collection,
        # 0x01 introduces one item, itself a plain PropertyList.
        items = []
        while True:
            marker = r.u8()
            if marker == 0:
                break
            if marker != 1:
                raise ValueError(f"unexpected collection item marker {marker} at {r.pos - 1}")
            props = read_properties(r)
            items.append(props)
        return ("collection", items)
    if tag == 15:
        return r.f32()
    if tag == 16:
        return r.i64() / 10000.0
    if tag == 17:
        return r.f64()
    if tag == 18:
        # length is a CHARACTER count, not a byte count
        n_chars = r.u32()
        return r.bytes(n_chars * 2).decode("utf-16-le")
    if tag == 19:
        return r.i64()
    if tag == 20:
        n = r.u32()
        return r.bytes(n).decode("utf-8", errors="replace")
    if tag == 21:
        return r.f64()
    raise ValueError(f"unknown value tag {tag} at pos {r.pos - 1}")


def read_properties(r: Reader):
    props = []
    while True:
        start = r.pos
        name = r.shortstr()
        if name == "":
            break
        if DEBUG:
            print(f"  prop @{start}: {name}", file=sys.stderr)
        val = read_value(r)
        props.append((name, val))
    return props


def read_object(r: Reader):
    start = r.pos
    cls = r.shortstr()
    name = r.shortstr()
    if DEBUG:
        print(f"object @{start}: {cls} / {name}", file=sys.stderr)
    props = read_properties(r)
    children = []
    while True:
        peek_start = r.pos
        child_cls = r.shortstr()
        if child_cls == "":
            break
        r.pos = peek_start
        children.append(read_object(r))
    return {"class": cls, "name": name, "props": props, "children": children}


def parse_dfm(data: bytes):
    assert data[:4] == b"TPF0", "not a TPF0 resource"
    r = Reader(data)
    r.pos = 4
    obj = read_object(r)
    leftover = len(data) - r.pos
    return obj, leftover


def dump(obj, indent=0, out=None):
    pad = "  " * indent
    print(f"{pad}object {obj['class']} as {obj['name']}", file=out)
    for k, v in obj["props"]:
        print(f"{pad}  {k} = {v!r}", file=out)
    for c in obj["children"]:
        dump(c, indent + 1, out=out)


if __name__ == "__main__":
    path = sys.argv[1]
    with open(path, "rb") as f:
        data = f.read()
    obj, leftover = parse_dfm(data)
    print(f"# parsed {path}: {len(data)} bytes, leftover={leftover}", file=sys.stderr)
    if len(sys.argv) > 2 and sys.argv[2] == "--json":

        def to_jsonable(v):
            if isinstance(v, tuple):
                return {"_tag": v[0], "value": to_jsonable(v[1])}
            if isinstance(v, bytes):
                return v.hex()
            if isinstance(v, list):
                return [to_jsonable(x) for x in v]
            if isinstance(v, dict):
                return {k: to_jsonable(x) for k, x in v.items()}
            return v

        def obj_to_jsonable(o):
            return {
                "class": o["class"],
                "name": o["name"],
                "props": {k: to_jsonable(v) for k, v in o["props"]},
                "children": [obj_to_jsonable(c) for c in o["children"]],
            }

        print(json.dumps(obj_to_jsonable(obj), indent=2))
    else:
        dump(obj)
