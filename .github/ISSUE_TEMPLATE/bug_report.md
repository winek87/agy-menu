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

{ _ble_edit_exec_gexec__save_lastarg "$@"; } 4>&1 5>&2 &>/dev/null

## `agy-menu --diagnose`

<!-- Wklej output tej komendy. Pomaga ustalić, które konta są wykryte i czytelne. -->
<!-- tutaj -->

## Logi / zrzuty

<!--
Jeśli błąd dotyczy renderowania — wklej screenshot.
Jeśli dotyczy `su` — wklej pełny output `su zakończył się kodem …`.
Logi błędów uruchomienia: `RUST_BACKTRACE=1 agy-menu 2>err.log`
-->

## Dodatkowy kontekst

<!-- Cokolwiek jeszcze może pomóc. -->
