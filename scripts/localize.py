#!/usr/bin/env python3
"""把 Lyricade 的上游英文界面文案替换为简体中文。

用法：在仓库根目录执行  python3 scripts/localize.py
映射表：scripts/zh-strings.json （英文原文 -> 中文译文）

作用范围严格限定在界面层（src/ui/**）与 src/util.rs 的界面文案，
不会改动写入音频文件的 ID3 标签描述、解析器关键字或数据库取值。
可重复执行（幂等）。
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAP_PATH = ROOT / "scripts" / "zh-strings.json"
STR_LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')

TARGETS = [ROOT / "src" / "ui", ROOT / "src" / "util.rs"]


def iter_files():
    for target in TARGETS:
        if target.is_dir():
            yield from sorted(target.rglob("*.rs"))
        elif target.is_file():
            yield target


def main() -> int:
    mapping = json.loads(MAP_PATH.read_text(encoding="utf-8"))
    total = 0
    touched = []

    for path in iter_files():
        text = path.read_text(encoding="utf-8")
        hits = 0

        def repl(match: re.Match[str]) -> str:
            nonlocal hits
            literal = match.group(1)
            zh = mapping.get(literal)
            if zh is None:
                return match.group(0)
            hits += 1
            return '"' + zh + '"'

        new_text = STR_LIT.sub(repl, text)
        if new_text != text:
            path.write_text(new_text, encoding="utf-8")
            touched.append((path.relative_to(ROOT), hits))
            total += hits

    for rel, hits in touched:
        print(f"  {rel}: {hits} 处")
    print(f"已应用 {total} 处替换，涉及 {len(touched)} 个文件（映射表 {len(mapping)} 条）")

    leftovers = []
    for path in iter_files():
        text = path.read_text(encoding="utf-8")
        for match in STR_LIT.finditer(text):
            if match.group(1) in mapping:
                leftovers.append((path.relative_to(ROOT), match.group(1)))

    if leftovers:
        print("警告：以下字符串仍为英文（上游可能改写了该行，需要更新映射表）：")
        for rel, literal in leftovers[:30]:
            print(f"  {rel}: {literal}")
        return 1

    print("校验通过：界面层已无待替换的英文目标字符串")
    return 0


if __name__ == "__main__":
    sys.exit(main())
