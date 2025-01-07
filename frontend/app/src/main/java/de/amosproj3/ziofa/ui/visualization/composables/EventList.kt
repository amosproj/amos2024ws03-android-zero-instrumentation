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
import androidx.compose.ui.text.intl.Locale
import de.amosproj3.ziofa.api.events.BackendEvent

@Composable
fun EventList(events: List<BackendEvent>, modifier: Modifier = Modifier) {
    val locale = Locale.current.platformLocale

    events.getOrNull(0)?.let { Header(it) }
    LazyColumn(modifier.fillMaxSize()) {
        items(events) { event ->
            Row {
                Text(text = event.processId.toString(), modifier = Modifier.weight(1f))
                Text(text = event.fileDescriptor.toString(), modifier = Modifier.weight(1f))
                Text(
                    text =
                        String.format(
                            locale,
                            "%.2f",
                            event.startTimestamp.toDouble() / 1_000_000_000,
                        ),
                    modifier = Modifier.weight(1f),
                )
                when (event) {
                    is BackendEvent.SendMessageEvent -> {
                        Text(
                            text = (event.durationNanos / 1_000_000u).toString(),
                            modifier = Modifier.weight(1f),
                        )
                    }
                    is BackendEvent.VfsWriteEvent -> {
                        Text(text = event.size.toString(), modifier = Modifier.weight(1f))
                    }
                }
            }
        }
    }
}

@Composable
fun Header(firstEvent: BackendEvent, modifier: Modifier = Modifier) {
    Row(modifier) {
        Text(text = "Process ID", modifier = Modifier.weight(1f))
        Text(text = "File Descriptor", modifier = Modifier.weight(1f))
        Text(text = "Event time since Boot in s", modifier = Modifier.weight(1f))
        when (firstEvent) {
            is BackendEvent.SendMessageEvent -> {
                Text(text = "Duration in ms", modifier = Modifier.weight(1f))
            }
            is BackendEvent.VfsWriteEvent -> {
                Text(text = "Size in byte", modifier = Modifier.weight(1f))
            }
        }
    }
}
