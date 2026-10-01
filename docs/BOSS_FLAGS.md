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

Death bits are the after line. Before/during bits are the scan that preceded it. Noise already removed from the after column: `896002`, `896006`, `896009`, `896010`. `9015` and `9019` are the shared pair and are left in the column where they rose.

| Nightlord | Day | Boss | Before / during | After |
|---|---:|---|---|---|
| Adel | 1 | Gaping Dragon | 1102, 1108, 9015, 9019, 67020, 887000, 887001, 887006, 887009, 887011 | 887007, 887008, 887010, 887012, 887013, 887014 |
| Adel | 1 | Duke's Dear Freja | 878001, 878006, 878009, 878010, 878015 | 878000 |
| Adel | 1 | Valiant Gargoyle | 1102, 88016, 623900, 878001, 878006, 878009, 878010 | 88014, 878007, 878008, 878011, 878012, 893900, 893901, 893902, 893903 |
| Adel | 1 | Valiant Gargoyle run 2 | 53022, 81019, 623900, 749800, 752000, 752001, 752002, 752003, 752004, 920900 | 88014, 116005 |
| Adel | 2 | Ancient Dragon | 1105, 1314, 9015, 9018, 9019, 200010, 200011, 200099 | 752021, 752025, 752028, 797003, 797007, 797010, 896003 |
| Gladius | 1 | Demihuman Queen | 1102, 1108, 9015, 9019, 623900, 896001, 896011, 896012, 920900 | 896007, 896008, 896013, 896017 |
| Gladius | overworld | Demihuman Queen | 531, 74000-74019, 779011-779023, truncated +35 | 5000, 67014, 67043, 67044, 67048, 67050, 67051, 67052, 67057, 67060, 67062, 74018 |


Valiant Gargoyle, Adel Night 1, 2026-10-01. After line also had 1108, 1113, 1181, 1301, 1302, 67045, 74012, 95004, 95007, 116005, 878000. Those repeat an earlier Adel post-fight scan or the Freja death bit, so they are not in the After column.

Gargoyle run 2 after line also had 530, 5000, 53004, 81003, 81005, 81006, 779006, 788006, 824006, 833006, 860006, 887007, 887008, 887011, 887012, 1004038-1004046. `887007` and `887008` were on the Gaping Dragon death line, so that pair is not the boss. `88014` and `116005` repeated from Gargoyle run 1.
## Adding a sample

Copy a row. Keep the Nightlord, day, and boss name. If the same pair is run again, add the new bits in the notes column of `data/boss_flags.toml` instead of deleting the first sample.


## Script candidate, not yet logged

NR Sandbox's `m18_00_00_00` event sets these when a night-boss HP ratio hits 0. They were not in the Adel death window, so they are candidates until a run logs them.

| What | Flag |
|---|---:|
| Night boss HP reached 0 | 7512 |
| Expedition finalized after that | 6020 |

`IncrementTeamBossesKilled(1)` runs in the same event. The `887xxx` and `896xxx` clusters are not written by these scripts.

Confirmed fight noise, removed from the table: `896006`, `896009`, `896010`. They also appeared in the Adel Day 2 window, so those three are not the boss. `7512` did not rise in either Demihuman Queen scan.

Overworld Demihuman Queen, Gladius expedition, 2026-10-01. The before/during line was `+35` and truncated. The after-death line was `+13` and complete. `896002` was on that after line and also on the Adel Ancient Dragon death line, so it is noise.

Duke's Dear Freja, Adel Night 1, 2026-10-01. Before line: `878001, 878006, 878009, 878010, 878015`. After line: `9015, 9019, 878000`. `9015` and `9019` are the shared pair, not the boss.
