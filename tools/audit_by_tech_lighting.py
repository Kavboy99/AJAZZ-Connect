"""Offline audit of one pinned vendor PE and its profile. Never loads vendor code or opens HID."""
from __future__ import annotations

import argparse
import configparser
import hashlib
import json
import struct
from pathlib import Path

EXPECTED_SHA256 = "fc7109b0467602a7624b2a83402bae12255ad5ad2e9483a2c4750c584231c06c"


class PeImage:
    def __init__(self, path: Path):
        self._data = path.read_bytes()
        self.sha256 = hashlib.sha256(self._data).hexdigest()
        if self.sha256 != EXPECTED_SHA256:
            raise ValueError("This audit applies only to the pinned OemDrv.exe build")
        pe = struct.unpack_from("<I", self._data, 0x3C)[0]
        if self._data[pe:pe + 4] != b"PE\0\0":
            raise ValueError("Missing PE header")
        count = struct.unpack_from("<H", self._data, pe + 6)[0]
        optional_size = struct.unpack_from("<H", self._data, pe + 20)[0]
        optional = pe + 24
        if struct.unpack_from("<H", self._data, optional)[0] != 0x10B:
            raise ValueError("Expected PE32")
        self._base = struct.unpack_from("<I", self._data, optional + 28)[0]
        self._sections = []
        for index in range(count):
            at = optional + optional_size + 40 * index
            _, rva, raw_size, raw = struct.unpack_from("<IIII", self._data, at + 8)
            self._sections.append((rva, raw_size, raw))

    def read(self, address: int, length: int) -> bytes:
        rva = address - self._base
        for start, size, raw in self._sections:
            if start <= rva and rva + length <= start + size:
                result = self._data[raw + rva - start:raw + rva - start + length]
                if len(result) == length:
                    return result
        raise ValueError(f"Address outside file-backed PE data: {address:#x}")

    def require(self, address: int, expected_hex: str, meaning: str) -> dict:
        expected = bytes.fromhex(expected_hex)
        if self.read(address, len(expected)) != expected:
            raise ValueError(f"Instruction mismatch at {address:#x}")
        return {"address": f"0x{address:08X}", "bytes": expected.hex(), "meaning": meaning}


def audit(exe: Path, profile: Path) -> dict:
    image = PeImage(exe)
    for address, name in [(0x5FCD54, "SpeedHW"), (0x5FCD94, "LightHW")]:
        if image.read(address, len(name) * 2 + 2) != (name + "\0").encode("utf-16le"):
            raise ValueError(f"Missing pinned profile key: {name}")
    checks = [
        (0x410CE6, "6854cd5f00", "Pass SpeedHW string to the profile reader"),
        (0x410D0B, "8dbd502e0000", "Load SpeedHW array destination: object + 0x2E50"),
        (0x410E00, "6894cd5f00", "Pass LightHW string to the profile reader"),
        (0x410E29, "8dbd702e0000", "Load LightHW array destination: object + 0x2E70"),
        (0x48CE81, "8d7b3b", "EDI starts at config + 0x3B (second byte of effect 1)"),
        (0x48CE9C, "8a9430702e0000", "DL receives LightHW value from table + 0x2E70"),
        (0x48CEA3, "8a8c31502e0000", "CL receives SpeedHW value from table + 0x2E50"),
        (0x48CEB7, "240f", "Mask the color slot into AL's low nibble"),
        (0x48CEB9, "c0e104", "Shift speed CL left by four bits"),
        (0x48CEBC, "0ac1", "Combine speed and color slot"),
        (0x48CEBE, "8807", "Store packed speed/slot in the second byte"),
        (0x48CEC4, "8857ff", "Store brightness DL in the first byte"),
        (0x48CECF, "83c702", "Advance the output pair by two bytes"),
        (0x48D787, "8b51fc", "Load a source color DWORD"),
        (0x48D78C, "88941cd5000000", "Copy source bits 0..7 to RGBIndex[0]"),
        (0x48D79C, "c1eb08", "Extract source bits 8..15"),
        (0x48D7A1, "889c2cd5000000", "Copy that byte to RGBIndex[1]"),
        (0x48D7B1, "c1ea10", "Extract source bits 16..23"),
        (0x48D7B4, "88941cd5000000", "Copy that byte to RGBIndex[2]"),
    ]
    evidence = [image.require(*check) for check in checks]
    ini = configparser.ConfigParser(strict=False, allow_no_value=True)
    ini.read(profile, encoding="utf-8-sig")
    opt = ini["OPT"]
    lists = {key: [int(v.strip(), 0) for v in opt[key].split(",")]
             for key in ["LightHW", "SpeedHW", "RGBIndex"]}
    if opt["Fw"].strip() != "24" or lists != {
        "LightHW": [0, 1, 2, 3, 4], "SpeedHW": [0, 1, 2, 3, 4], "RGBIndex": [0, 1, 2]
    }:
        raise ValueError("Profile differs from the audited hardware mapping")
    return {
        "scope": "Static PE bytes and text profile only; no vendor execution or device IO",
        "exe_sha256": image.sha256,
        "profile_sha256": hashlib.sha256(profile.read_bytes()).hexdigest(),
        "profile_tables": lists,
        "layout": "config[0x38 + 2*effect] = brightness; next byte = speed<<4 | color_slot",
        "checks_passed": len(evidence),
        "evidence": evidence,
        "limits": ["Does not execute the firmware", "Does not measure PWM depth or optical output",
                   "Intermediate RGB bytes in storage do not establish an optical transfer function"],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("exe", type=Path)
    parser.add_argument("profile", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = json.dumps(audit(args.exe, args.profile), indent=2)
    if args.output:
        args.output.write_text(result + "\n", encoding="utf-8")
    else:
        print(result)


if __name__ == "__main__":
    main()
