#!/usr/bin/env bash
# setup-repo.sh — tworzy pliki repo agy-menu
set -euo pipefail

USER_NAME="${1:-winek87}"
REAL_NAME="${2:-Filip}"
YEAR="${3:-$(date +%Y)}"

if [[ "$USER_NAME" == "twoj-user" ]]; then
  echo "Użycie: $0 <github-user> [imię-nazwisko] [rok]"
  echo "  np.: $0 piotrkowalski 'Piotr Kowalski' 2025"
  exit 1
fi

echo "→ Tworzę pliki dla: $USER_NAME ($REAL_NAME, $YEAR)"

# ──────────────────────────────────────────────────────────────── .gitignore
cat << 'PYEOF' > .gitignore
# Rust
/target/
**/*.rs.bk
Cargo.lock.bak

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# lokalne konfiguracje
.env
.env.local
*.local.toml
PYEOF

# ──────────────────────────────────────────────────────────────── rust-toolchain.toml
cat << 'PYEOF' > rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
PYEOF

# ──────────────────────────────────────────────────────────────── LICENSE
cat << PYEOF > LICENSE
MIT License

Copyright (c) ${YEAR} ${REAL_NAME}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
PYEOF

# ──────────────────────────────────────────────────────────────── README.md
cat << PYEOF > README.md
# agy-menu

[![CI](https://github.com/${USER_NAME}/agy-menu/actions/workflows/ci.yml/badge.svg)](https://github.com/${USER_NAME}/agy-menu/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/${USER_NAME}/agy-menu?logo=github)](https://github.com/${USER_NAME}/agy-menu/releases)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey)]()

Interaktywne TUI do przeglądania, filtrowania i wznawiania rozmów **AGY** — z wielu kont użytkowników jednocześnie.

## Funkcje

- 📋 **Lista rozmów** z tytułem, datą i liczbą kroków
- 🔍 **Wyszukiwanie** po tytule i ścieżce workspace
- 👥 **Wielu użytkowników** — auto-skan \`/home/*\`, \`/root\`, \`/Users/*\` + ręczne \`--user\`
- 📄 **Panel szczegółów** — ostatnia wiadomość, lista raportów \`.md\`, workspace URIs
- 📜 **Pełny transkrypt** sesji (\`transcript.jsonl\`) z wirtualizowanym renderowaniem
- 🚀 **Wznawianie sesji** — \`Enter\` uruchamia \`agy --conversation <id>\`; dla innego usera przez \`su\`
- ⚡ **Responsywność** przy dużych transkryptach (10 000+ linii)

## Instalacja

\`\`\`bash
git clone https://github.com/${USER_NAME}/agy-menu.git
cd agy-menu
cargo build --release
sudo install -m 755 target/release/agy-menu /usr/local/bin/agy-menu
\`\`\`

Wymaga **Rust ≥ 1.88** (\`rustup update stable\`).

## Użycie

\`\`\`bash
# domyślnie: bieżący użytkownik + auto-skan /home/*, /root, /Users/*
agy-menu

# dodatkowi użytkownicy przez env
AGY_EXTRA_USERS="ala:/home/ala/.agyw/shared/cache,bob:/home/bob/.agyw/shared/cache" agy-menu

# lub przez CLI (wielokrotne)
agy-menu \\
  --user ala:/home/ala/.agyw/shared/cache \\
  --user bob:/home/bob/.agyw/shared/cache

# diagnostyka: kto wykryty, kto czytelny, gotowe komendy ACL
agy-menu --diagnose
\`\`\`

## Klawisze

### Lista

| Klawisz | Akcja |
|---------|-------|
| \`↑\` \`↓\` \`j\` \`k\` | nawigacja |
| \`Home\` / \`[\` | początek listy |
| \`End\` / \`]\` | koniec listy |
| \`Ctrl+Home\` | początek panelu szczegółów |
| \`Ctrl+End\` | koniec panelu szczegółów |
| \`PgUp\` / \`PgDn\` | przewiń szczegóły o ekran |
| \`/\` | szukaj |
| \`U\` | zmień użytkownika (wszyscy → ala → bob → …) |
| \`T\` | pokaż transkrypt |
| \`Enter\` | wybierz i uruchom \`agy\` |
| \`Ctrl+L\` | wyczyść filtr |
| \`Q\` / \`Ctrl+C\` | wyjdź |

### Szukanie

| Klawisz | Akcja |
|---------|-------|
| litery | wpisuj |
| \`Backspace\` | usuń znak |
| \`Ctrl+U\` | wyczyść cały wpis |
| \`Enter\` | zatwierdź |
| \`Ctrl+C\` | wyjdź |

### Transkrypt

| Klawisz | Akcja |
|---------|-------|
| \`↑\` \`↓\` \`j\` \`k\` | przewiń linię |
| \`PgUp\` / \`PgDn\` | przewiń ekran |
| \`Home\` / \`[\` | początek |
| \`End\` / \`]\` | koniec |
| \`B\` / \`T\` | powrót do listy |
| \`Q\` / \`Ctrl+C\` | wyjdź |

## Wiele użytkowników

\`agy-menu\` automatycznie wykrywa cache AGY w:

1. \`\$HOME/.agyw/shared/cache\` (bieżący user — zawsze)
2. \`/root/.agyw/shared/cache\` (jeśli czytelne lub jako \`(?)\`)
3. \`/home/*/.agyw/shared/cache\` (Linux)
4. \`/Users/*/.agyw/shared/cache\` (macOS)
5. \`AGY_EXTRA_USERS="nazwa:ścieżka,..."\` (env)
6. \`--user nazwa:ścieżka\` (CLI)

Konta bez praw odczytu są widoczne jako \`nazwa (?)\` z podpowiedzią, jak naprawić uprawnienia. Uruchom \`agy-menu --diagnose\`, żeby zobaczyć status.

### Naprawa uprawnień (ACL, jako root)

\`\`\`bash
sudo setfacl -R -m u:<twoj-user>:rx /home/ala/.agyw
sudo setfacl -R -m u:<twoj-user>:rx /root/.agyw
\`\`\`

### Wznawianie cudzej sesji

\`Enter\` na rozmowie innego usera uruchamia:

\`\`\`bash
su - ala -c "cd '/home/ala/projekt' && agy --conversation 'abc-123'"
\`\`\`

Może pojawić się prośba o hasło. Alternatywa z \`sudo\` (jeśli masz \`NOPASSWD\`):

\`\`\`bash
sudo -iu ala sh -c "cd '/home/ala/projekt' && agy --conversation 'abc-123'"
\`\`\`

## Wymagania

- Linux lub macOS
- Rust ≥ 1.88 (edycja 2024)
- \`agy\` w \`\$PATH\`
- Do wznawiania cudzych sesji: \`su\` lub \`sudo\`

## Wkład

PR-y mile widziane. Przed wysłaniem:

\`\`\`bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test --all
\`\`\`

Zobacz [CONTRIBUTING.md](CONTRIBUTING.md) po szczegóły.

Wszystkie zmiany lądują w [CHANGELOG.md](CHANGELOG.md) (sekcja \`[Unreleased]\`).

## Licencja

MIT — patrz [LICENSE](LICENSE).
PYEOF

# ──────────────────────────────────────────────────────────────── CHANGELOG.md
cat << PYEOF > CHANGELOG.md
# Changelog

Wszystkie istotne zmiany w projekcie są dokumentowane w tym pliku.

Format oparty na [Keep a Changelog](https://keepachangelog.com/pl/1.1.0/),
wersjonowanie zgodne z [Semantic Versioning](https://semver.org/lang/pl/).

## [Unreleased]

### Planowane

- Async I/O przy ładowaniu dużych transkryptów (\`tokio::task::spawn_blocking\`)
- Fuzzy search (jak w \`fzf\`) zamiast \`contains\`
- Grupowanie rozmów po workspace (Tabs)
- Testy jednostkowe dla \`wrap_string\`, \`truncate_chars\`, parsowania metadanych
- Wsparcie dla Windows (WSL poza zakresem — natywne \`C:\\Users\\*\`)

---

## [0.1.0] — 2025-11-07

### Dodane

- **Lista rozmów AGY** z tytułem, datą modyfikacji i liczbą kroków
- **Wyszukiwanie** po tytule i ścieżce workspace (\`/\`)
- **Panel szczegółów** — ostatnia wiadomość, lista raportów \`.md\`, wszystkie Workspace URIs
- **Pełny transkrypt sesji** (\`transcript.jsonl\`) z parsowaniem JSONL (\`T\`)
- **Wznawianie sesji** przez \`agy --conversation <id>\` (\`Enter\`)
- **Wielu użytkowników**:
  - auto-skan \`/home/*\`, \`/root\`, \`/Users/*\`
  - \`AGY_EXTRA_USERS="ala:/path,bob:/path"\` (env)
  - \`--user ala:/path\` (CLI, wielokrotne)
  - cykliczny filtr użytkownika (\`U\`): \`wszyscy → ala → bob → …\`
  - oznaczenie \`(?)\` przy kontach bez praw odczytu
- **Wznawianie cudzej sesji** przez \`su - <user> -c "…"\` z bezpiecznym escapowaniem
- **\`--diagnose\`** — lista wykrytych userów, status odczytu, gotowe komendy ACL do naprawy
- **Wirtualizacja renderowania** — pre-wrap raz, wycinek widoczny w \`draw()\`:
  - responsywność przy transkryptach 10 000+ linii
  - cache panelu szczegółów (przebudowa tylko przy zmianie zaznaczenia)
  - cache etykiet listy (bez \`format!\` co klatkę)
  - filtr na \`Vec<usize>\` indeksów (bez klonowania \`ConversationItem\`)
- **Bezpieczeństwo UTF-8** — \`truncate_chars\`, \`short_id\` (bez paniki na \`ł\`, emoji)
- **Panic hook** przez \`ratatui::init()\` / \`ratatui::restore()\` — terminal zawsze wraca do stanu normalnego

### Klawisze

- \`Home\` / \`[\` / \`End\` / \`]\` — początek / koniec listy
- \`Ctrl+Home\` / \`Ctrl+End\` — początek / koniec panelu szczegółów
- \`PgUp\` / \`PgDn\` — przewijanie o ekran
- \`Ctrl+L\` — wyczyść filtr, \`Ctrl+U\` — wyczyść wpis szukania
- \`B\` / \`T\` — powrót z transkryptu
- \`Q\` / \`Ctrl+C\` — wyjście

### Naprawione

- Panika na \`&conv_id[..8]\` i \`&txt[..147]\` przy wielobajtowych znakach
- Zacinanie UI przy dużych transkryptach (file I/O co klatkę)
- Wyczerpanie \`u16\` przy przewijaniu >65 535 linii → \`usize\`
- Terminal zostający w raw mode po panice

### Znane ograniczenia

- Czytanie cudzych \`/home/*\` wymaga ACL albo uruchomienia z \`sudo\`
- \`Ctrl+Home\` / \`Ctrl+End\` mogą nie działać w \`tmux\`/\`screen\` bez \`extended-keys\`
- Windows nieobsługiwany (ścieżki \`HOME\`, \`su\`)

---

[Unreleased]: https://github.com/${USER_NAME}/agy-menu/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/${USER_NAME}/agy-menu/releases/tag/v0.1.0
PYEOF

# ──────────────────────────────────────────────────────────────── CONTRIBUTING.md
cat << PYEOF > CONTRIBUTING.md
# Współtworzenie agy-menu

Dzięki za chęć pomocy! Ten plik opisuje, jak zgłaszać błędy, proponować funkcje i wysyłać PR-y.

## Zgłaszanie błędów

Otwórz [issue](https://github.com/${USER_NAME}/agy-menu/issues/new/choose) i użyj szablonu **Bug report**.
Dołącz:

- output \`agy-menu --diagnose\`
- wersję \`agy\` i terminala
- krok po kroku, jak odtworzyć

Bez tego trudno cokolwiek ustalić — większość bugów to nieczytelne \`/home/*\` albo specyfika terminala.

## Propozycje funkcji

Użyj szablonu **Feature request**. Opisz **problem**, nie tylko rozwiązanie —
„chcę X” to mniej niż „przy Y dzieje się Z, chciałbym żeby…”.

## Pull requesty

1. Zforkuj repo, utwórz branch: \`git checkout -b feat/nazwa\` lub \`fix/nazwa\`
2. Zrób zmianę — **minimalną**, jedna rzecz na PR
3. Uruchom lokalnie:
   \`\`\`bash
   cargo fmt --all
   cargo clippy --all-targets -- -D warnings
   cargo build --release
   cargo test --all
   \`\`\`
4. Dopisz wpis do \`CHANGELOG.md\` w sekcji \`[Unreleased]\` (kategoria: Dodane / Zmienione / Naprawione / Usunięte)
5. Jeśli zmiana dotyczy klawiszy lub CLI — zaktualizuj \`README.md\`
6. Otwórz PR z wypełnionym szablonem

### Styl kodu

- \`cargo fmt\` — bez dyskusji
- \`cargo clippy -- -D warnings\` — bez wyjątków (chyba że \`#[allow(...)]\` z uzasadnieniem)
- Nowe funkcje: testy jednostkowe jeśli logika jest testowalna
- Zero \`unwrap()\` w kodzie produkcyjnym — \`?\`, \`unwrap_or_default()\`, \`ok_or\`
- Komentarze: po polsku w kodzie, po angielsku w publicznych API / doc-commentach

### Commity

Konwencja [Conventional Commits](https://www.conventionalcommits.org/):

\`\`\`
feat: dodaj fuzzy search
fix: panic na wielobajtowych znakach w truncate
docs: opis --diagnose w README
refactor: wydziel loader do modułu
test: wrap_string dla pustego stringa
chore: bump ratatui do 0.30.3
\`\`\`

Nie „update”, nie „fix bug”, nie „WIP”. Jeśli commit robi kilka rzeczy — rozbij.

## Licencja

Wysyłając PR, zgadzasz się na licencjonowanie swojego wkładu na warunkach [MIT](LICENSE).
PYEOF

# ──────────────────────────────────────────────────── .github/workflows/ci.yml
mkdir -p .github/workflows
cat << 'PYEOF' > .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - uses: Swatinem/rust-cache@v2

      - name: fmt
        run: cargo fmt --all -- --check

      - name: clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: build
        run: cargo build --release

      - name: test
        run: cargo test --all
PYEOF

# ─────────────────────────────────────── .github/ISSUE_TEMPLATE/bug_report.md
mkdir -p .github/ISSUE_TEMPLATE
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
