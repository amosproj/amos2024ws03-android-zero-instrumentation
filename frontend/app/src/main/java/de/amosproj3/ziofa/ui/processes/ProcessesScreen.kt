// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.ui.processes.composables.EditButton
import de.amosproj3.ziofa.ui.processes.composables.IconAndName
import de.amosproj3.ziofa.ui.processes.composables.ProcessesHeader
import de.amosproj3.ziofa.ui.processes.composables.ProcessesSearchBar
import de.amosproj3.ziofa.ui.processes.data.ProcessesListState
import org.koin.androidx.compose.koinViewModel

@Composable
fun ProcessesScreen(
    modifier: Modifier = Modifier,
    viewModel: ProcessesViewModel = koinViewModel(),
    onClickEdit: (RunningComponent) -> Unit,
) {
    Box(modifier = modifier.fillMaxSize()) {
        Column {
            val processesListState by remember { viewModel.processListState }.collectAsState()
            var searchQuery by remember { mutableStateOf("") }

            ProcessesSearchBar(
                value = searchQuery,
                onValueChanged = { searchQuery = it },
                onStartSearch = { viewModel.startSearch(query = searchQuery) },
            )
            ProcessesHeader()
            when (val state = processesListState) {
                is ProcessesListState.Loading ->
                    Box(modifier.fillMaxSize()) {
                        CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
                    }

                is ProcessesListState.NoResults ->
                    Box(Modifier.fillMaxSize()) {
                        Text(
                            text = "No processes found.",
                            modifier = Modifier.align(Alignment.Center),
                        )
                    }

                is ProcessesListState.Valid ->
                    LazyColumn(modifier = Modifier.padding(horizontal = 20.dp).fillMaxSize()) {
                        items(state.list) { option ->
                            ProcessListRow(option = option, onClickEdit = onClickEdit)
                        }
                    }
            }
        }
    }
}

@Composable
fun ProcessListRow(
    modifier: Modifier = Modifier,
    option: RunningComponent,
    onClickEdit: (RunningComponent) -> Unit = {},
) {
    Row(
        modifier = modifier.fillMaxSize().padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        when (option) {
            is RunningComponent.StandaloneProcess -> {
                IconAndName(option, modifier = Modifier.weight(1f))
                Text(text = option.process.pid.toString(), modifier = Modifier.weight(1f))
                Text(text = option.process.ppid.toString(), modifier = Modifier.weight(1f))
            }

            is RunningComponent.Application -> {
                IconAndName(option, modifier = Modifier.weight(1f))
                Text(
                    text = option.processList.map { it.pid }.joinToString(","),
                    modifier = Modifier.weight(1f),
                )
                Text(
                    text = option.processList.map { it.ppid }.toSet().joinToString(","),
                    modifier = Modifier.weight(1f),
                )
            }
        }
        EditButton(Modifier.weight(1f), onClick = { onClickEdit(option) })
    }
    HorizontalDivider(Modifier.height(5.dp))
}
