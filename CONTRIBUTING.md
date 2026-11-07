# Współtworzenie agy-menu

Dzięki za chęć pomocy! Ten plik opisuje, jak zgłaszać błędy, proponować funkcje i wysyłać PR-y.

## Zgłaszanie błędów

Otwórz [issue](https://github.com/winek87/agy-menu/issues/new/choose) i użyj szablonu **Bug report**.
Dołącz:

- output `agy-menu --diagnose`
- wersję `agy` i terminala
- krok po kroku, jak odtworzyć

Bez tego trudno cokolwiek ustalić — większość bugów to nieczytelne `/home/*` albo specyfika terminala.

## Propozycje funkcji

Użyj szablonu **Feature request**. Opisz **problem**, nie tylko rozwiązanie —
„chcę X” to mniej niż „przy Y dzieje się Z, chciałbym żeby…”.

## Pull requesty

1. Zforkuj repo, utwórz branch: `git checkout -b feat/nazwa` lub `fix/nazwa`
2. Zrób zmianę — **minimalną**, jedna rzecz na PR
3. Uruchom lokalnie:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets -- -D warnings
   cargo build --release
   cargo test --all
   ```
4. Dopisz wpis do `CHANGELOG.md` w sekcji `[Unreleased]` (kategoria: Dodane / Zmienione / Naprawione / Usunięte)
5. Jeśli zmiana dotyczy klawiszy lub CLI — zaktualizuj `README.md`
6. Otwórz PR z wypełnionym szablonem

### Styl kodu

- `cargo fmt` — bez dyskusji
- `cargo clippy -- -D warnings` — bez wyjątków (chyba że `#[allow(...)]` z uzasadnieniem)
- Nowe funkcje: testy jednostkowe jeśli logika jest testowalna
- Zero `unwrap()` w kodzie produkcyjnym — `?`, `unwrap_or_default()`, `ok_or`
- Komentarze: po polsku w kodzie, po angielsku w publicznych API / doc-commentach

### Commity

Konwencja [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: dodaj fuzzy search
fix: panic na wielobajtowych znakach w truncate
docs: opis --diagnose w README
refactor: wydziel loader do modułu
test: wrap_string dla pustego stringa
chore: bump ratatui do 0.30.3
```

Nie „update”, nie „fix bug”, nie „WIP”. Jeśli commit robi kilka rzeczy — rozbij.

## Licencja

Wysyłając PR, zgadzasz się na licencjonowanie swojego wkładu na warunkach [MIT](LICENSE).
