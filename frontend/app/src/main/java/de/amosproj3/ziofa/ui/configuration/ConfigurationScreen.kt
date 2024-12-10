// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.composables.EbpfIOFeatureOptions
import de.amosproj3.ziofa.ui.configuration.composables.EbpfUprobeFeatureOptions
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.configuration.composables.SubmitFab
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.configuration.data.ConfigurationScreenState
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

/** Screen for configuring eBPF programs */
@Preview(device = Devices.AUTOMOTIVE_1024p)
@Composable
fun ConfigurationScreen(
    modifier: Modifier = Modifier,
    onBack: () -> Unit = {},
    onAddUprobeSelected: () -> Unit = {},
    pids: IntArray? = null,
) {

    val viewModel: ConfigurationViewModel =
        koinViewModel(
            parameters = {
                parametersOf(pids?.let { it.map { int -> int.toUInt() } } ?: listOf<UInt>())
            }
        )

    Box(modifier = modifier.padding(horizontal = 20.dp, vertical = 20.dp).fillMaxSize()) {
        val screenState by remember { viewModel.configurationScreenState }.collectAsState()
        val configurationChangedByUser by remember { viewModel.changed }.collectAsState()
        when (val state = screenState) { // needed for immutability
            is ConfigurationScreenState.Valid -> {

                Column(Modifier.fillMaxWidth()) {
                    // Render list of options
                    SectionTitleRow("IO Observability Features")
                    EbpfIOFeatureOptions(
                        options =
                            state.options.filter { it !is BackendFeatureOptions.UprobeOption },
                        onOptionChanged = { option, newState ->
                            viewModel.optionChanged(option, newState)
                        },
                    )

                    SectionTitleRow("Uprobes")
                    EbpfUprobeFeatureOptions(
                        options =
                            state.options.mapNotNull {
                                if (it is BackendFeatureOptions.UprobeOption) it else null
                            },
                        onOptionDeleted = { option ->
                            viewModel.optionChanged(option, active = false)
                        },
                        onAddUprobeSelected = onAddUprobeSelected,
                    )
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

@Composable
fun SectionTitleRow(title: String) {
    Row(horizontalArrangement = Arrangement.Center, modifier = Modifier.padding(bottom = 10.dp)) {
        Text(title, fontWeight = FontWeight.Bold)
    }
    HorizontalDivider(thickness = 5.dp)
}
