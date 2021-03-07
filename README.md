# Barad-dûr
matrix phone-home stats collector 

## import stats from panopticon
for panopticon version lesser or equal v0.1.2 import csv via
```shell
cat dump.csv | psql <dburl> -f scripts/import-v0.1.2.sql
```
for panopticon v0.1.3 or v0.1.4 use
```shell
cat dump.csv | psql <dburl> -f scripts/import-v0.1.3.sql
```
for panopticon > v0.1.4 use
```shell
cat dump.csv | psql <dburl> -f scripts/import-v0.1.5.sql
```