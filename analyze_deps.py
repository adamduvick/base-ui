#!/usr/bin/env python3
"""
Dependency analysis script for porting Base UI React to Leptos.

Statically analyzes the Base UI React codebase to produce a dependency map
in topological order, showing which items must be ported first, which can
be worked in parallel, and tracking progress.

Usage:
    python analyze_deps.py                              # Analyze and print report
    python analyze_deps.py --set-status utils/empty done  # Update status
    python analyze_deps.py --ready                       # Show only ready items
"""

import argparse
import json
import os
import re
import sys
from collections import defaultdict, deque
from datetime import datetime, timezone
from pathlib import Path

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

REPO_ROOT = Path(__file__).resolve().parent
UTILS_SRC = REPO_ROOT / "packages" / "utils" / "src"
REACT_SRC = REPO_ROOT / "packages" / "react" / "src"
STATUS_FILE = REPO_ROOT / "port-status.json"

VALID_STATUSES = ("not_started", "in_progress", "done")

SOURCE_EXTENSIONS = {".ts", ".tsx"}
TEST_PATTERNS = {".test.tsx", ".test.ts", ".spec.tsx", ".spec.ts"}

# Import/export-from regex (applied after comment stripping)
IMPORT_RE = re.compile(
    r"""(?:import|export)\s.*?from\s*['"]([^'"]+)['"]""", re.DOTALL
)

# Comment stripping
BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
LINE_COMMENT_RE = re.compile(r"//[^\n]*")

# ANSI colors
COLORS = {
    "reset": "\033[0m",
    "bold": "\033[1m",
    "dim": "\033[2m",
    "red": "\033[31m",
    "green": "\033[32m",
    "yellow": "\033[33m",
    "blue": "\033[34m",
    "magenta": "\033[35m",
    "cyan": "\033[36m",
    "white": "\033[37m",
}

# Detect if we should use colors
USE_COLOR = sys.stdout.isatty()


def c(color, text):
    if USE_COLOR:
        return f"{COLORS[color]}{text}{COLORS['reset']}"
    return text


# ---------------------------------------------------------------------------
# Step 1: Discover logical units
# ---------------------------------------------------------------------------


def is_test_file(path):
    name = path.name
    return any(name.endswith(pat) for pat in TEST_PATTERNS)


def is_source_file(path):
    return path.suffix in SOURCE_EXTENSIONS and not is_test_file(path)


def discover_units():
    """
    Returns dict: unit_name -> list of source file Paths.

    Grouping rules:
    - packages/utils/src/store/ -> "utils/store" (whole directory)
    - packages/utils/src/<file>.ts -> "utils/<stem>" (one file per unit)
    - packages/react/src/<dir>/ -> "react/<dir>" (each top-level dir)
    """
    units = {}

    # --- utils ---
    store_dir = UTILS_SRC / "store"

    # utils/store: whole directory
    if store_dir.is_dir():
        files = [f for f in store_dir.rglob("*") if f.is_file() and is_source_file(f)]
        if files:
            units["utils/store"] = sorted(files)

    # utils/<file>: one per top-level .ts file
    for f in sorted(UTILS_SRC.iterdir()):
        if f.is_file() and is_source_file(f):
            stem = f.stem
            units[f"utils/{stem}"] = [f]

    # --- react ---
    for d in sorted(REACT_SRC.iterdir()):
        if d.is_dir():
            files = [
                f for f in d.rglob("*") if f.is_file() and is_source_file(f)
            ]
            if files:
                units[f"react/{d.name}"] = sorted(files)

    return units


# ---------------------------------------------------------------------------
# Step 2: Extract imports from source files
# ---------------------------------------------------------------------------


def strip_comments(source):
    source = BLOCK_COMMENT_RE.sub("", source)
    source = LINE_COMMENT_RE.sub("", source)
    return source


def extract_imports(filepath):
    """Return list of raw import specifiers from a source file."""
    try:
        source = filepath.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return []
    cleaned = strip_comments(source)
    return IMPORT_RE.findall(cleaned)


# ---------------------------------------------------------------------------
# Step 3: Resolve import specifiers to unit names
# ---------------------------------------------------------------------------


