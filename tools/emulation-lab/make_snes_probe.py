#!/usr/bin/env python3
"""Generate an original, freely redistributable SNES test cartridge (no game assets).

The screen is red. WRAM $0000/$0001 hold auto-read joypad bits, $0002
counts frames; SRAM $0000 mirrors the counter. Used by the real-core test.
"""
from pathlib import Path
import sys


def rom(pal=False, hires=False):
    code = bytearray()
    labels = {}
    branches = []
    def emit(*data): code.extend(data)
    def label(name): labels[name] = len(code)
    def branch(op, name):
        emit(op, 0)
        branches.append((len(code) - 1, name))
    def lda(value): emit(0xA9, value)
    def sta(address): emit(0x8D, address & 255, address >> 8)
    def stz(address): emit(0x9C, address & 255, address >> 8)
    emit(0x78, 0xD8)  # SEI, CLD; reset starts in emulation mode, 8-bit A/X.
    lda(0x80); sta(0x2100)
    stz(0x4200); stz(0x2121)
    lda(31); sta(0x2122); stz(0x2122)
    stz(0x2105); stz(0x212C)
    lda(0x0d if hires else 0); sta(0x2133)  # Interlace, overscan, pseudo-hires.
    lda(15); sta(0x2100)
    lda(1); sta(0x4200)  # Automatic controller reading, no NMI.
    stz(0); stz(1); stz(2)
    label('end'); emit(0xAD, 0x12, 0x42); branch(0x30, 'end')
    label('start'); emit(0xAD, 0x12, 0x42); branch(0x10, 'start')
    # Auto-read completes shortly after vblank begins. The preceding frame's
    # result is also valid, so integration tests hold each input for 4 frames.
    label('joy'); emit(0xAD, 0x12, 0x42, 0x29, 1); branch(0xD0, 'joy')
    emit(0xAD, 0x18, 0x42); sta(0)
    emit(0xAD, 0x19, 0x42); sta(1)
    emit(0xEE, 2, 0, 0xAD, 2, 0, 0x8F, 0, 0, 0x70)
    address = 0x8000 + labels['end']
    emit(0x4C, address & 255, address >> 8)
    for at, name in branches:
        offset = labels[name] - (at + 1)
        assert -128 <= offset <= 127
        code[at] = offset & 255
    result = bytearray([0xEA] * 0x8000)
    result[:len(code)] = code
    result[0x7FC0:0x7FD5] = b'SLOT SNES INPUT PROBE'
    result[0x7FD5:0x7FDC] = bytes([0x20, 2, 5, 3, 2 if pal else 1, 0, 0])
    result[0x7FDC:0x7FE0] = bytes([255, 255, 0, 0])
    for at in [0x7FE4, 0x7FE6, 0x7FE8, 0x7FEA, 0x7FEE, 0x7FF4, 0x7FF8, 0x7FFA, 0x7FFC, 0x7FFE]:
        result[at:at+2] = bytes([0, 0x80])
    checksum = sum(result) & 65535
    result[0x7FDC:0x7FDE] = (checksum ^ 65535).to_bytes(2, 'little')
    result[0x7FDE:0x7FE0] = checksum.to_bytes(2, 'little')
    return result


if __name__ == '__main__':
    target = Path(sys.argv[1])
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(rom('--pal' in sys.argv[2:], '--hires' in sys.argv[2:]))
    print(target)
