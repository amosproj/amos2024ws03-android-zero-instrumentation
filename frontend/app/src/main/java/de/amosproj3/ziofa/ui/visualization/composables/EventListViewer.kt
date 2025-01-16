// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.composables

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import de.amosproj3.ziofa.ui.visualization.data.EventListMetadata
import de.amosproj3.ziofa.ui.visualization.data.GraphedData
import kotlinx.coroutines.launch

@Composable
fun EventListViewer(
    eventListData: GraphedData.EventListData,
    eventListMetadata: EventListMetadata,
    autoScrollActive: Boolean,
    modifier: Modifier = Modifier,
) {
    HeaderRow(eventListMetadata)
    val lazyListState = rememberLazyListState()
    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(eventListData.eventData.size) {
        if (eventListData.eventData.isNotEmpty() && autoScrollActive) {
            coroutineScope.launch {
                lazyListState.animateScrollToItem(eventListData.eventData.lastIndex)
            }
        }
    }

    LazyColumn(state = lazyListState, modifier = modifier.fillMaxSize()) {
        items(eventListData.eventData) { event ->
            ListRow(col1 = event.col1, col2 = event.col2, col3 = event.col3, col4 = event.col4)
        }
    }
}

@Composable
fun ListRow(col1: String, col2: String, col3: String, col4: String, modifier: Modifier = Modifier) {
    Row(modifier = modifier) {
        Text(text = col1, modifier = Modifier.weight(1f))
        Text(text = col2, modifier = Modifier.weight(1f))
        Text(text = col3, modifier = Modifier.weight(1f))
        Text(text = col4, modifier = Modifier.weight(1f))
    }
}

@Composable
fun HeaderRow(eventListMetadata: EventListMetadata, modifier: Modifier = Modifier) {
    Row(modifier = modifier) {
        Text(text = eventListMetadata.label1, modifier = Modifier.weight(1f))
        Text(text = eventListMetadata.label2, modifier = Modifier.weight(1f))
        Text(text = eventListMetadata.label3, modifier = Modifier.weight(1f))
        Text(text = eventListMetadata.label4, modifier = Modifier.weight(1f))
    }
}
