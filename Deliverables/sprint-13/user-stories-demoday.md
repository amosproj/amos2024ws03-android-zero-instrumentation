<!--
SPDX-FileCopyrightText: 2025 Tom Weisshuhn <tom.weisshuhn@fau.de>
SPDX-FileCopyrightText: 2025 Franz Schlicht <franz.schlicht@gmail.com>

SPDX-License-Identifier: MIT
-->

# user stories for demo day

Before starting with the demo: Note that our "users" are software developers for a niche topic. 
We tried our best to make it as understandable as possible for you, but keep that in mind when you don't understand all information displayed in the UI.

I am a software developer at eSolutions and my task is to integrate the software we get from our suppliers into the infotainment system. 

Please note that the following user stories are extremely simplified and fictive. They may not reflect the real implementations of eSolutions Infotainment Software. 

## 1. rear view camera is laggy
One of the many software suppliers has delivered its new rear-view camera system. Since its integration the UI's response time worsened and the camera pictures are laggy.
I want to find out why this is the case and suspect the traffic between the infotainment system and the camera or excessive memory usage. To pin down the problem further I take the following steps:

1. start ZIOFA, go to the configuration and select the process for the camera. (I get the PID from the task_name)
2. activate `UNIX Domain Socket Analysis` (for traffic) and `Garbage Collector Analysis` (for memory usage)
3. go back to the landing screen and open "Visualize". Select the metrics for Garbage Collection and select the chart (nothing unusual)
4. go on with visualizing the socket traffic as a chart (you see rapidly increasing duration for receiving packets over time)

This indicates that the UI is constantly waiting while receiving traffic. This has to be addressed.

Now - after distilling the root of the problem - I can tackle it directly together with the software supplier.

before the next use case, make sure to reset the hooks

# 2. navigation app is slow
Since the last release the navigation app is significantly slower. The only changes made were related to the backups. Thus my first starting point is, that the slow down is due to opening/ writing to files.
1. go to "configuration", select the navigation app
2. activate `VFS Write Analysis` (detecting excessive file-writing) and `Open File Deskriptors` (to track open files)
3. visualize the metrics for file-writing as a chart (nothing unusual)
4. visualize as a chart, too (recognise a constant increase of open files -> apparently they don't get closed)

With this information, I can address the slower navigation app better.
