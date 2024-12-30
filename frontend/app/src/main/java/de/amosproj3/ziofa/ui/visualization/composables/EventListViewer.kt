// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import de.amosproj3.ziofa.client.Event
import de.amosproj3.ziofa.ui.visualization.utils.nanosToSeconds

@Composable
fun EventListViewer(events: List<Event>, modifier: Modifier = Modifier) {

    events.getOrNull(0)?.let { Header(it) }
    LazyColumn(modifier.fillMaxSize()) {
        items(events) { event ->
            Row {
                when (event) {
                    is Event.SysSendmsg -> {
                        ListItem(
                            col1 = "${event.pid}",
                            col2 = "${event.fd}",
                            col3 = event.beginTimeStamp.nanosToSeconds(),
                            col4 = event.durationNanoSecs.nanosToSeconds(),
                            modifier = Modifier.weight(1f)
                        )
                    }

                    is Event.VfsWrite -> {
                        ListItem(
                            col1 = "${event.fp}",
                            col2 = "${event.pid}",
                            col3 = event.beginTimeStamp.nanosToSeconds(),
                            col4 = "${event.bytesWritten}",
                            modifier = Modifier.weight(1f)
                        )
                    }

                    is Event.JniReferences ->
                        ListItem(
                            col1 = "${event.pid}",
                            col2 = "${event.tid}",
                            col3 = "${event.beginTimeStamp}",
                            col4 = event.jniMethodName!!.name, //TODO why is this nullable
                            modifier = Modifier.weight(1f)
                        )
                }
            }
        }
    }
}

@Composable
fun ListItem(
    col1: String,
    col2: String,
    col3: String,
    col4: String,
    modifier: Modifier = Modifier
) {
    Text(text = col1, modifier = modifier)
    Text(text = col2, modifier = modifier)
    Text(text = col3, modifier = modifier)
    Text(text = col4, modifier = modifier)
}

@Composable
fun Header(firstEvent: Event, modifier: Modifier = Modifier) {
    Row(modifier) {
        Text(text = "Process ID", modifier = Modifier.weight(1f))
        Text(text = "File Descriptor", modifier = Modifier.weight(1f))
        Text(text = "Event time since Boot in s", modifier = Modifier.weight(1f))
        when (firstEvent) {
            is Event.SysSendmsg -> {
                Text(text = "Duration in ms", modifier = Modifier.weight(1f))
            }

            is Event.VfsWrite -> {
                Text(text = "Size in byte", modifier = Modifier.weight(1f))
            }

            is Event.JniReferences -> TODO()
        }
    }
}
