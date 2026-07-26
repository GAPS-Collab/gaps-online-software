#!/usr/bin/env python3
"""Standalone check: what percentage of rb_events fall into each mangling status."""
import argparse
import sys
from collections import Counter
from pathlib import Path

import gondola as gon

MANGLING_STATI = {
    gon.events.EventStatus.ChnSyncErrors: "ChnSyncErrors",
    gon.events.EventStatus.CellSyncErrors: "CellSyncErrors",
    gon.events.EventStatus.CellAndChnSyncErrors: "CellAndChnSyncErrors",
}


def read_files_txt(path: Path) -> list[str]:
    return [
        line.strip()
        for line in path.read_text().splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--file-list", default="run_lists/flight_small.txt")
    ap.add_argument("--max-files", type=int, default=None)
    args = ap.parse_args()

    files = read_files_txt(Path(args.file_list))
    if args.max_files:
        files = files[: args.max_files]

    rb_event_count = 0
    status_counts = Counter()

    for idx, f in enumerate(files, start=1):
        print(f"[{idx}/{len(files)}] {f}", file=sys.stderr, flush=True)
        reader = gon.io.TofPacketReader(
            str(f), filter=gon.packets.TofPacketType.TofEvent
        )
        for pack in reader:
            ev = gon.events.TofEvent.from_bytestream(pack.payload, 0)
            for rb in ev.rb_events:
                rb_event_count += 1
                status_counts[rb.status] += 1

    mangling_count = sum(
        status_counts[s] for s in MANGLING_STATI if s in status_counts
    )

    print(f"\ntotal rb_events: {rb_event_count}")
    print(f"total mangled rb_events: {mangling_count} "
          f"({100 * mangling_count / rb_event_count:.3f}% of all rb_events)\n")

    print("breakdown of mangling categories (% of ALL rb_events):")
    for status, name in MANGLING_STATI.items():
        c = status_counts.get(status, 0)
        pct = 100 * c / rb_event_count if rb_event_count else 0.0
        print(f"  {name:24s} {c:8d}  {pct:6.3f}%")

    print("\nbreakdown of mangling categories (% of MANGLED rb_events only):")
    for status, name in MANGLING_STATI.items():
        c = status_counts.get(status, 0)
        pct = 100 * c / mangling_count if mangling_count else 0.0
        print(f"  {name:24s} {c:8d}  {pct:6.3f}%")


if __name__ == "__main__":
    main()
