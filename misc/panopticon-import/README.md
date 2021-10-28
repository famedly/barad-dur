## Import reports from panopticon

Barad-dûr comes with import scripts for migrating from panopticon.
For using those, you need to:

1. Dump the table with existing reports (just the raw stats, not the aggregated
   stats) into a csv file, like so: `select * from stats INTO OUTFILE
   '/tmp/dump.csv' FIELDS TERMINATED BY ',' ENCLOSED BY '"' LINES TERMINATED
   BY '\n';`

2. Start Barad-dûr temporarily to ensure the database schema is in place. Stop
   Barad-dûr again after this.

3. Pipe the csv dump into the migration script, with the appropriate database
   URL and panopticon version, like so:
`cat dump.csv | psql <dburl> -f import-v<version>.sql`

4. You can start Barad-dûr again now. On the first run, it will reaggregate the
   whole timeframe, as imported data could overlay preexisting data.
