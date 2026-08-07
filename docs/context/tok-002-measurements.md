# TOK-002 Measurements

Baseline Worker WASM size (pre-change reference): **601320** bytes.

## Decision

- Winning alphabet: **derived_cl100k_v1**
- Median S6 token ratio (shipping fixtures, shipped transforms): **0.3868**
- Ship token claim: **true**
- Default transforms: keys=false, tabular=true, intern=true

### Notes

- alphabet control: median_token_ratio_S6=0.6575, mean_|divergence|=0.1152
- alphabet printable: median_token_ratio_S6=0.5241, mean_|divergence|=0.1673
- alphabet bare: median_token_ratio_S6=0.4207, mean_|divergence|=0.1014
- alphabet common_word: median_token_ratio_S6=0.4259, mean_|divergence|=0.0748
- alphabet derived_cl100k_v1: median_token_ratio_S6=0.3868, mean_|divergence|=0.0668
- alphabet derived_o200k_v1: median_token_ratio_S6=0.3873, mean_|divergence|=0.0672
- alphabet derived_llama_v1: median_token_ratio_S6=0.3972, mean_|divergence|=0.0780
- transform key_dictionary on derived_cl100k_v1: hits=2/shipping with ratio<=0.90, prose_ok=true => default_on=true
- transform tabular on derived_cl100k_v1: hits=5/shipping with ratio<=0.90, prose_ok=true => default_on=true
- transform intern_values on derived_cl100k_v1: hits=5/shipping with ratio<=0.90, prose_ok=true => default_on=true
- transforms_default pinned to ADR-011 shipped values (key_dictionary=false, tabular=true, intern_values=true); isolation hits above are diagnostic only
- prose_heavy S6 token_ratio=1.0051 (must be <=1.05)

## Full pipeline (S7) — primary table

| fixture | alphabet | bytes_before | bytes_after | tokens_before | tokens_after | byte_ratio | token_ratio | divergence | identity |
|---------|----------|-------------:|------------:|--------------:|-------------:|-----------:|------------:|-----------:|---------|
| homogeneous_8 | control | 604 | 624 | 235 | 244 | 1.0331 | 1.0383 | +0.0052 | true |
| homogeneous_8 | printable | 604 | 624 | 235 | 244 | 1.0331 | 1.0383 | +0.0052 | true |
| homogeneous_8 | bare | 604 | 624 | 235 | 244 | 1.0331 | 1.0383 | +0.0052 | true |
| homogeneous_8 | common_word | 604 | 624 | 235 | 244 | 1.0331 | 1.0383 | +0.0052 | true |
| homogeneous_8 | derived_cl100k_v1 | 604 | 633 | 235 | 249 | 1.0480 | 1.0596 | +0.0116 | true |
| homogeneous_8 | derived_o200k_v1 | 604 | 633 | 235 | 251 | 1.0480 | 1.0681 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | 604 | 633 | 235 | 251 | 1.0480 | 1.0681 | +0.0201 | true |
| homogeneous_100 | control | 7624 | 4282 | 2903 | 2026 | 0.5616 | 0.6979 | +0.1363 | false |
| homogeneous_100 | printable | 7624 | 2762 | 2903 | 1537 | 0.3623 | 0.5295 | +0.1672 | false |
| homogeneous_100 | bare | 7624 | 2449 | 2903 | 1327 | 0.3212 | 0.4571 | +0.1359 | false |
| homogeneous_100 | common_word | 7624 | 2764 | 2903 | 1349 | 0.3625 | 0.4647 | +0.1022 | false |
| homogeneous_100 | derived_cl100k_v1 | 7624 | 2507 | 2903 | 1234 | 0.3288 | 0.4251 | +0.0962 | false |
| homogeneous_100 | derived_o200k_v1 | 7624 | 2507 | 2903 | 1236 | 0.3288 | 0.4258 | +0.0969 | false |
| homogeneous_100 | derived_llama_v1 | 7624 | 2492 | 2903 | 1259 | 0.3269 | 0.4337 | +0.1068 | false |
| homogeneous_1000 | control | 77224 | 36682 | 29003 | 16298 | 0.4750 | 0.5619 | +0.0869 | false |
| homogeneous_1000 | printable | 77224 | 21662 | 29003 | 11437 | 0.2805 | 0.3943 | +0.1138 | false |
| homogeneous_1000 | bare | 77224 | 18649 | 29003 | 9427 | 0.2415 | 0.3250 | +0.0835 | false |
| homogeneous_1000 | common_word | 77224 | 21664 | 29003 | 9609 | 0.2805 | 0.3313 | +0.0508 | false |
| homogeneous_1000 | derived_cl100k_v1 | 77224 | 19007 | 29003 | 8392 | 0.2461 | 0.2893 | +0.0432 | false |
| homogeneous_1000 | derived_o200k_v1 | 77224 | 19007 | 29003 | 8394 | 0.2461 | 0.2894 | +0.0433 | false |
| homogeneous_1000 | derived_llama_v1 | 77224 | 18992 | 29003 | 8717 | 0.2459 | 0.3006 | +0.0546 | false |
| heterogeneous_nested | control | 2002 | 2022 | 766 | 776 | 1.0100 | 1.0131 | +0.0031 | true |
| heterogeneous_nested | printable | 2002 | 2022 | 766 | 776 | 1.0100 | 1.0131 | +0.0031 | true |
| heterogeneous_nested | bare | 2002 | 2022 | 766 | 776 | 1.0100 | 1.0131 | +0.0031 | true |
| heterogeneous_nested | common_word | 2002 | 1644 | 766 | 736 | 0.8212 | 0.9608 | +0.1397 | false |
| heterogeneous_nested | derived_cl100k_v1 | 2002 | 1659 | 766 | 700 | 0.8287 | 0.9138 | +0.0852 | false |
| heterogeneous_nested | derived_o200k_v1 | 2002 | 1659 | 766 | 702 | 0.8287 | 0.9164 | +0.0878 | false |
| heterogeneous_nested | derived_llama_v1 | 2002 | 1664 | 766 | 743 | 0.8312 | 0.9700 | +0.1388 | false |
| realistic_api | control | 9382 | 7829 | 3200 | 3447 | 0.8345 | 1.0772 | +0.2427 | false |
| realistic_api | printable | 9382 | 5799 | 3200 | 2862 | 0.6181 | 0.8944 | +0.2763 | false |
| realistic_api | bare | 9382 | 7423 | 3200 | 2789 | 0.7912 | 0.8716 | +0.0804 | false |
| realistic_api | common_word | 9382 | 5635 | 3200 | 2467 | 0.6006 | 0.7709 | +0.1703 | false |
| realistic_api | derived_cl100k_v1 | 9382 | 5445 | 3200 | 2309 | 0.5804 | 0.7216 | +0.1412 | false |
| realistic_api | derived_o200k_v1 | 9382 | 5445 | 3200 | 2311 | 0.5804 | 0.7222 | +0.1418 | false |
| realistic_api | derived_llama_v1 | 9382 | 5484 | 3200 | 2394 | 0.5845 | 0.7481 | +0.1636 | false |
| prose_heavy | control | 14496 | 14516 | 2965 | 2975 | 1.0014 | 1.0034 | +0.0020 | true |
| prose_heavy | printable | 14496 | 14516 | 2965 | 2975 | 1.0014 | 1.0034 | +0.0020 | true |
| prose_heavy | bare | 14496 | 14516 | 2965 | 2975 | 1.0014 | 1.0034 | +0.0020 | true |
| prose_heavy | common_word | 14496 | 14516 | 2965 | 2975 | 1.0014 | 1.0034 | +0.0020 | true |
| prose_heavy | derived_cl100k_v1 | 14496 | 14525 | 2965 | 2980 | 1.0020 | 1.0051 | +0.0031 | true |
| prose_heavy | derived_o200k_v1 | 14496 | 14525 | 2965 | 2982 | 1.0020 | 1.0057 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | 14496 | 14525 | 2965 | 2982 | 1.0020 | 1.0057 | +0.0037 | true |
| size_1k | control | 983 | 1003 | 380 | 389 | 1.0203 | 1.0237 | +0.0033 | true |
| size_1k | printable | 983 | 1003 | 380 | 389 | 1.0203 | 1.0237 | +0.0033 | true |
| size_1k | bare | 983 | 1003 | 380 | 389 | 1.0203 | 1.0237 | +0.0033 | true |
| size_1k | common_word | 983 | 1003 | 380 | 389 | 1.0203 | 1.0237 | +0.0033 | true |
| size_1k | derived_cl100k_v1 | 983 | 1012 | 380 | 394 | 1.0295 | 1.0368 | +0.0073 | true |
| size_1k | derived_o200k_v1 | 983 | 1012 | 380 | 396 | 1.0295 | 1.0421 | +0.0126 | true |
| size_1k | derived_llama_v1 | 983 | 1012 | 380 | 396 | 1.0295 | 1.0421 | +0.0126 | true |
| size_10k | control | 10176 | 5470 | 3860 | 2550 | 0.5375 | 0.6606 | +0.1231 | false |
| size_10k | printable | 10176 | 3455 | 3860 | 1900 | 0.3395 | 0.4922 | +0.1527 | false |
| size_10k | bare | 10176 | 3043 | 3860 | 1624 | 0.2990 | 0.4207 | +0.1217 | false |
| size_10k | common_word | 10176 | 3457 | 3860 | 1651 | 0.3397 | 0.4277 | +0.0880 | false |
| size_10k | derived_cl100k_v1 | 10176 | 3112 | 3860 | 1498 | 0.3058 | 0.3881 | +0.0823 | false |
| size_10k | derived_o200k_v1 | 10176 | 3112 | 3860 | 1500 | 0.3058 | 0.3886 | +0.0828 | false |
| size_10k | derived_llama_v1 | 10176 | 3097 | 3860 | 1533 | 0.3043 | 0.3972 | +0.0928 | false |
| size_100k | control | 105894 | 50224 | 39983 | 22468 | 0.4743 | 0.5619 | +0.0877 | false |
| size_100k | printable | 105894 | 29714 | 39983 | 15829 | 0.2806 | 0.3959 | +0.1153 | false |
| size_100k | bare | 105894 | 25603 | 39983 | 13087 | 0.2418 | 0.3273 | +0.0855 | false |
| size_100k | common_word | 105894 | 29716 | 39983 | 13334 | 0.2806 | 0.3335 | +0.0529 | false |
| size_100k | derived_cl100k_v1 | 105894 | 26083 | 39983 | 11668 | 0.2463 | 0.2918 | +0.0455 | false |
| size_100k | derived_o200k_v1 | 105894 | 26083 | 39983 | 11670 | 0.2463 | 0.2919 | +0.0456 | false |
| size_100k | derived_llama_v1 | 105894 | 26068 | 39983 | 12117 | 0.2462 | 0.3031 | +0.0569 | false |
| size_250k | control | 258958 | 122522 | 98603 | 55406 | 0.4731 | 0.5619 | +0.0888 | false |
| size_250k | printable | 258958 | 72702 | 98603 | 39277 | 0.2807 | 0.3983 | +0.1176 | false |
| size_250k | bare | 258958 | 62729 | 98603 | 32627 | 0.2422 | 0.3309 | +0.0887 | false |
| size_250k | common_word | 258958 | 72704 | 98603 | 33224 | 0.2808 | 0.3369 | +0.0562 | false |
| size_250k | derived_cl100k_v1 | 258958 | 63861 | 98603 | 29161 | 0.2466 | 0.2957 | +0.0491 | false |
| size_250k | derived_o200k_v1 | 258958 | 63861 | 98603 | 29163 | 0.2466 | 0.2958 | +0.0492 | false |
| size_250k | derived_llama_v1 | 258958 | 63845 | 98603 | 30257 | 0.2465 | 0.3069 | +0.0603 | false |

