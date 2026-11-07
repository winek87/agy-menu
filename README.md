# agy-menu

[![CI](https://github.com/winek87/agy-menu/actions/workflows/ci.yml/badge.svg)](https://github.com/winek87/agy-menu/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/winek87/agy-menu?logo=github)](https://github.com/winek87/agy-menu/releases)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey)]()

Interaktywne TUI do przeglądania, filtrowania i wznawiania rozmów **AGY** — z wielu kont użytkowników jednocześnie.

## Funkcje

- 📋 **Lista rozmów** z tytułem, datą i liczbą kroków
- 🔍 **Wyszukiwanie** po tytule i ścieżce workspace
- 👥 **Wielu użytkowników** — auto-skan `/home/*`, `/root`, `/Users/*` + ręczne `--user`
- 📄 **Panel szczegółów** — ostatnia wiadomość, lista raportów `.md`, workspace URIs
- 📜 **Pełny transkrypt** sesji (`transcript.jsonl`) z wirtualizowanym renderowaniem
- 🚀 **Wznawianie sesji** — `Enter` uruchamia `agy --conversation <id>`; dla innego usera przez `su`
- ⚡ **Responsywność** przy dużych transkryptach (10 000+ linii)

## Instalacja

```bash
git clone https://github.com/winek87/agy-menu.git
cd agy-menu
cargo build --release
sudo install -m 755 target/release/agy-menu /usr/local/bin/agy-menu
```

Wymaga **Rust ≥ 1.88** (`rustup update stable`).

## Użycie

```bash
# domyślnie: bieżący użytkownik + auto-skan /home/*, /root, /Users/*
agy-menu

# dodatkowi użytkownicy przez env
AGY_EXTRA_USERS="ala:/home/ala/.agyw/shared/cache,bob:/home/bob/.agyw/shared/cache" agy-menu

# lub przez CLI (wielokrotne)
agy-menu \
  --user ala:/home/ala/.agyw/shared/cache \
  --user bob:/home/bob/.agyw/shared/cache

# diagnostyka: kto wykryty, kto czytelny, gotowe komendy ACL
agy-menu --diagnose
```

## Klawisze

### Lista

| Klawisz | Akcja |
|---------|-------|
| `↑` `↓` `j` `k` | nawigacja |
| `Home` / `[` | początek listy |
| `End` / `]` | koniec listy |
| `Ctrl+Home` | początek panelu szczegółów |
| `Ctrl+End` | koniec panelu szczegółów |
| `PgUp` / `PgDn` | przewiń szczegóły o ekran |
| `/` | szukaj |
| `U` | zmień użytkownika (wszyscy → ala → bob → …) |
| `T` | pokaż transkrypt |
| `Enter` | wybierz i uruchom `agy` |
| `Ctrl+L` | wyczyść filtr |
| `Q` / `Ctrl+C` | wyjdź |

### Szukanie

| Klawisz | Akcja |
|---------|-------|
| litery | wpisuj |
| `Backspace` | usuń znak |
| `Ctrl+U` | wyczyść cały wpis |
| `Enter` | zatwierdź |
| `Ctrl+C` | wyjdź |

### Transkrypt

| Klawisz | Akcja |
|---------|-------|
| `↑` `↓` `j` `k` | przewiń linię |
| `PgUp` / `PgDn` | przewiń ekran |
| `Home` / `[` | początek |
| `End` / `]` | koniec |
| `B` / `T` | powrót do listy |
| `Q` / `Ctrl+C` | wyjdź |

## Wiele użytkowników

`agy-menu` automatycznie wykrywa cache AGY w:

1. `$HOME/.agyw/shared/cache` (bieżący user — zawsze)
2. `/root/.agyw/shared/cache` (jeśli czytelne lub jako `(?)`)
3. `/home/*/.agyw/shared/cache` (Linux)
4. `/Users/*/.agyw/shared/cache` (macOS)
5. `AGY_EXTRA_USERS="nazwa:ścieżka,..."` (env)
6. `--user nazwa:ścieżka` (CLI)

Konta bez praw odczytu są widoczne jako `nazwa (?)` z podpowiedzią, jak naprawić uprawnienia. Uruchom `agy-menu --diagnose`, żeby zobaczyć status.

### Naprawa uprawnień (ACL, jako root)

```bash
sudo setfacl -R -m u:<twoj-user>:rx /home/ala/.agyw
sudo setfacl -R -m u:<twoj-user>:rx /root/.agyw
```

### Wznawianie cudzej sesji

`Enter` na rozmowie innego usera uruchamia:

```bash
su - ala -c "cd '/home/ala/projekt' && agy --conversation 'abc-123'"
```

Może pojawić się prośba o hasło. Alternatywa z `sudo` (jeśli masz `NOPASSWD`):

```bash
sudo -iu ala sh -c "cd '/home/ala/projekt' && agy --conversation 'abc-123'"
```

## Wymagania

- Linux lub macOS
- Rust ≥ 1.88 (edycja 2024)
- `agy` w `$PATH`
- Do wznawiania cudzych sesji: `su` lub `sudo`

## Wkład

PR-y mile widziane. Przed wysłaniem:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test --all
```

Zobacz [CONTRIBUTING.md](CONTRIBUTING.md) po szczegóły.

Wszystkie zmiany lądują w [CHANGELOG.md](CHANGELOG.md) (sekcja `[Unreleased]`).

## Licencja

MIT — patrz [LICENSE](LICENSE).
