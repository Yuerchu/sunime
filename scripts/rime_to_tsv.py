"""
Convert rime-ice .dict.yaml files to a TSV file for sunime build.

Usage: uv run python scripts/rime_to_tsv.py <rime_ice_dir> <output.tsv>

Output format (tab-separated):
  code\ttext\tfreq
"""

import os
import sys


def parse_dict_yaml(path: str, base_dir: str, visited: set | None = None) -> list[tuple[str, str, int]]:
    if visited is None:
        visited = set()

    abs_path = os.path.abspath(path)
    if abs_path in visited:
        return []
    visited.add(abs_path)

    entries = []
    import_tables = []
    in_header = True

    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n\r")

            if in_header:
                stripped = line.strip()
                if stripped == "...":
                    in_header = False
                    continue

                if stripped.startswith("- ") and "import_tables" not in stripped:
                    table_name = stripped[2:].strip()
                    table_name = table_name.split("#")[0].strip()
                    if table_name:
                        import_tables.append(table_name)
                continue

            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue

            parts = stripped.split("\t")
            if len(parts) < 2:
                continue

            text = parts[0]
            code = parts[1]
            freq = 0
            if len(parts) >= 3:
                try:
                    freq = int(parts[2])
                except ValueError:
                    pass

            entries.append((code, text, freq))

    for table in import_tables:
        table_path = os.path.join(base_dir, f"{table}.dict.yaml")
        if os.path.exists(table_path):
            entries.extend(parse_dict_yaml(table_path, base_dir, visited))
        else:
            sub_dir = os.path.join(base_dir, os.path.dirname(table))
            sub_file = os.path.join(base_dir, f"{table}.dict.yaml")
            if os.path.exists(sub_file):
                entries.extend(parse_dict_yaml(sub_file, base_dir, visited))

    return entries


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <rime_ice_dir> <output.tsv>")
        sys.exit(1)

    rime_dir = sys.argv[1]
    output_path = sys.argv[2]

    main_dict = os.path.join(rime_dir, "rime_ice.dict.yaml")
    if not os.path.exists(main_dict):
        print(f"Not found: {main_dict}")
        sys.exit(1)

    print(f"Parsing {main_dict}...")
    entries = parse_dict_yaml(main_dict, rime_dir)
    print(f"  {len(entries)} raw entries")

    seen = set()
    unique = []
    for code, text, freq in entries:
        key = (code, text)
        if key not in seen:
            seen.add(key)
            unique.append((code, text, freq))

    unique.sort(key=lambda x: (x[0], -x[2]))
    print(f"  {len(unique)} unique entries")

    with open(output_path, "w", encoding="utf-8", newline="\n") as f:
        for code, text, freq in unique:
            f.write(f"{code}\t{text}\t{freq}\n")

    print(f"Written to {output_path}")


if __name__ == "__main__":
    main()
