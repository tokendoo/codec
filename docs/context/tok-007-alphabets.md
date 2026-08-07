# TOK-007 — Vocabulary-derived alphabets

TOK-002 bare shipping median (cl100k, reference): **0.421**.

## Gate

- cl100k derived median (S6, shipping): **0.3868**
- Beats bare baseline 0.421: **true**
- Default alphabet changed: **true**

## Per-tokenizer summaries (S6, shipping fixtures)

| tokenizer | bare median | derived median | beats 0.421 | symbols |
|-----------|------------:|---------------:|:-----------:|--------:|
| cl100k_base | 0.4207 | 0.3868 | true | 2048 |
| o200k_base | 0.4210 | 0.3876 | true | 2048 |
| llama | 0.3747 | 0.3681 | true | 1377 |

## Table overlap

cl100k: 2048, o200k: 2048, llama: 1377

cl100k∩o200k: 1838, cl100k∩llama: 438

## Cells

| tokenizer | fixture | alphabet | tokens_before | tokens_after | ratio | identity |
|-----------|---------|----------|--------------:|-------------:|------:|:--------:|
| cl100k_base | homogeneous_100 | bare | 2903 | 1327 | 0.4571 | false |
| cl100k_base | homogeneous_100 | derived_cl100k_v1 | 2903 | 1229 | 0.4234 | false |
| cl100k_base | homogeneous_1000 | bare | 29003 | 9427 | 0.3250 | false |
| cl100k_base | homogeneous_1000 | derived_cl100k_v1 | 29003 | 8387 | 0.2892 | false |
| cl100k_base | realistic_api | bare | 3200 | 2789 | 0.8716 | false |
| cl100k_base | realistic_api | derived_cl100k_v1 | 3200 | 2415 | 0.7547 | false |
| cl100k_base | size_10k | bare | 3860 | 1624 | 0.4207 | false |
| cl100k_base | size_10k | derived_cl100k_v1 | 3860 | 1493 | 0.3868 | false |
| cl100k_base | size_100k | bare | 39983 | 13087 | 0.3273 | false |
| cl100k_base | size_100k | derived_cl100k_v1 | 39983 | 11663 | 0.2917 | false |
| o200k_base | homogeneous_100 | bare | 2903 | 1328 | 0.4575 | false |
| o200k_base | homogeneous_100 | derived_o200k_v1 | 2903 | 1232 | 0.4244 | false |
| o200k_base | homogeneous_1000 | bare | 29003 | 9428 | 0.3251 | false |
| o200k_base | homogeneous_1000 | derived_o200k_v1 | 29003 | 8390 | 0.2893 | false |
| o200k_base | realistic_api | bare | 3242 | 2829 | 0.8726 | false |
| o200k_base | realistic_api | derived_o200k_v1 | 3242 | 2462 | 0.7594 | false |
| o200k_base | size_10k | bare | 3860 | 1625 | 0.4210 | false |
| o200k_base | size_10k | derived_o200k_v1 | 3860 | 1496 | 0.3876 | false |
| o200k_base | size_100k | bare | 39983 | 13088 | 0.3273 | false |
| o200k_base | size_100k | derived_o200k_v1 | 39983 | 11666 | 0.2918 | false |
| llama | homogeneous_100 | bare | 3958 | 1623 | 0.4101 | false |
| llama | homogeneous_100 | derived_llama_v1 | 3958 | 1596 | 0.4032 | false |
| llama | homogeneous_1000 | bare | 40558 | 11523 | 0.2841 | false |
| llama | homogeneous_1000 | derived_llama_v1 | 40558 | 11240 | 0.2771 | false |
| llama | realistic_api | bare | 4352 | 3748 | 0.8612 | false |
| llama | realistic_api | derived_llama_v1 | 4352 | 3311 | 0.7608 | false |
| llama | size_10k | bare | 5300 | 1986 | 0.3747 | false |
| llama | size_10k | derived_llama_v1 | 5300 | 1951 | 0.3681 | false |
| llama | size_100k | bare | 55808 | 15915 | 0.2852 | false |
| llama | size_100k | derived_llama_v1 | 55808 | 15528 | 0.2782 | false |
