// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.composables.EbpfUprobeFeatureOptions
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.configuration.composables.PresetFeatureOptionsGroup
import de.amosproj3.ziofa.ui.configuration.composables.SubmitFab
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.configuration.data.ConfigurationScreenState
import de.amosproj3.ziofa.ui.configuration.data.FeatureType
import kotlinx.collections.immutable.ImmutableList
import kotlinx.collections.immutable.persistentListOf
import kotlinx.collections.immutable.toImmutableList
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

/** Screen for configuring eBPF programs */
@Preview(device = Devices.AUTOMOTIVE_1024p)
@Composable
fun ConfigurationScreen(
    modifier: Modifier = Modifier,
    onAddUprobeSelected: () -> Unit = {},
    pids: ImmutableList<UInt>? = persistentListOf(),
) {

    val viewModel: ConfigurationViewModel = koinViewModel(parameters = { parametersOf(pids) })

    Box(modifier = modifier.padding(horizontal = 20.dp, vertical = 20.dp).fillMaxSize()) {
        val screenState by remember { viewModel.configurationScreenState }.collectAsState()
        val configurationChangedByUser by remember { viewModel.changed }.collectAsState()
        when (val state = screenState) { // needed for immutability
            is ConfigurationScreenState.Valid -> {
                // Render list of options
                LazyColumn(Modifier.fillMaxWidth()) {
                    item {
                        PresetFeatureOptionsGroup(
                            options = state.options,
                            type = FeatureType.IO,
                            onOptionChanged = { option, newState ->
                                viewModel.optionChanged(option, newState)
                            },
                        )
                    }

                    item {
                        PresetFeatureOptionsGroup(
                            options = state.options,
                            type = FeatureType.SIGNALS,
                            onOptionChanged = { option, newState ->
                                viewModel.optionChanged(option, newState)
                            },
                        )
                    }

                    item {
                        EbpfUprobeFeatureOptions(
                            options =
                                state.options
                                    .mapNotNull { it as? BackendFeatureOptions.UprobeOption }
                                    .toImmutableList(),
                            onOptionDeleted = { option ->
                                viewModel.optionChanged(option, active = false)
                            },
                            onAddUprobeSelected = onAddUprobeSelected,
                        )
                    }
                }

                // Show the submit button if the user changed settings
                if (configurationChangedByUser) {
                    SubmitFab(
                        modifier = Modifier.align(Alignment.BottomEnd),
                        onClick = { viewModel.configurationSubmitted() },
                    )
                }
            }

            is ConfigurationScreenState.Invalid -> {
                ErrorScreen(state.errorMessage)
            }

            is ConfigurationScreenState.Loading -> {
                // Display loading anim while state is unknown
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            }
        }
    }
}
