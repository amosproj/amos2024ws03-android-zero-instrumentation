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
import de.amosproj3.ziofa.ui.visualization.data.EventListMetadata
import de.amosproj3.ziofa.ui.visualization.data.GraphedData

@Composable
fun EventListViewer(
    eventListData: GraphedData.EventListData,
    eventListMetadata: EventListMetadata,
    modifier: Modifier = Modifier,
) {
    Header(eventListMetadata)
    LazyColumn(modifier.fillMaxSize()) {
        items(eventListData.eventData) { event ->
            Row {
                ListItem(
                    col1 = event.col1,
                    col2 = event.col2,
                    col3 = event.col3,
                    col4 = event.col4,
                    modifier = Modifier.weight(1f),
                )
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
    modifier: Modifier = Modifier,
) {
    Text(text = col1, modifier = modifier)
    Text(text = col2, modifier = modifier)
    Text(text = col3, modifier = modifier)
    Text(text = col4, modifier = modifier)
}

@Composable
fun Header(eventListMetadata: EventListMetadata, modifier: Modifier = Modifier) {
    Row(modifier) {
        Text(text = eventListMetadata.label1, modifier = modifier.weight(1f))
        Text(text = eventListMetadata.label2, modifier = modifier.weight(1f))
        Text(text = eventListMetadata.label3, modifier = modifier.weight(1f))
        Text(text = eventListMetadata.label4, modifier = modifier.weight(1f))
    }
}
