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
import de.amosproj3.ziofa.api.WriteEvent
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

@Composable
fun EventList(events: List<WriteEvent>) {
    val locale = Locale.current.platformLocale

    events.getOrNull(0)?.let { Header(it) }
    LazyColumn(Modifier.fillMaxSize()) {
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
                    is WriteEvent.SendMessageEvent -> {
                        Text(
                            text = (event.durationMicros / 1_000u).toString(),
                            modifier = Modifier.weight(1f),
                        )
                    }
                    is WriteEvent.VfsWriteEvent -> {
                        Text(text = event.size.toString(), modifier = Modifier.weight(1f))
                    }
                }
            }
        }
    }
}

@Composable
fun Header(firstEvent: WriteEvent) {
    Row {
        Text(text = "Process ID", modifier = Modifier.weight(1f))
        Text(text = "File Descriptor", modifier = Modifier.weight(1f))
        Text(text = "Event time since Boot in s", modifier = Modifier.weight(1f))
        when (firstEvent) {
            is WriteEvent.SendMessageEvent -> {
                Text(text = "Duration in ms", modifier = Modifier.weight(1f))
            }
            is WriteEvent.VfsWriteEvent -> {
                Text(text = "Size in byte", modifier = Modifier.weight(1f))
            }
        }
    }
}

fun ULong.toFormattedTime(): String? {
    return Instant.ofEpochMilli((this / 1_000_000u).toLong())
        .atZone(ZoneId.systemDefault())
        .toLocalDateTime()
        .format(DateTimeFormatter.ISO_LOCAL_DATE_TIME)
}
