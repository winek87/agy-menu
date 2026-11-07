#!/usr/bin/env bash
# setup-repo-part2.sh — dokańcza brakujące pliki repo agy-menu
set -euo pipefail

USER_NAME="${1:-winek87}"
REAL_NAME="${2:-Filip}"

echo "→ Uzupełniam brakujące pliki dla: $USER_NAME ($REAL_NAME)"

mkdir -p .github/ISSUE_TEMPLATE .github/workflows

# ────────────────────────────────── .github/ISSUE_TEMPLATE/bug_report.md
cat << 'PYEOF' > .github/ISSUE_TEMPLATE/bug_report.md
---
name: 🐛 Bug report
about: Zgłoś błąd — coś nie działa jak powinno
title: "[BUG] "
labels: ["bug", "triage"]
assignees: []
---

## Opis błędu

<!-- Krótko: co się dzieje, czego oczekujesz. -->

## Kroki reprodukcji

1.
2.
3.

## Oczekiwane zachowanie

<!-- Co powinno się stać. -->

## Rzeczywiste zachowanie

<!-- Co się dzieje zamiast tego. Wklej błąd / stack trace jeśli jest. -->

## Środowisko

- **Wersja agy-menu**: <!-- output `agy-menu --version` lub commit hash -->
- **OS**: <!-- np. Ubuntu 24.04, macOS 15.1 -->
- **Rust**: <!-- output `rustc --version` -->
- **Terminal**: <!-- np. alacritty 0.14, kitty 0.39, tmux 3.5, foot 1.20 -->
- **`agy`**: <!-- output `agy --version` -->

## `agy-menu --diagnose`

<!-- Wklej output tej komendy. Pomaga ustalić, które konta są wykryte i czytelne. -->