def resolve_specifier_to_unit(specifier, importing_file, units_by_name):
    """
    Resolve an import specifier to a unit name, or None if external.

    Handles:
    - @base-ui/utils/store -> utils/store
    - @base-ui/utils/useTimeout -> utils/useTimeout
    - Relative paths like ../../dialog/root/DialogRootContext -> react/dialog
    """

    # --- @base-ui/utils imports ---
    if specifier.startswith("@base-ui/utils"):
        # @base-ui/utils/store -> utils/store
        # @base-ui/utils/useTimeout -> utils/useTimeout
        parts = specifier.split("/")
        if len(parts) >= 3:
            sub = parts[2]  # e.g. "store", "useTimeout"
            unit_name = f"utils/{sub}"
            if unit_name in units_by_name:
                return unit_name
        return None

    # --- @base-ui/react imports (less common but possible) ---
    if specifier.startswith("@base-ui/react"):
        parts = specifier.split("/")
        if len(parts) >= 3:
            component = parts[2]
            unit_name = f"react/{component}"
            if unit_name in units_by_name:
                return unit_name
        return None

    # --- Relative imports ---
    if specifier.startswith("."):
        # Resolve relative to the importing file's directory
        importing_dir = importing_file.parent
        resolved = (importing_dir / specifier).resolve()

        # Check if it falls under utils/src/
        try:
            rel_to_utils = resolved.relative_to(UTILS_SRC.resolve())
            # Check if it's in store/
            parts = rel_to_utils.parts
            if parts and parts[0] == "store":
                return "utils/store"
            # Otherwise it's a top-level util file
            stem = parts[0] if parts else None
            if stem:
                # Remove extension if present
                stem = Path(stem).stem
                unit_name = f"utils/{stem}"
                if unit_name in units_by_name:
                    return unit_name
            return None
        except ValueError:
            pass

        # Check if it falls under react/src/
        try:
            rel_to_react = resolved.relative_to(REACT_SRC.resolve())
            parts = rel_to_react.parts
            if parts:
                component = parts[0]
                unit_name = f"react/{component}"
                if unit_name in units_by_name:
                    return unit_name
            return None
        except ValueError:
            pass

        return None

    # External package (react, @floating-ui/*, etc.) -> skip
    return None


def build_dependency_graph(units):
    """
    Returns dict: unit_name -> set of unit_names it depends on.
    """
    deps = {name: set() for name in units}

    for unit_name, files in units.items():
        for filepath in files:
            specifiers = extract_imports(filepath)
            for spec in specifiers:
                target = resolve_specifier_to_unit(spec, filepath, units)
                if target and target != unit_name:
                    deps[unit_name].add(target)

    return deps


# ---------------------------------------------------------------------------
# Step 4: Topological sort with tiers (Kahn's algorithm)
# ---------------------------------------------------------------------------


def topological_sort_tiers(units, deps):
    """
    Returns (tiers, cycles) where:
    - tiers: list of lists, each list is a set of unit names at that tier
    - cycles: list of unit names involved in cycles (if any)
    """
    in_degree = {name: 0 for name in units}
    reverse_deps = defaultdict(set)

    for name, dep_set in deps.items():
        for dep in dep_set:
            if dep in units:  # only count internal deps
                in_degree[name] += 1
                reverse_deps[dep].add(name)

    # BFS
    queue = deque()
    for name in sorted(units.keys()):
        if in_degree[name] == 0:
            queue.append(name)

    tiers = []
    visited = set()

    while queue:
        # All items in current queue form one tier
        current_tier = sorted(queue)
        tiers.append(current_tier)
        next_queue = deque()

        for name in current_tier:
            visited.add(name)
            for dependent in sorted(reverse_deps[name]):
                in_degree[dependent] -= 1
                if in_degree[dependent] == 0:
                    next_queue.append(dependent)

        queue = next_queue

    # Detect cycles: any unvisited nodes
    cycles = sorted(set(units.keys()) - visited)

    return tiers, cycles


# ---------------------------------------------------------------------------
# Step 5: Status persistence
# ---------------------------------------------------------------------------


def load_status():
    """Load status dict from port-status.json, or empty dict."""
    if STATUS_FILE.exists():
        try:
            data = json.loads(STATUS_FILE.read_text(encoding="utf-8"))
            # Extract unit statuses from the tiers structure
            statuses = {}
            for tier in data.get("tiers", []):
                for unit in tier.get("units", []):
                    name = unit.get("name")
                    status = unit.get("status", "not_started")
                    if name:
                        statuses[name] = status
            return statuses
        except (json.JSONDecodeError, KeyError):
            return {}
    return {}