## Isolation appendix (all subsets)

| fixture | alphabet | subset | token_ratio | byte_ratio | divergence | identity |
|---------|----------|--------|------------:|-----------:|-----------:|---------|
| homogeneous_8 | control | S0_none | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | control | S1_keys | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | control | S2_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | control | S3_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | control | S4_keys_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | control | S5_keys_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | control | S6_tabular_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | control | S7_full | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | printable | S0_none | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | printable | S1_keys | 1.1319 | 0.9404 | +0.1915 | false |
| homogeneous_8 | printable | S2_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | printable | S3_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | printable | S4_keys_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | printable | S5_keys_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | printable | S6_tabular_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | printable | S7_full | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S0_none | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S1_keys | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S2_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S3_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S4_keys_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S5_keys_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S6_tabular_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | bare | S7_full | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | common_word | S0_none | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | common_word | S1_keys | 1.0383 | 0.8957 | +0.1426 | false |
| homogeneous_8 | common_word | S2_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | common_word | S3_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | common_word | S4_keys_tabular | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | common_word | S5_keys_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | common_word | S6_tabular_intern | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | common_word | S7_full | 1.0383 | 1.0331 | +0.0052 | true |
| homogeneous_8 | derived_cl100k_v1 | S0_none | 1.0596 | 1.0480 | +0.0116 | true |
| homogeneous_8 | derived_cl100k_v1 | S1_keys | 0.9915 | 0.9238 | +0.0676 | false |
| homogeneous_8 | derived_cl100k_v1 | S2_tabular | 1.0596 | 1.0480 | +0.0116 | true |
| homogeneous_8 | derived_cl100k_v1 | S3_intern | 1.0596 | 1.0480 | +0.0116 | true |
| homogeneous_8 | derived_cl100k_v1 | S4_keys_tabular | 1.0596 | 1.0480 | +0.0116 | true |
| homogeneous_8 | derived_cl100k_v1 | S5_keys_intern | 1.0596 | 1.0480 | +0.0116 | true |
| homogeneous_8 | derived_cl100k_v1 | S6_tabular_intern | 1.0596 | 1.0480 | +0.0116 | true |
| homogeneous_8 | derived_cl100k_v1 | S7_full | 1.0596 | 1.0480 | +0.0116 | true |
| homogeneous_8 | derived_o200k_v1 | S0_none | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_o200k_v1 | S1_keys | 1.0000 | 0.9238 | +0.0762 | false |
| homogeneous_8 | derived_o200k_v1 | S2_tabular | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_o200k_v1 | S3_intern | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_o200k_v1 | S4_keys_tabular | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_o200k_v1 | S5_keys_intern | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_o200k_v1 | S6_tabular_intern | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_o200k_v1 | S7_full | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S0_none | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S1_keys | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S2_tabular | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S3_intern | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S4_keys_tabular | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S5_keys_intern | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S6_tabular_intern | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_8 | derived_llama_v1 | S7_full | 1.0681 | 1.0480 | +0.0201 | true |
| homogeneous_100 | control | S0_none | 1.0031 | 1.0026 | +0.0005 | true |
| homogeneous_100 | control | S1_keys | 1.0031 | 1.0026 | +0.0005 | true |
| homogeneous_100 | control | S2_tabular | 0.7327 | 0.6149 | +0.1178 | false |
| homogeneous_100 | control | S3_intern | 1.0675 | 0.9456 | +0.1219 | false |
| homogeneous_100 | control | S4_keys_tabular | 0.7368 | 0.6187 | +0.1181 | false |
| homogeneous_100 | control | S5_keys_intern | 1.0031 | 1.0026 | +0.0005 | true |
| homogeneous_100 | control | S6_tabular_intern | 0.6938 | 0.5578 | +0.1359 | false |
| homogeneous_100 | control | S7_full | 0.6979 | 0.5616 | +0.1363 | false |
| homogeneous_100 | printable | S0_none | 1.0031 | 1.0026 | +0.0005 | true |
| homogeneous_100 | printable | S1_keys | 1.0741 | 0.8746 | +0.1995 | false |
| homogeneous_100 | printable | S2_tabular | 0.7317 | 0.6142 | +0.1174 | false |
| homogeneous_100 | printable | S3_intern | 0.8319 | 0.7488 | +0.0831 | false |
| homogeneous_100 | printable | S4_keys_tabular | 0.7007 | 0.6161 | +0.0846 | false |
| homogeneous_100 | printable | S5_keys_intern | 0.9029 | 0.6208 | +0.2821 | false |
| homogeneous_100 | printable | S6_tabular_intern | 0.5605 | 0.3604 | +0.2000 | false |
| homogeneous_100 | printable | S7_full | 0.5295 | 0.3623 | +0.1672 | false |
| homogeneous_100 | bare | S0_none | 1.0031 | 1.0026 | +0.0005 | true |
| homogeneous_100 | bare | S1_keys | 1.0031 | 1.0026 | +0.0005 | true |
| homogeneous_100 | bare | S2_tabular | 0.7317 | 0.6144 | +0.1173 | false |
| homogeneous_100 | bare | S3_intern | 0.7286 | 0.7095 | +0.0191 | false |
| homogeneous_100 | bare | S4_keys_tabular | 0.7317 | 0.6144 | +0.1173 | false |
| homogeneous_100 | bare | S5_keys_intern | 0.7286 | 0.7095 | +0.0191 | false |
| homogeneous_100 | bare | S6_tabular_intern | 0.4571 | 0.3212 | +0.1359 | false |
| homogeneous_100 | bare | S7_full | 0.4571 | 0.3212 | +0.1359 | false |
| homogeneous_100 | common_word | S0_none | 1.0031 | 1.0026 | +0.0005 | true |
| homogeneous_100 | common_word | S1_keys | 0.9714 | 0.8228 | +0.1486 | false |
| homogeneous_100 | common_word | S2_tabular | 0.7317 | 0.6144 | +0.1173 | false |
| homogeneous_100 | common_word | S3_intern | 0.7337 | 0.7488 | -0.0151 | false |
| homogeneous_100 | common_word | S4_keys_tabular | 0.7341 | 0.6163 | +0.1177 | false |
| homogeneous_100 | common_word | S5_keys_intern | 0.7020 | 0.5690 | +0.1330 | false |
| homogeneous_100 | common_word | S6_tabular_intern | 0.4623 | 0.3606 | +0.1017 | false |
| homogeneous_100 | common_word | S7_full | 0.4647 | 0.3625 | +0.1022 | false |
| homogeneous_100 | derived_cl100k_v1 | S0_none | 1.0048 | 1.0038 | +0.0010 | true |
| homogeneous_100 | derived_cl100k_v1 | S1_keys | 0.9042 | 0.8371 | +0.0671 | false |
| homogeneous_100 | derived_cl100k_v1 | S2_tabular | 0.7334 | 0.6156 | +0.1178 | false |
| homogeneous_100 | derived_cl100k_v1 | S3_intern | 0.7158 | 0.7150 | +0.0008 | false |
| homogeneous_100 | derived_cl100k_v1 | S4_keys_tabular | 0.7351 | 0.6177 | +0.1174 | false |
| homogeneous_100 | derived_cl100k_v1 | S5_keys_intern | 0.6269 | 0.5483 | +0.0787 | false |
| homogeneous_100 | derived_cl100k_v1 | S6_tabular_intern | 0.4234 | 0.3267 | +0.0966 | false |
| homogeneous_100 | derived_cl100k_v1 | S7_full | 0.4251 | 0.3288 | +0.0962 | false |
| homogeneous_100 | derived_o200k_v1 | S0_none | 1.0055 | 1.0038 | +0.0017 | true |
| homogeneous_100 | derived_o200k_v1 | S1_keys | 0.9049 | 0.8371 | +0.0678 | false |
| homogeneous_100 | derived_o200k_v1 | S2_tabular | 0.7341 | 0.6156 | +0.1185 | false |
| homogeneous_100 | derived_o200k_v1 | S3_intern | 0.7165 | 0.7150 | +0.0015 | false |
| homogeneous_100 | derived_o200k_v1 | S4_keys_tabular | 0.7358 | 0.6177 | +0.1181 | false |
| homogeneous_100 | derived_o200k_v1 | S5_keys_intern | 0.6276 | 0.5483 | +0.0794 | false |
| homogeneous_100 | derived_o200k_v1 | S6_tabular_intern | 0.4240 | 0.3267 | +0.0973 | false |
| homogeneous_100 | derived_o200k_v1 | S7_full | 0.4258 | 0.3288 | +0.0969 | false |
| homogeneous_100 | derived_llama_v1 | S0_none | 1.0055 | 1.0038 | +0.0017 | true |
| homogeneous_100 | derived_llama_v1 | S1_keys | 1.0055 | 1.0038 | +0.0017 | true |
| homogeneous_100 | derived_llama_v1 | S2_tabular | 0.7341 | 0.6156 | +0.1185 | false |
| homogeneous_100 | derived_llama_v1 | S3_intern | 0.7282 | 0.7151 | +0.0131 | false |
| homogeneous_100 | derived_llama_v1 | S4_keys_tabular | 0.7341 | 0.6156 | +0.1185 | false |
| homogeneous_100 | derived_llama_v1 | S5_keys_intern | 0.7282 | 0.7151 | +0.0131 | false |
| homogeneous_100 | derived_llama_v1 | S6_tabular_intern | 0.4337 | 0.3269 | +0.1068 | false |
| homogeneous_100 | derived_llama_v1 | S7_full | 0.4337 | 0.3269 | +0.1068 | false |
| homogeneous_1000 | control | S0_none | 1.0003 | 1.0003 | +0.0001 | true |
| homogeneous_1000 | control | S1_keys | 1.0003 | 1.0003 | +0.0001 | true |
| homogeneous_1000 | control | S2_tabular | 0.7250 | 0.6123 | +0.1126 | false |
| homogeneous_1000 | control | S3_intern | 0.9403 | 0.8625 | +0.0777 | false |
| homogeneous_1000 | control | S4_keys_tabular | 0.7254 | 0.6127 | +0.1127 | false |
| homogeneous_1000 | control | S5_keys_intern | 1.1474 | 0.9276 | +0.2198 | false |
| homogeneous_1000 | control | S6_tabular_intern | 0.5615 | 0.4746 | +0.0869 | false |
| homogeneous_1000 | control | S7_full | 0.5619 | 0.4750 | +0.0869 | false |
| homogeneous_1000 | printable | S0_none | 1.0003 | 1.0003 | +0.0001 | true |
| homogeneous_1000 | printable | S1_keys | 1.0695 | 0.8711 | +0.1984 | false |
| homogeneous_1000 | printable | S2_tabular | 0.7249 | 0.6123 | +0.1126 | false |
| homogeneous_1000 | printable | S3_intern | 0.7039 | 0.6683 | +0.0356 | false |
| homogeneous_1000 | printable | S4_keys_tabular | 0.6908 | 0.6125 | +0.0783 | false |
| homogeneous_1000 | printable | S5_keys_intern | 0.7731 | 0.5391 | +0.2339 | false |
| homogeneous_1000 | printable | S6_tabular_intern | 0.4285 | 0.2803 | +0.1481 | false |
| homogeneous_1000 | printable | S7_full | 0.3943 | 0.2805 | +0.1138 | false |
| homogeneous_1000 | bare | S0_none | 1.0003 | 1.0003 | +0.0001 | true |
| homogeneous_1000 | bare | S1_keys | 1.0003 | 1.0003 | +0.0001 | true |
| homogeneous_1000 | bare | S2_tabular | 0.7249 | 0.6123 | +0.1126 | false |
| homogeneous_1000 | bare | S3_intern | 0.6005 | 0.6295 | -0.0290 | false |
| homogeneous_1000 | bare | S4_keys_tabular | 0.7249 | 0.6123 | +0.1126 | false |
| homogeneous_1000 | bare | S5_keys_intern | 0.6005 | 0.6295 | -0.0290 | false |
| homogeneous_1000 | bare | S6_tabular_intern | 0.3250 | 0.2415 | +0.0835 | false |
| homogeneous_1000 | bare | S7_full | 0.3250 | 0.2415 | +0.0835 | false |
| homogeneous_1000 | common_word | S0_none | 1.0003 | 1.0003 | +0.0001 | true |
| homogeneous_1000 | common_word | S1_keys | 0.9661 | 0.8193 | +0.1468 | false |
| homogeneous_1000 | common_word | S2_tabular | 0.7249 | 0.6123 | +0.1126 | false |
| homogeneous_1000 | common_word | S3_intern | 0.6065 | 0.6683 | -0.0618 | false |
| homogeneous_1000 | common_word | S4_keys_tabular | 0.7251 | 0.6125 | +0.1126 | false |
| homogeneous_1000 | common_word | S5_keys_intern | 0.5723 | 0.4874 | +0.0849 | false |
| homogeneous_1000 | common_word | S6_tabular_intern | 0.3311 | 0.2803 | +0.0507 | false |
| homogeneous_1000 | common_word | S7_full | 0.3313 | 0.2805 | +0.0508 | false |
| homogeneous_1000 | derived_cl100k_v1 | S0_none | 1.0005 | 1.0004 | +0.0001 | true |
| homogeneous_1000 | derived_cl100k_v1 | S1_keys | 0.8973 | 0.8324 | +0.0649 | false |
| homogeneous_1000 | derived_cl100k_v1 | S2_tabular | 0.7251 | 0.6124 | +0.1126 | false |
| homogeneous_1000 | derived_cl100k_v1 | S3_intern | 0.5866 | 0.6339 | -0.0473 | false |
| homogeneous_1000 | derived_cl100k_v1 | S4_keys_tabular | 0.7252 | 0.6126 | +0.1126 | false |
| homogeneous_1000 | derived_cl100k_v1 | S5_keys_intern | 0.4950 | 0.4659 | +0.0291 | false |
| homogeneous_1000 | derived_cl100k_v1 | S6_tabular_intern | 0.2892 | 0.2459 | +0.0433 | false |
| homogeneous_1000 | derived_cl100k_v1 | S7_full | 0.2893 | 0.2461 | +0.0432 | false |
| homogeneous_1000 | derived_o200k_v1 | S0_none | 1.0006 | 1.0004 | +0.0002 | true |
| homogeneous_1000 | derived_o200k_v1 | S1_keys | 0.8974 | 0.8324 | +0.0650 | false |
| homogeneous_1000 | derived_o200k_v1 | S2_tabular | 0.7251 | 0.6124 | +0.1127 | false |
| homogeneous_1000 | derived_o200k_v1 | S3_intern | 0.5867 | 0.6339 | -0.0472 | false |
| homogeneous_1000 | derived_o200k_v1 | S4_keys_tabular | 0.7253 | 0.6126 | +0.1127 | false |
| homogeneous_1000 | derived_o200k_v1 | S5_keys_intern | 0.4951 | 0.4659 | +0.0291 | false |
| homogeneous_1000 | derived_o200k_v1 | S6_tabular_intern | 0.2892 | 0.2459 | +0.0433 | false |
| homogeneous_1000 | derived_o200k_v1 | S7_full | 0.2894 | 0.2461 | +0.0433 | false |
| homogeneous_1000 | derived_llama_v1 | S0_none | 1.0006 | 1.0004 | +0.0002 | true |
| homogeneous_1000 | derived_llama_v1 | S1_keys | 1.0006 | 1.0004 | +0.0002 | true |
| homogeneous_1000 | derived_llama_v1 | S2_tabular | 0.7251 | 0.6124 | +0.1127 | false |
| homogeneous_1000 | derived_llama_v1 | S3_intern | 0.5982 | 0.6339 | -0.0357 | false |
| homogeneous_1000 | derived_llama_v1 | S4_keys_tabular | 0.7251 | 0.6124 | +0.1127 | false |
| homogeneous_1000 | derived_llama_v1 | S5_keys_intern | 0.5982 | 0.6339 | -0.0357 | false |
| homogeneous_1000 | derived_llama_v1 | S6_tabular_intern | 0.3006 | 0.2459 | +0.0546 | false |
| homogeneous_1000 | derived_llama_v1 | S7_full | 0.3006 | 0.2459 | +0.0546 | false |
| heterogeneous_nested | control | S0_none | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | control | S1_keys | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | control | S2_tabular | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | control | S3_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | control | S4_keys_tabular | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | control | S5_keys_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | control | S6_tabular_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | control | S7_full | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | printable | S0_none | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | printable | S1_keys | 1.1410 | 0.8971 | +0.2439 | false |
| heterogeneous_nested | printable | S2_tabular | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | printable | S3_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | printable | S4_keys_tabular | 1.1410 | 0.8971 | +0.2439 | false |
| heterogeneous_nested | printable | S5_keys_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | printable | S6_tabular_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | printable | S7_full | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S0_none | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S1_keys | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S2_tabular | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S3_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S4_keys_tabular | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S5_keys_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S6_tabular_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | bare | S7_full | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | common_word | S0_none | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | common_word | S1_keys | 1.0196 | 0.8506 | +0.1689 | false |
| heterogeneous_nested | common_word | S2_tabular | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | common_word | S3_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | common_word | S4_keys_tabular | 1.0196 | 0.8506 | +0.1689 | false |
| heterogeneous_nested | common_word | S5_keys_intern | 0.9608 | 0.8212 | +0.1397 | false |
| heterogeneous_nested | common_word | S6_tabular_intern | 1.0131 | 1.0100 | +0.0031 | true |
| heterogeneous_nested | common_word | S7_full | 0.9608 | 0.8212 | +0.1397 | false |
| heterogeneous_nested | derived_cl100k_v1 | S0_none | 1.0196 | 1.0145 | +0.0051 | true |
| heterogeneous_nested | derived_cl100k_v1 | S1_keys | 0.9856 | 0.8706 | +0.1150 | false |
| heterogeneous_nested | derived_cl100k_v1 | S2_tabular | 1.0196 | 1.0145 | +0.0051 | true |
| heterogeneous_nested | derived_cl100k_v1 | S3_intern | 1.0196 | 1.0145 | +0.0051 | true |
| heterogeneous_nested | derived_cl100k_v1 | S4_keys_tabular | 0.9856 | 0.8706 | +0.1150 | false |
| heterogeneous_nested | derived_cl100k_v1 | S5_keys_intern | 0.9138 | 0.8287 | +0.0852 | false |
| heterogeneous_nested | derived_cl100k_v1 | S6_tabular_intern | 1.0196 | 1.0145 | +0.0051 | true |
| heterogeneous_nested | derived_cl100k_v1 | S7_full | 0.9138 | 0.8287 | +0.0852 | false |
| heterogeneous_nested | derived_o200k_v1 | S0_none | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_o200k_v1 | S1_keys | 0.9883 | 0.8706 | +0.1176 | false |
| heterogeneous_nested | derived_o200k_v1 | S2_tabular | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_o200k_v1 | S3_intern | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_o200k_v1 | S4_keys_tabular | 0.9883 | 0.8706 | +0.1176 | false |
| heterogeneous_nested | derived_o200k_v1 | S5_keys_intern | 0.9164 | 0.8287 | +0.0878 | false |
| heterogeneous_nested | derived_o200k_v1 | S6_tabular_intern | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_o200k_v1 | S7_full | 0.9164 | 0.8287 | +0.0878 | false |
| heterogeneous_nested | derived_llama_v1 | S0_none | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_llama_v1 | S1_keys | 1.0287 | 0.8706 | +0.1581 | false |
| heterogeneous_nested | derived_llama_v1 | S2_tabular | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_llama_v1 | S3_intern | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_llama_v1 | S4_keys_tabular | 1.0287 | 0.8706 | +0.1581 | false |
| heterogeneous_nested | derived_llama_v1 | S5_keys_intern | 0.9700 | 0.8312 | +0.1388 | false |
| heterogeneous_nested | derived_llama_v1 | S6_tabular_intern | 1.0222 | 1.0145 | +0.0077 | true |
| heterogeneous_nested | derived_llama_v1 | S7_full | 0.9700 | 0.8312 | +0.1388 | false |
| realistic_api | control | S0_none | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | control | S1_keys | 1.2100 | 0.9950 | +0.2150 | false |
| realistic_api | control | S2_tabular | 0.8725 | 0.7916 | +0.0809 | false |
| realistic_api | control | S3_intern | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | control | S4_keys_tabular | 0.9694 | 0.7886 | +0.1807 | false |
| realistic_api | control | S5_keys_intern | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | control | S6_tabular_intern | 0.9803 | 0.8375 | +0.1429 | false |
| realistic_api | control | S7_full | 1.0772 | 0.8345 | +0.2427 | false |
| realistic_api | printable | S0_none | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | printable | S1_keys | 1.0600 | 0.8031 | +0.2569 | false |
| realistic_api | printable | S2_tabular | 0.8716 | 0.7911 | +0.0805 | false |
| realistic_api | printable | S3_intern | 0.9934 | 0.9201 | +0.0734 | false |
| realistic_api | printable | S4_keys_tabular | 0.9041 | 0.7002 | +0.2039 | false |
| realistic_api | printable | S5_keys_intern | 1.0503 | 0.7211 | +0.3293 | false |
| realistic_api | printable | S6_tabular_intern | 0.8619 | 0.7090 | +0.1529 | false |
| realistic_api | printable | S7_full | 0.8944 | 0.6181 | +0.2763 | false |
| realistic_api | bare | S0_none | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | bare | S1_keys | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | bare | S2_tabular | 0.8716 | 0.7912 | +0.0804 | false |
| realistic_api | bare | S3_intern | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | bare | S4_keys_tabular | 0.8716 | 0.7912 | +0.0804 | false |
| realistic_api | bare | S5_keys_intern | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | bare | S6_tabular_intern | 0.8716 | 0.7912 | +0.0804 | false |
| realistic_api | bare | S7_full | 0.8716 | 0.7912 | +0.0804 | false |
| realistic_api | common_word | S0_none | 1.0031 | 1.0021 | +0.0010 | true |
| realistic_api | common_word | S1_keys | 0.9475 | 0.7648 | +0.1827 | false |
| realistic_api | common_word | S2_tabular | 0.8716 | 0.7912 | +0.0804 | false |
| realistic_api | common_word | S3_intern | 0.9216 | 0.9201 | +0.0015 | false |
| realistic_api | common_word | S4_keys_tabular | 0.8525 | 0.6827 | +0.1698 | false |
| realistic_api | common_word | S5_keys_intern | 0.8659 | 0.6827 | +0.1832 | false |
| realistic_api | common_word | S6_tabular_intern | 0.7900 | 0.7091 | +0.0809 | false |
| realistic_api | common_word | S7_full | 0.7709 | 0.6006 | +0.1703 | false |
| realistic_api | derived_cl100k_v1 | S0_none | 1.0047 | 1.0031 | +0.0016 | true |
| realistic_api | derived_cl100k_v1 | S1_keys | 0.9116 | 0.7700 | +0.1416 | false |
| realistic_api | derived_cl100k_v1 | S2_tabular | 0.8731 | 0.7922 | +0.0810 | false |
| realistic_api | derived_cl100k_v1 | S3_intern | 0.8909 | 0.8997 | -0.0088 | false |
| realistic_api | derived_cl100k_v1 | S4_keys_tabular | 0.8413 | 0.6838 | +0.1575 | false |
| realistic_api | derived_cl100k_v1 | S5_keys_intern | 0.7966 | 0.6666 | +0.1300 | false |
| realistic_api | derived_cl100k_v1 | S6_tabular_intern | 0.7547 | 0.6888 | +0.0659 | false |
| realistic_api | derived_cl100k_v1 | S7_full | 0.7216 | 0.5804 | +0.1412 | false |
| realistic_api | derived_o200k_v1 | S0_none | 1.0053 | 1.0031 | +0.0022 | true |
| realistic_api | derived_o200k_v1 | S1_keys | 0.9122 | 0.7700 | +0.1422 | false |
| realistic_api | derived_o200k_v1 | S2_tabular | 0.8738 | 0.7922 | +0.0816 | false |
| realistic_api | derived_o200k_v1 | S3_intern | 0.8916 | 0.8997 | -0.0081 | false |
| realistic_api | derived_o200k_v1 | S4_keys_tabular | 0.8419 | 0.6838 | +0.1581 | false |
| realistic_api | derived_o200k_v1 | S5_keys_intern | 0.7972 | 0.6666 | +0.1306 | false |
| realistic_api | derived_o200k_v1 | S6_tabular_intern | 0.7553 | 0.6888 | +0.0665 | false |
| realistic_api | derived_o200k_v1 | S7_full | 0.7222 | 0.5804 | +0.1418 | false |
| realistic_api | derived_llama_v1 | S0_none | 1.0053 | 1.0031 | +0.0022 | true |
| realistic_api | derived_llama_v1 | S1_keys | 0.9472 | 0.7700 | +0.1772 | false |
| realistic_api | derived_llama_v1 | S2_tabular | 0.8738 | 0.7922 | +0.0816 | false |
| realistic_api | derived_llama_v1 | S3_intern | 0.9034 | 0.8997 | +0.0037 | false |
| realistic_api | derived_llama_v1 | S4_keys_tabular | 0.8544 | 0.6879 | +0.1665 | false |
| realistic_api | derived_llama_v1 | S5_keys_intern | 0.8453 | 0.6666 | +0.1787 | false |
| realistic_api | derived_llama_v1 | S6_tabular_intern | 0.7675 | 0.6888 | +0.0787 | false |
| realistic_api | derived_llama_v1 | S7_full | 0.7481 | 0.5845 | +0.1636 | false |
| prose_heavy | control | S0_none | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | control | S1_keys | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | control | S2_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | control | S3_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | control | S4_keys_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | control | S5_keys_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | control | S6_tabular_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | control | S7_full | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S0_none | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S1_keys | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S2_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S3_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S4_keys_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S5_keys_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S6_tabular_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | printable | S7_full | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S0_none | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S1_keys | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S2_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S3_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S4_keys_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S5_keys_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S6_tabular_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | bare | S7_full | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S0_none | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S1_keys | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S2_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S3_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S4_keys_tabular | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S5_keys_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S6_tabular_intern | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | common_word | S7_full | 1.0034 | 1.0014 | +0.0020 | true |
| prose_heavy | derived_cl100k_v1 | S0_none | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_cl100k_v1 | S1_keys | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_cl100k_v1 | S2_tabular | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_cl100k_v1 | S3_intern | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_cl100k_v1 | S4_keys_tabular | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_cl100k_v1 | S5_keys_intern | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_cl100k_v1 | S6_tabular_intern | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_cl100k_v1 | S7_full | 1.0051 | 1.0020 | +0.0031 | true |
| prose_heavy | derived_o200k_v1 | S0_none | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_o200k_v1 | S1_keys | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_o200k_v1 | S2_tabular | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_o200k_v1 | S3_intern | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_o200k_v1 | S4_keys_tabular | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_o200k_v1 | S5_keys_intern | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_o200k_v1 | S6_tabular_intern | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_o200k_v1 | S7_full | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S0_none | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S1_keys | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S2_tabular | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S3_intern | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S4_keys_tabular | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S5_keys_intern | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S6_tabular_intern | 1.0057 | 1.0020 | +0.0037 | true |
| prose_heavy | derived_llama_v1 | S7_full | 1.0057 | 1.0020 | +0.0037 | true |
| size_1k | control | S0_none | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | control | S1_keys | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | control | S2_tabular | 0.7895 | 0.6684 | +0.1211 | false |
| size_1k | control | S3_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | control | S4_keys_tabular | 0.8211 | 0.6979 | +0.1232 | false |
| size_1k | control | S5_keys_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | control | S6_tabular_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | control | S7_full | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | printable | S0_none | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | printable | S1_keys | 1.1079 | 0.9125 | +0.1954 | false |
| size_1k | printable | S2_tabular | 0.7816 | 0.6633 | +0.1183 | false |
| size_1k | printable | S3_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | printable | S4_keys_tabular | 0.7737 | 0.6775 | +0.0962 | false |
| size_1k | printable | S5_keys_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | printable | S6_tabular_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | printable | S7_full | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | bare | S0_none | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | bare | S1_keys | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | bare | S2_tabular | 0.7816 | 0.6643 | +0.1173 | false |
| size_1k | bare | S3_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | bare | S4_keys_tabular | 0.7816 | 0.6643 | +0.1173 | false |
| size_1k | bare | S5_keys_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | bare | S6_tabular_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | bare | S7_full | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | common_word | S0_none | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | common_word | S1_keys | 1.0105 | 0.8647 | +0.1458 | false |
| size_1k | common_word | S2_tabular | 0.7816 | 0.6643 | +0.1173 | false |
| size_1k | common_word | S3_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | common_word | S4_keys_tabular | 0.8000 | 0.6796 | +0.1204 | false |
| size_1k | common_word | S5_keys_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | common_word | S6_tabular_intern | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | common_word | S7_full | 1.0237 | 1.0203 | +0.0033 | true |
| size_1k | derived_cl100k_v1 | S0_none | 1.0368 | 1.0295 | +0.0073 | true |
| size_1k | derived_cl100k_v1 | S1_keys | 0.9553 | 0.8871 | +0.0682 | false |
| size_1k | derived_cl100k_v1 | S2_tabular | 0.7947 | 0.6734 | +0.1213 | false |
| size_1k | derived_cl100k_v1 | S3_intern | 1.0368 | 1.0295 | +0.0073 | true |
| size_1k | derived_cl100k_v1 | S4_keys_tabular | 0.8079 | 0.6897 | +0.1182 | false |
| size_1k | derived_cl100k_v1 | S5_keys_intern | 1.0368 | 1.0295 | +0.0073 | true |
| size_1k | derived_cl100k_v1 | S6_tabular_intern | 1.0368 | 1.0295 | +0.0073 | true |
| size_1k | derived_cl100k_v1 | S7_full | 1.0368 | 1.0295 | +0.0073 | true |
| size_1k | derived_o200k_v1 | S0_none | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_o200k_v1 | S1_keys | 0.9605 | 0.8871 | +0.0734 | false |
| size_1k | derived_o200k_v1 | S2_tabular | 0.8000 | 0.6734 | +0.1266 | false |
| size_1k | derived_o200k_v1 | S3_intern | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_o200k_v1 | S4_keys_tabular | 0.8132 | 0.6897 | +0.1234 | false |
| size_1k | derived_o200k_v1 | S5_keys_intern | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_o200k_v1 | S6_tabular_intern | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_o200k_v1 | S7_full | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_llama_v1 | S0_none | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_llama_v1 | S1_keys | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_llama_v1 | S2_tabular | 0.8000 | 0.6734 | +0.1266 | false |
| size_1k | derived_llama_v1 | S3_intern | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_llama_v1 | S4_keys_tabular | 0.8000 | 0.6734 | +0.1266 | false |
| size_1k | derived_llama_v1 | S5_keys_intern | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_llama_v1 | S6_tabular_intern | 1.0421 | 1.0295 | +0.0126 | true |
| size_1k | derived_llama_v1 | S7_full | 1.0421 | 1.0295 | +0.0126 | true |
| size_10k | control | S0_none | 1.0023 | 1.0020 | +0.0004 | true |
| size_10k | control | S1_keys | 1.0023 | 1.0020 | +0.0004 | true |
| size_10k | control | S2_tabular | 0.7306 | 0.6142 | +0.1164 | false |
| size_10k | control | S3_intern | 1.0326 | 0.9225 | +0.1102 | false |
| size_10k | control | S4_keys_tabular | 0.7337 | 0.6170 | +0.1166 | false |
| size_10k | control | S5_keys_intern | 1.0023 | 1.0020 | +0.0004 | true |
| size_10k | control | S6_tabular_intern | 0.6575 | 0.5347 | +0.1228 | false |
| size_10k | control | S7_full | 0.6606 | 0.5375 | +0.1231 | false |
| size_10k | printable | S0_none | 1.0023 | 1.0020 | +0.0004 | true |
| size_10k | printable | S1_keys | 1.0728 | 0.8736 | +0.1992 | false |
| size_10k | printable | S2_tabular | 0.7298 | 0.6137 | +0.1161 | false |
| size_10k | printable | S3_intern | 0.7966 | 0.7264 | +0.0702 | false |
| size_10k | printable | S4_keys_tabular | 0.6979 | 0.6151 | +0.0829 | false |
| size_10k | printable | S5_keys_intern | 0.8671 | 0.5981 | +0.2690 | false |
| size_10k | printable | S6_tabular_intern | 0.5241 | 0.3381 | +0.1859 | false |
| size_10k | printable | S7_full | 0.4922 | 0.3395 | +0.1527 | false |
| size_10k | bare | S0_none | 1.0023 | 1.0020 | +0.0004 | true |
| size_10k | bare | S1_keys | 1.0023 | 1.0020 | +0.0004 | true |
| size_10k | bare | S2_tabular | 0.7298 | 0.6138 | +0.1160 | false |
| size_10k | bare | S3_intern | 0.6933 | 0.6872 | +0.0061 | false |
| size_10k | bare | S4_keys_tabular | 0.7298 | 0.6138 | +0.1160 | false |
| size_10k | bare | S5_keys_intern | 0.6933 | 0.6872 | +0.0061 | false |
| size_10k | bare | S6_tabular_intern | 0.4207 | 0.2990 | +0.1217 | false |
| size_10k | bare | S7_full | 0.4207 | 0.2990 | +0.1217 | false |
| size_10k | common_word | S0_none | 1.0023 | 1.0020 | +0.0004 | true |
| size_10k | common_word | S1_keys | 0.9699 | 0.8218 | +0.1481 | false |
| size_10k | common_word | S2_tabular | 0.7298 | 0.6138 | +0.1160 | false |
| size_10k | common_word | S3_intern | 0.6984 | 0.7264 | -0.0280 | false |
| size_10k | common_word | S4_keys_tabular | 0.7316 | 0.6153 | +0.1163 | false |
| size_10k | common_word | S5_keys_intern | 0.6661 | 0.5463 | +0.1198 | false |
| size_10k | common_word | S6_tabular_intern | 0.4259 | 0.3382 | +0.0877 | false |
| size_10k | common_word | S7_full | 0.4277 | 0.3397 | +0.0880 | false |
| size_10k | derived_cl100k_v1 | S0_none | 1.0036 | 1.0028 | +0.0008 | true |
| size_10k | derived_cl100k_v1 | S1_keys | 0.9023 | 0.8358 | +0.0665 | false |
| size_10k | derived_cl100k_v1 | S2_tabular | 0.7311 | 0.6147 | +0.1164 | false |
| size_10k | derived_cl100k_v1 | S3_intern | 0.6801 | 0.6924 | -0.0124 | false |
| size_10k | derived_cl100k_v1 | S4_keys_tabular | 0.7324 | 0.6163 | +0.1161 | false |
| size_10k | derived_cl100k_v1 | S5_keys_intern | 0.5904 | 0.5254 | +0.0651 | false |
| size_10k | derived_cl100k_v1 | S6_tabular_intern | 0.3868 | 0.3042 | +0.0825 | false |
| size_10k | derived_cl100k_v1 | S7_full | 0.3881 | 0.3058 | +0.0823 | false |
| size_10k | derived_o200k_v1 | S0_none | 1.0041 | 1.0028 | +0.0013 | true |
| size_10k | derived_o200k_v1 | S1_keys | 0.9028 | 0.8358 | +0.0671 | false |
| size_10k | derived_o200k_v1 | S2_tabular | 0.7316 | 0.6147 | +0.1169 | false |
| size_10k | derived_o200k_v1 | S3_intern | 0.6806 | 0.6924 | -0.0118 | false |
| size_10k | derived_o200k_v1 | S4_keys_tabular | 0.7329 | 0.6163 | +0.1166 | false |
| size_10k | derived_o200k_v1 | S5_keys_intern | 0.5909 | 0.5254 | +0.0656 | false |
| size_10k | derived_o200k_v1 | S6_tabular_intern | 0.3873 | 0.3042 | +0.0831 | false |
| size_10k | derived_o200k_v1 | S7_full | 0.3886 | 0.3058 | +0.0828 | false |
| size_10k | derived_llama_v1 | S0_none | 1.0041 | 1.0028 | +0.0013 | true |
| size_10k | derived_llama_v1 | S1_keys | 1.0041 | 1.0028 | +0.0013 | true |
| size_10k | derived_llama_v1 | S2_tabular | 0.7316 | 0.6147 | +0.1169 | false |
| size_10k | derived_llama_v1 | S3_intern | 0.6925 | 0.6925 | -0.0000 | false |
| size_10k | derived_llama_v1 | S4_keys_tabular | 0.7316 | 0.6147 | +0.1169 | false |
| size_10k | derived_llama_v1 | S5_keys_intern | 0.6925 | 0.6925 | -0.0000 | false |
| size_10k | derived_llama_v1 | S6_tabular_intern | 0.3972 | 0.3043 | +0.0928 | false |
| size_10k | derived_llama_v1 | S7_full | 0.3972 | 0.3043 | +0.0928 | false |
| size_100k | control | S0_none | 1.0002 | 1.0002 | +0.0000 | true |
| size_100k | control | S1_keys | 1.0002 | 1.0002 | +0.0000 | true |
| size_100k | control | S2_tabular | 0.7273 | 0.6136 | +0.1137 | false |
| size_100k | control | S3_intern | 0.9371 | 0.8606 | +0.0765 | false |
| size_100k | control | S4_keys_tabular | 0.7276 | 0.6139 | +0.1137 | false |
| size_100k | control | S5_keys_intern | 1.1422 | 0.9253 | +0.2169 | false |
| size_100k | control | S6_tabular_intern | 0.5616 | 0.4740 | +0.0876 | false |
| size_100k | control | S7_full | 0.5619 | 0.4743 | +0.0877 | false |
| size_100k | printable | S0_none | 1.0002 | 1.0002 | +0.0000 | true |
| size_100k | printable | S1_keys | 1.0687 | 0.8714 | +0.1973 | false |
| size_100k | printable | S2_tabular | 0.7272 | 0.6136 | +0.1136 | false |
| size_100k | printable | S3_intern | 0.7028 | 0.6671 | +0.0357 | false |
| size_100k | printable | S4_keys_tabular | 0.6933 | 0.6137 | +0.0796 | false |
| size_100k | printable | S5_keys_intern | 0.7713 | 0.5383 | +0.2330 | false |
| size_100k | printable | S6_tabular_intern | 0.4298 | 0.2805 | +0.1493 | false |
| size_100k | printable | S7_full | 0.3959 | 0.2806 | +0.1153 | false |
| size_100k | bare | S0_none | 1.0002 | 1.0002 | +0.0000 | true |
| size_100k | bare | S1_keys | 1.0002 | 1.0002 | +0.0000 | true |
| size_100k | bare | S2_tabular | 0.7272 | 0.6136 | +0.1136 | false |
| size_100k | bare | S3_intern | 0.6003 | 0.6284 | -0.0281 | false |
| size_100k | bare | S4_keys_tabular | 0.7272 | 0.6136 | +0.1136 | false |
| size_100k | bare | S5_keys_intern | 0.6003 | 0.6284 | -0.0281 | false |
| size_100k | bare | S6_tabular_intern | 0.3273 | 0.2418 | +0.0855 | false |
| size_100k | bare | S7_full | 0.3273 | 0.2418 | +0.0855 | false |
| size_100k | common_word | S0_none | 1.0002 | 1.0002 | +0.0000 | true |
| size_100k | common_word | S1_keys | 0.9663 | 0.8199 | +0.1464 | false |
| size_100k | common_word | S2_tabular | 0.7272 | 0.6136 | +0.1136 | false |
| size_100k | common_word | S3_intern | 0.6063 | 0.6671 | -0.0608 | false |
| size_100k | common_word | S4_keys_tabular | 0.7274 | 0.6137 | +0.1137 | false |
| size_100k | common_word | S5_keys_intern | 0.5724 | 0.4868 | +0.0856 | false |
| size_100k | common_word | S6_tabular_intern | 0.3333 | 0.2805 | +0.0528 | false |
| size_100k | common_word | S7_full | 0.3335 | 0.2806 | +0.0529 | false |
| size_100k | derived_cl100k_v1 | S0_none | 1.0004 | 1.0003 | +0.0001 | true |
| size_100k | derived_cl100k_v1 | S1_keys | 0.8981 | 0.8329 | +0.0652 | false |
| size_100k | derived_cl100k_v1 | S2_tabular | 0.7273 | 0.6137 | +0.1137 | false |
| size_100k | derived_cl100k_v1 | S3_intern | 0.5866 | 0.6328 | -0.0462 | false |
| size_100k | derived_cl100k_v1 | S4_keys_tabular | 0.7275 | 0.6138 | +0.1136 | false |
| size_100k | derived_cl100k_v1 | S5_keys_intern | 0.4957 | 0.4654 | +0.0304 | false |
| size_100k | derived_cl100k_v1 | S6_tabular_intern | 0.2917 | 0.2462 | +0.0455 | false |
| size_100k | derived_cl100k_v1 | S7_full | 0.2918 | 0.2463 | +0.0455 | false |
| size_100k | derived_o200k_v1 | S0_none | 1.0004 | 1.0003 | +0.0001 | true |
| size_100k | derived_o200k_v1 | S1_keys | 0.8981 | 0.8329 | +0.0653 | false |
| size_100k | derived_o200k_v1 | S2_tabular | 0.7274 | 0.6137 | +0.1137 | false |
| size_100k | derived_o200k_v1 | S3_intern | 0.5866 | 0.6328 | -0.0461 | false |
| size_100k | derived_o200k_v1 | S4_keys_tabular | 0.7275 | 0.6138 | +0.1137 | false |
| size_100k | derived_o200k_v1 | S5_keys_intern | 0.4958 | 0.4654 | +0.0304 | false |
| size_100k | derived_o200k_v1 | S6_tabular_intern | 0.2917 | 0.2462 | +0.0456 | false |
| size_100k | derived_o200k_v1 | S7_full | 0.2919 | 0.2463 | +0.0456 | false |
| size_100k | derived_llama_v1 | S0_none | 1.0004 | 1.0003 | +0.0001 | true |
| size_100k | derived_llama_v1 | S1_keys | 1.0004 | 1.0003 | +0.0001 | true |
| size_100k | derived_llama_v1 | S2_tabular | 0.7274 | 0.6137 | +0.1137 | false |
| size_100k | derived_llama_v1 | S3_intern | 0.5981 | 0.6328 | -0.0347 | false |
| size_100k | derived_llama_v1 | S4_keys_tabular | 0.7274 | 0.6137 | +0.1137 | false |
| size_100k | derived_llama_v1 | S5_keys_intern | 0.5981 | 0.6328 | -0.0347 | false |
| size_100k | derived_llama_v1 | S6_tabular_intern | 0.3031 | 0.2462 | +0.0569 | false |
| size_100k | derived_llama_v1 | S7_full | 0.3031 | 0.2462 | +0.0569 | false |
| size_250k | control | S0_none | 1.0001 | 1.0001 | +0.0000 | true |
| size_250k | control | S1_keys | 1.0001 | 1.0001 | +0.0000 | true |
| size_250k | control | S2_tabular | 0.7309 | 0.6156 | +0.1153 | false |
| size_250k | control | S3_intern | 0.9320 | 0.8575 | +0.0745 | false |
| size_250k | control | S4_keys_tabular | 0.7310 | 0.6157 | +0.1153 | false |
| size_250k | control | S5_keys_intern | 1.1341 | 0.9217 | +0.2124 | false |
| size_250k | control | S6_tabular_intern | 0.5618 | 0.4730 | +0.0888 | false |
| size_250k | control | S7_full | 0.5619 | 0.4731 | +0.0888 | false |
| size_250k | printable | S0_none | 1.0001 | 1.0001 | +0.0000 | true |
| size_250k | printable | S1_keys | 1.0675 | 0.8720 | +0.1955 | false |
| size_250k | printable | S2_tabular | 0.7308 | 0.6156 | +0.1152 | false |
| size_250k | printable | S3_intern | 0.7011 | 0.6652 | +0.0360 | false |
| size_250k | printable | S4_keys_tabular | 0.6973 | 0.6157 | +0.0816 | false |
| size_250k | printable | S5_keys_intern | 0.7685 | 0.5370 | +0.2315 | false |
| size_250k | printable | S6_tabular_intern | 0.4319 | 0.2807 | +0.1512 | false |
| size_250k | printable | S7_full | 0.3983 | 0.2807 | +0.1176 | false |
| size_250k | bare | S0_none | 1.0001 | 1.0001 | +0.0000 | true |
| size_250k | bare | S1_keys | 1.0001 | 1.0001 | +0.0000 | true |
| size_250k | bare | S2_tabular | 0.7308 | 0.6156 | +0.1152 | false |
| size_250k | bare | S3_intern | 0.6001 | 0.6267 | -0.0266 | false |
| size_250k | bare | S4_keys_tabular | 0.7308 | 0.6156 | +0.1152 | false |
| size_250k | bare | S5_keys_intern | 0.6001 | 0.6267 | -0.0266 | false |
| size_250k | bare | S6_tabular_intern | 0.3309 | 0.2422 | +0.0887 | false |
| size_250k | bare | S7_full | 0.3309 | 0.2422 | +0.0887 | false |
| size_250k | common_word | S0_none | 1.0001 | 1.0001 | +0.0000 | true |
| size_250k | common_word | S1_keys | 0.9665 | 0.8207 | +0.1458 | false |
| size_250k | common_word | S2_tabular | 0.7308 | 0.6156 | +0.1152 | false |
| size_250k | common_word | S3_intern | 0.6061 | 0.6652 | -0.0590 | false |
| size_250k | common_word | S4_keys_tabular | 0.7309 | 0.6157 | +0.1152 | false |
| size_250k | common_word | S5_keys_intern | 0.5725 | 0.4858 | +0.0867 | false |
| size_250k | common_word | S6_tabular_intern | 0.3369 | 0.2807 | +0.0562 | false |
| size_250k | common_word | S7_full | 0.3369 | 0.2808 | +0.0562 | false |
| size_250k | derived_cl100k_v1 | S0_none | 1.0001 | 1.0001 | +0.0000 | true |
| size_250k | derived_cl100k_v1 | S1_keys | 0.8992 | 0.8336 | +0.0657 | false |
| size_250k | derived_cl100k_v1 | S2_tabular | 0.7309 | 0.6156 | +0.1153 | false |
| size_250k | derived_cl100k_v1 | S3_intern | 0.5865 | 0.6310 | -0.0445 | false |
| size_250k | derived_cl100k_v1 | S4_keys_tabular | 0.7310 | 0.6157 | +0.1152 | false |
| size_250k | derived_cl100k_v1 | S5_keys_intern | 0.4968 | 0.4645 | +0.0324 | false |
| size_250k | derived_cl100k_v1 | S6_tabular_intern | 0.2957 | 0.2465 | +0.0491 | false |
| size_250k | derived_cl100k_v1 | S7_full | 0.2957 | 0.2466 | +0.0491 | false |
| size_250k | derived_o200k_v1 | S0_none | 1.0002 | 1.0001 | +0.0001 | true |
| size_250k | derived_o200k_v1 | S1_keys | 0.8992 | 0.8336 | +0.0657 | false |
| size_250k | derived_o200k_v1 | S2_tabular | 0.7309 | 0.6156 | +0.1153 | false |
| size_250k | derived_o200k_v1 | S3_intern | 0.5866 | 0.6310 | -0.0444 | false |
| size_250k | derived_o200k_v1 | S4_keys_tabular | 0.7310 | 0.6157 | +0.1153 | false |
| size_250k | derived_o200k_v1 | S5_keys_intern | 0.4969 | 0.4645 | +0.0324 | false |
| size_250k | derived_o200k_v1 | S6_tabular_intern | 0.2957 | 0.2465 | +0.0492 | false |
| size_250k | derived_o200k_v1 | S7_full | 0.2958 | 0.2466 | +0.0492 | false |
| size_250k | derived_llama_v1 | S0_none | 1.0002 | 1.0001 | +0.0001 | true |
| size_250k | derived_llama_v1 | S1_keys | 1.0002 | 1.0001 | +0.0001 | true |
| size_250k | derived_llama_v1 | S2_tabular | 0.7309 | 0.6156 | +0.1153 | false |
| size_250k | derived_llama_v1 | S3_intern | 0.5978 | 0.6310 | -0.0332 | false |
| size_250k | derived_llama_v1 | S4_keys_tabular | 0.7309 | 0.6156 | +0.1153 | false |
| size_250k | derived_llama_v1 | S5_keys_intern | 0.5978 | 0.6310 | -0.0332 | false |
| size_250k | derived_llama_v1 | S6_tabular_intern | 0.3069 | 0.2465 | +0.0603 | false |
| size_250k | derived_llama_v1 | S7_full | 0.3069 | 0.2465 | +0.0603 | false |
