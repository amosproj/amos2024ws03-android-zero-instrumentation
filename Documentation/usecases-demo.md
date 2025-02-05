<!--
SPDX-FileCopyrightText: 2025 Tom Weisshuhn <tom.weisshuhn@fau.de>
SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>
SPDX-FileCopyrightText: 2025 Felix Hilgers <felix.hilgers@fau.de>

SPDX-License-Identifier: MIT
-->

# Use Cases

- Users of this product are other software developers, more specifically software engineers in the automotive industry.
- Keywords: Observability, Profiling, Debugging, Tracing
- Focus is to be "always on" and collect data in the background that will help diagnosing problems that might happen later on
- Product is split into two components: Event Visualization and Event Collection

> [!NOTE]
> The event collection for part two can be started while talking about part 1

## Visualization 

> [!NOTE]
> The data used for this demonstration is fake. 
> This is due to the fact that demonstrating the visualization with seemingly random data is not insightful.
> The real data collection will be shown in part 2.

### Usecase: navigation app is slow

- This shows the data inspection part of the workflow
- In the real world, the data would have already been collected
- The data will then be used, together with other sources, to diagnose the problem

Since the last release the navigation app is significantly slower. 
The only changes made were related to the backups. 
Thus my first starting point is, that the slow down is due to opening and writing to files.

1. go to "configuration", select the navigation app
2. activate `VFS Write Analysis` (detecting excessive file-writing) and `Open File Deskriptors` (to track open files)
3. visualize the metrics for file-writing as a chart (nothing unusual)
4. visualize as a chart, too (recognise a constant increase of open files -> apparently they don't get closed)

With this information, I can address the slower navigation app better.

## Collection

- Using GRPC and Protobuf, separate `proto` files make developing integrations very easy

### Usecase: Collect Events in the Background and Query the data later

1. Start the daemon: `cargo xtask daemon --android --release`
2. Start the client to collect events: `cargo xtask client --android --release -- collect /data/local/tmp/db.sqlite`
3. Use the emulator so that events are generated
4. Pull the data from the emulator: `adb pull /data/local/tmp/db.sqlite`
5. Use any sqlite browser, for example [SQLime](https://sqlime.org/) or [SQLiteBrowser](https://sqlitebrowser.org/) to open the database
6. Look at the data and use the provided queries

- While still collecting, show that the app has not much overhead (use `top`)
- One could correlate the events with a real event that happened (e.g. App Crashed, cross reference timestamps)
- Query what apps are writing a lot of data
- Query which paths are written to a lot

#### Garbage Collection

- Offers an overview over all garbage collections (don't mind the nanosecond conversions^^)

```sql
-- Garbage collection statistics (inspired by LogGC but this catches all garbage collections)

SELECT process, timestamp, printf('%.2fMB', freed) AS freed,
                           printf('%d%%', FREE) AS FREE,
                           printf('%dMB/%dMB', allocated, target) AS used,
                           printf('%.3fms', duration) AS duration
FROM
  (SELECT cmdline AS process,
          datetime(timestamp / 1000000000, 'unixepoch', 'localtime') AS timestamp,
          ROUND(gce.freed_bytes / 1000000.0, 2) AS freed,
          CAST(ROUND(100 - (gce.num_bytes_allocated * 100.0 / gce.target_footprint)) AS INTEGER) AS FREE,
          gce.num_bytes_allocated / 1000000 AS allocated,
          gce.target_footprint / 1000000 AS target,
          ROUND(gce.duration_ns / 1000000.0, 3) AS duration
   FROM garbage_collect_events gce
   JOIN EVENTS e ON gce.event = e.id) AS garbage_collect;
```

#### Write Queries

```sql
-- Show all paths that were written to

SELECT file_path AS path
FROM write_events we
JOIN EVENTS e ON we.event = e.id
GROUP BY file_path;
```

```sql
-- Bucket writes into 5 seconds segments and show count and sum of written bytes

SELECT cmdline,
       datetime((timestamp / 5000000000 * 5), 'unixepoch', 'localtime') AS time_bucket,
       file_path,
       COUNT(*) AS write_count,
       printf('%.2fKB', SUM(bytes_written) / 1000.0) AS bytes_written
FROM write_events we
JOIN EVENTS e ON we.event = e.id
GROUP BY file_path,
         time_bucket;
```

```sql
-- Show the files that was written most to

SELECT cmdline,
       file_path AS PATH,
       COUNT(*) AS write_count,
       printf('%.2fKB', SUM(bytes_written) / 1000.0) AS written
FROM write_events we
JOIN EVENTS e ON we.event = e.id
GROUP BY file_path
ORDER BY SUM(bytes_written) DESC;
```

#### Conclusion

- Reiterate: High Level of modularity, writing such this tool on top of the existing work is easy (the sqlite collection took 1 evening)
- Target are developers: Developers want to tailor the tool to their needs (this project is just a building block for e.solutions)