def save_status(units, deps, tiers, cycles, statuses):
    """Write the full port-status.json."""
    # Compute readiness per unit
    def is_ready(name):
        if statuses.get(name) == "done":
            return False  # already done, not "ready to work on"
        for dep in deps.get(name, set()):
            if statuses.get(dep) != "done":
                return False
        return True

    # Representative path for each unit
    def unit_path(name):
        files = units.get(name, [])
        if not files:
            return ""
        # Return the path relative to repo root
        try:
            return str(files[0].relative_to(REPO_ROOT))
        except ValueError:
            return str(files[0])

    total = len(units)
    done_count = sum(1 for s in statuses.values() if s == "done")
    in_progress_count = sum(1 for s in statuses.values() if s == "in_progress")
    ready_count = sum(1 for name in units if is_ready(name))

    tier_data = []
    for i, tier_names in enumerate(tiers):
        tier_units = []
        for name in tier_names:
            tier_units.append(
                {
                    "name": name,
                    "path": unit_path(name),
                    "dependencies": sorted(deps.get(name, set())),
                    "status": statuses.get(name, "not_started"),
                    "ready": is_ready(name),
                }
            )
        tier_data.append({"tier": i, "units": tier_units})

    # Add cycle tier if present
    if cycles:
        cycle_units = []
        for name in cycles:
            cycle_units.append(
                {
                    "name": name,
                    "path": unit_path(name),
                    "dependencies": sorted(deps.get(name, set())),
                    "status": statuses.get(name, "not_started"),
                    "ready": is_ready(name),
                    "in_cycle": True,
                }
            )
        tier_data.append({"tier": "cycle", "units": cycle_units})

    output = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "summary": {
            "total": total,
            "done": done_count,
            "in_progress": in_progress_count,
            "ready": ready_count,
        },
        "tiers": tier_data,
    }

    STATUS_FILE.write_text(
        json.dumps(output, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


# ---------------------------------------------------------------------------
# Step 6: Console report
# ---------------------------------------------------------------------------


def print_report(units, deps, tiers, cycles, statuses, ready_only=False):
    """Print a color-coded tiered report to stdout."""

    def is_ready(name):
        if statuses.get(name) == "done":
            return False
        for dep in deps.get(name, set()):
            if statuses.get(dep) != "done":
                return False
        return True

    def status_str(name):
        s = statuses.get(name, "not_started")
        if s == "done":
            return c("green", "DONE")
        elif s == "in_progress":
            return c("yellow", "IN PROGRESS")
        else:
            return c("dim", "not started")

    total = len(units)
    done_count = sum(1 for s in statuses.values() if s == "done")
    in_progress_count = sum(1 for s in statuses.values() if s == "in_progress")
    ready_count = sum(1 for name in units if is_ready(name))

    # Header
    print()
    print(c("bold", "Base UI Dependency Analysis for Leptos Port"))
    print(c("bold", "=" * 46))
    print(
        f"  Total units: {c('bold', str(total))}  |  "
        f"Done: {c('green', str(done_count))}  |  "
        f"In progress: {c('yellow', str(in_progress_count))}  |  "
        f"Ready: {c('cyan', str(ready_count))}"
    )
    print()

    for i, tier_names in enumerate(tiers):
        tier_items = tier_names
        if ready_only:
            tier_items = [n for n in tier_names if is_ready(n)]
            if not tier_items:
                continue

        print(c("bold", f"--- Tier {i} ({len(tier_items)} units) ---"))

        for name in tier_items:
            dep_list = sorted(deps.get(name, set()))
            ready_marker = ""
            if is_ready(name):
                ready_marker = c("cyan", " [READY]")

            status = status_str(name)
            print(f"  {c('bold', name):40s}  {status}{ready_marker}")
            if dep_list and not ready_only:
                dep_strs = []
                for d in dep_list:
                    if statuses.get(d) == "done":
                        dep_strs.append(c("green", d))
                    else:
                        dep_strs.append(c("dim", d))
                print(f"    deps: {', '.join(dep_strs)}")

        print()

    # Cycles
    if cycles:
        print(c("red", c("bold", "=== CYCLES DETECTED ===")))
        print(c("red", "The following units are involved in circular dependencies:"))
        for name in cycles:
            dep_list = sorted(deps.get(name, set()))
            cycle_deps = [d for d in dep_list if d in set(cycles)]
            print(f"  {c('red', name)}")
            if cycle_deps:
                print(f"    cycle with: {', '.join(c('red', d) for d in cycle_deps)}")
        print()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    parser = argparse.ArgumentParser(
        description="Dependency analysis for Base UI -> Leptos port"
    )
    parser.add_argument(
        "--set-status",
        nargs=2,
        metavar=("UNIT", "STATUS"),
        help=f"Set a unit's port status ({', '.join(VALID_STATUSES)})",
    )
    parser.add_argument(
        "--ready",
        action="store_true",
        help="Show only units that are ready to work on",
    )
    args = parser.parse_args()

    # Discover units
    units = discover_units()

    # Build dependency graph
    deps = build_dependency_graph(units)

    # Topological sort
    tiers, cycles = topological_sort_tiers(units, deps)

    # Load existing statuses
    statuses = load_status()

    # Ensure all units have a status
    for name in units:
        if name not in statuses:
            statuses[name] = "not_started"

    # Clean up statuses for units that no longer exist
    statuses = {k: v for k, v in statuses.items() if k in units}

    # Handle --set-status
    if args.set_status:
        unit_name, new_status = args.set_status
        if unit_name not in units:
            print(f"Error: unknown unit '{unit_name}'", file=sys.stderr)
            print(f"Available units:", file=sys.stderr)
            for name in sorted(units.keys()):
                print(f"  {name}", file=sys.stderr)
            sys.exit(1)
        if new_status not in VALID_STATUSES:
            print(
                f"Error: invalid status '{new_status}'. Must be one of: {', '.join(VALID_STATUSES)}",
                file=sys.stderr,
            )
            sys.exit(1)
        statuses[unit_name] = new_status
        save_status(units, deps, tiers, cycles, statuses)
        print(f"Set {unit_name} -> {new_status}")
        return

    # Save and print
    save_status(units, deps, tiers, cycles, statuses)
    print_report(units, deps, tiers, cycles, statuses, ready_only=args.ready)


if __name__ == "__main__":
    main()
