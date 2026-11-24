#!/usr/bin/env python3
"""Wrap an engineering task in a structured, externalized-work protocol.

This compiler improves prompt organization; it cannot disable hidden reasoning,
change product tools, guarantee output limits, or validate model-specific claims.
"""
from __future__ import annotations
import argparse
from pathlib import Path
import sys

TEMPLATE = '''<context>
Target label: GPT-6 Standard / Free Web (user-specified; unverified product identity).
Execution preference: concise, direct response. Do not claim hidden reasoning is disabled.
Task source: user-provided requirements below.
</context>

<invariants>
- Follow higher-priority system, developer, safety, and tool instructions.
- Do not invent APIs, facts, test results, file contents, or telemetry. Mark assumptions.
- Preserve existing behavior and public interfaces unless the task explicitly changes them.
- If requirements conflict or essential data is missing, state the blocker and ask a focused question.
- Do not omit requested files, tests, edge cases, or implementation details; never use “... existing code ...” as a substitute for requested code.
- Keep output in chat as raw Markdown/code fences. Do not invoke Canvas or browsing unless the task requires it and the available tools permit it.
- Use tools only when available and appropriate. Report what was actually executed; never imply unrun verification passed.
</invariants>

<pre_computation_gate>
Before final deliverables, emit a compact, user-visible planning artifact (not private chain-of-thought):
1. Goal and acceptance criteria.
2. Assumptions and unresolved questions.
3. Invariants / edge cases / failure conditions.
4. Ordered implementation or analysis steps.
Do not reveal private internal reasoning. Keep this artifact brief and evidence-oriented.
</pre_computation_gate>

<output_contract>
Phase 1 — Pre-flight: the planning artifact above; identify files/tools needed.
Phase 2 — Core result: deliver the requested solution completely. For code, include full requested files or precise SEARCH/REPLACE patches with unique context. Separate files clearly.
Phase 3 — Self-audit: check each acceptance criterion, edge cases, consistency, and whether verification actually ran. List residual uncertainty.
Use headings and scoped lists. If output limits prevent completion, stop at a file/sentence boundary and state exactly what remains; do not silently summarize or elide.
</output_contract>

<task>
{task}
</task>
'''

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('task_file', nargs='?', help='UTF-8 task file; if omitted, read stdin')
    ap.add_argument('-o', '--output', help='write compiled prompt to this path (default stdout)')
    args = ap.parse_args()
    task = Path(args.task_file).read_text(encoding='utf-8') if args.task_file else sys.stdin.read()
    if not task.strip():
        ap.error('task must not be empty')
    result = TEMPLATE.format(task=task.strip())
    if args.output:
        Path(args.output).write_text(result, encoding='utf-8')
    else:
        sys.stdout.write(result)
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
