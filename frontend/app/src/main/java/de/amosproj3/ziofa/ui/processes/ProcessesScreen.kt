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
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.processes.composables.EditButton
import de.amosproj3.ziofa.ui.processes.composables.IconAndName
import de.amosproj3.ziofa.ui.processes.composables.ProcessesHeader
import org.koin.androidx.compose.koinViewModel

@Composable
fun ProcessesScreen(
    modifier: Modifier,
    viewModel: ProcessesViewModel = koinViewModel(),
    onClickEdit: (ProcessListEntry) -> Unit,
) {
    Box(modifier = modifier.fillMaxSize()) {
        Column {
            val options by remember { viewModel.processesList }.collectAsState()

            ProcessesHeader()
            LazyColumn(modifier = Modifier.padding(horizontal = 20.dp).fillMaxSize()) {
                items(options) { option -> ProcessListRow(option, onClickEdit = onClickEdit) }
            }
        }
    }
}

@Composable
fun ProcessListRow(
    option: ProcessListEntry,
    onClickProcessInfo: (ProcessListEntry) -> Unit =
        {}, // TODO implement modal with info about processes
    onClickEdit: (ProcessListEntry) -> Unit = {},
) {
    Row(
        modifier = Modifier.fillMaxSize().padding(vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        when (option) {
            is ProcessListEntry.ProcessEntry -> {
                IconAndName(option, modifier = Modifier.weight(1f))
                Text(text = option.process.pid.toString(), modifier = Modifier.weight(1f))
                Text(text = option.process.ppid.toString(), modifier = Modifier.weight(1f))
            }

            is ProcessListEntry.ApplicationEntry -> {
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