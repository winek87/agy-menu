# Changelog

Wszystkie istotne zmiany w projekcie są dokumentowane w tym pliku.

Format oparty na [Keep a Changelog](https://keepachangelog.com/pl/1.1.0/),
wersjonowanie zgodne z [Semantic Versioning](https://semver.org/lang/pl/).

## [Unreleased]

### Planowane

- Async I/O przy ładowaniu dużych transkryptów (`tokio::task::spawn_blocking`)
- Fuzzy search (jak w `fzf`) zamiast `contains`
- Grupowanie rozmów po workspace (Tabs)
- Testy jednostkowe dla `wrap_string`, `truncate_chars`, parsowania metadanych
- Wsparcie dla Windows (WSL poza zakresem — natywne `C:\Users\*`)

---

## [0.1.0] — 2025-11-07

### Dodane

- **Lista rozmów AGY** z tytułem, datą modyfikacji i liczbą kroków
- **Wyszukiwanie** po tytule i ścieżce workspace (`/`)
- **Panel szczegółów** — ostatnia wiadomość, lista raportów `.md`, wszystkie Workspace URIs
- **Pełny transkrypt sesji** (`transcript.jsonl`) z parsowaniem JSONL (`T`)
- **Wznawianie sesji** przez `agy --conversation <id>` (`Enter`)
- **Wielu użytkowników**:
  - auto-skan `/home/*`, `/root`, `/Users/*`
  - `AGY_EXTRA_USERS="ala:/path,bob:/path"` (env)
  - `--user ala:/path` (CLI, wielokrotne)
  - cykliczny filtr użytkownika (`U`): `wszyscy → ala → bob → …`
  - oznaczenie `(?)` przy kontach bez praw odczytu
- **Wznawianie cudzej sesji** przez `su - <user> -c "…"` z bezpiecznym escapowaniem
- **`--diagnose`** — lista wykrytych userów, status odczytu, gotowe komendy ACL do naprawy
- **Wirtualizacja renderowania** — pre-wrap raz, wycinek widoczny w `draw()`:
  - responsywność przy transkryptach 10 000+ linii
  - cache panelu szczegółów (przebudowa tylko przy zmianie zaznaczenia)
  - cache etykiet listy (bez `format!` co klatkę)
  - filtr na `Vec<usize>` indeksów (bez klonowania `ConversationItem`)
- **Bezpieczeństwo UTF-8** — `truncate_chars`, `short_id` (bez paniki na `ł`, emoji)
- **Panic hook** przez `ratatui::init()` / `ratatui::restore()` — terminal zawsze wraca do stanu normalnego

### Klawisze

- `Home` / `[` / `End` / `]` — początek / koniec listy
- `Ctrl+Home` / `Ctrl+End` — początek / koniec panelu szczegółów
- `PgUp` / `PgDn` — przewijanie o ekran
- `Ctrl+L` — wyczyść filtr, `Ctrl+U` — wyczyść wpis szukania
- `B` / `T` — powrót z transkryptu
- `Q` / `Ctrl+C` — wyjście

### Naprawione

- Panika na `&conv_id[..8]` i `&txt[..147]` przy wielobajtowych znakach
- Zacinanie UI przy dużych transkryptach (file I/O co klatkę)
- Wyczerpanie `u16` przy przewijaniu >65 535 linii → `usize`
- Terminal zostający w raw mode po panice

### Znane ograniczenia

- Czytanie cudzych `/home/*` wymaga ACL albo uruchomienia z `sudo`
- `Ctrl+Home` / `Ctrl+End` mogą nie działać w `tmux`/`screen` bez `extended-keys`
- Windows nieobsługiwany (ścieżki `HOME`, `su`)

---

[Unreleased]: https://github.com/winek87/agy-menu/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/winek87/agy-menu/releases/tag/v0.1.0
