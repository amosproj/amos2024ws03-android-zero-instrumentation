// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import org.koin.androidx.compose.koinViewModel

/** Screen for configuring eBPF programs */
@Preview(device = Devices.AUTOMOTIVE_1024p)
@Composable
fun ConfigurationScreen(
    modifier: Modifier = Modifier,
    viewModel: ConfigurationViewModel = koinViewModel(),
) {
    Box(modifier = modifier.fillMaxSize()) {
        val options = viewModel.programList.collectAsState().value
        LazyColumn(modifier = Modifier.padding(horizontal = 20.dp).fillMaxSize()) {
            item { Spacer(Modifier.height(15.dp)) }
            items(options) { option ->
                Row(
                    modifier = Modifier.fillMaxSize(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(option.name)
                    Checkbox(
                        checked = option.active,
                        onCheckedChange = { viewModel.optionChanged(option, it) },
                    )
                }
            }
        }
        ExtendedFloatingActionButton(
            modifier = Modifier.align(Alignment.BottomEnd).padding(end = 25.dp, bottom = 25.dp),
            onClick = { viewModel.configurationSubmitted() },
            icon = { Icon(imageVector = Icons.AutoMirrored.Default.Send, contentDescription = "") },
            text = { Text("Submit") },
        )
    }
}
