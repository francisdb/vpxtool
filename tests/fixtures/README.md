# scores fixtures

Real score files copied verbatim from played-once tables. Each file documents
the table / source that exhibits the pattern and the parser behavior it pins;
verbatim bytes (CRLF, embedded non-UTF8, trailing junk, multi-section noise,
etc.) so the tests catch regressions against actual on-disk shapes.

Driven by `tests/scores_fixtures.rs`.

Tip: re-copy a fixture from the source table when adding a new shape - the
"real bytes" path is what we want, not a synthesized approximation.

## VPReg (modern HighScoreN/HighScoreNName)

| File | Source table | Pattern |
|---|---|---|
| `vpreg_modern_matrix.ini` | The Matrix (Original 2023) | 4-entry baseline, names after the number |
| `vpreg_modern_volkan_no_names.ini` | Volkan Steel and Metal | Score-only ranked list, no initials |
| `vpreg_modern_lochness_single_entry.ini` | Loch Ness Monster (GamePlan 1985) | Single entry; unranked HIGH SCORE block |
| `vpreg_modern_name_before_n.ini` | Van Halen (Original 2020) | `HighScoreName<N>` (Name *before* number) |

## VPReg (legacy hiscore/hsa)

| File | Source table | Pattern |
|---|---|---|
| `vpreg_legacy_abracadabra.ini` | Abra Ca Dabra (Gottlieb 1975) | Lowercase `hiscore` / `hsa1/2/3` with surrounding settings |
| `vpreg_legacy_a_go_go_uppercase.ini` | A-Go-Go (Williams 1966) | CamelCase `HighScore` / `HSA1/2/3` (case-insensitive match) |

## GLF (vpx-glf framework)

| File | Source table | Pattern |
|---|---|---|
| `glf_dark_chaos.ini` | Dark Chaos (apophis 2025) | One ranked category, distinct-top-label split |

## EMHS Black's 5+5

| File | Source table | Pattern |
|---|---|---|
| `emhs_5plus5_carnaval.txt` | Carnaval no Rio (LTD 1977) | 12-line file, 5 scores + 5 initials |
| `emhs_5plus5_8ball.txt` | 8 Ball (Williams 1966) | 16-line, six header bytes before the score block |
| `emhs_5plus5_roller_coaster.txt` | Roller Coaster (Gottlieb 1971) | 18-line, header bytes + trailing bytes |
| `emhs_5plus5_long_names.txt` | Cuphead Pro Package v1 | Character-name initials (`CUPHEAD`, ...) |

## EMHS single-hisc (no initials)

| File | Source table | Pattern |
|---|---|---|
| `emhs_single_2in1.txt` | 2 in 1 (Bally 1964) | 8-line all-integer; `hisc` is the max |
| `emhs_single_4queens.txt` | 4 Queens (Bally 1970) | 6-line all-integer |
