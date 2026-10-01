# Boss flag table

One row per Nightlord and day-boss pair. Bits are the flags that turned on in the death window. A single sample does not prove the bit belongs to the boss rather than the day slot. Add a row, do not overwrite a different sample.

Nightlord names are the expedition target. Day boss names are the field boss that died.

## Nightlord defeat

These are the Hold flags already watched by the client.

| Nightlord | Flag |
|---|---:|
| Gladius | 150 |
| Adel | 151 |
| Gnoster | 152 |
| Maris | 153 |
| Libra | 154 |
| Fulghor | 155 |
| Caligo | 156 |
| Heolstor | 160 |
| Harmonia | 161 |
| Straghess | 162 |
| Everdark Gladius | 170 |
| Everdark Adel | 171 |
| Everdark Gnoster | 172 |
| Everdark Maris | 173 |
| Everdark Libra | 174 |
| Everdark Fulghor | 175 |
| Everdark Caligo | 176 |
| Everdark Harmonia | 181 |

## Day bosses

| Nightlord | Day | Boss | Death bits | Sample |
|---|---:|---|---|---|
| Adel | 1 | Gaping Dragon | 887007, 887008, 887010, 887012, 887013, 887014 | 2026-10-01 |
| Adel | 2 | Ancient Dragon | 752021, 752025, 752028, 797003, 797007, 797010, 896002, 896003, 896006, 896009, 896010 | 2026-10-01 |

Fight bits that came on before the death are not in this table. For the Adel Day 1 fight those were the earlier `887000`-`887011` flags.

## Adding a sample

Copy a row. Keep the Nightlord, day, and boss name. If the same pair is run again, add the new bits in the notes column of `data/boss_flags.toml` instead of deleting the first sample.


## Script candidate, not yet logged

NR Sandbox's `m18_00_00_00` event sets these when a night-boss HP ratio hits 0. They were not in the Adel death window, so they are candidates until a run logs them.

| What | Flag |
|---|---:|
| Night boss HP reached 0 | 7512 |
| Expedition finalized after that | 6020 |

`IncrementTeamBossesKilled(1)` runs in the same event. The `887xxx` and `896xxx` clusters are not written by these scripts.
