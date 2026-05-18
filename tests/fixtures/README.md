# scores testdata

Real-world score-file shapes captured as fixtures. Each file documents the
table / source that exhibits the pattern and the parser behavior it pins.

Names and scores are synthesized (AAA/BBB or character-name placeholders);
the *shape* of the file is what matters for the parser.

| File | Pattern pinned | Original observed on |
|---|---|---|
| `vpreg_modern_name_before_n.ini` | VPReg with `HighScoreNameN` (Name *before* number) | Van Halen (Original 2020) |
| `emhs_5plus5_long_names.txt` | Black's 5+5 with character names longer than 3 chars | Cuphead Pro Package v1 |
