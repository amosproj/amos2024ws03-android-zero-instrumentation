// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Devices
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import de.amosproj3.ziofa.ui.configuration.composables.EbpfOptions
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.configuration.composables.SubmitFab
import de.amosproj3.ziofa.ui.configuration.data.ConfigurationScreenState
import org.koin.androidx.compose.koinViewModel

/** Screen for configuring eBPF programs */
@Preview(device = Devices.AUTOMOTIVE_1024p)
@Composable
fun ConfigurationScreen(
    modifier: Modifier = Modifier,
    viewModel: ConfigurationViewModel = koinViewModel(),
    onBack: () -> Unit = {}
) {
    Box(modifier = modifier.fillMaxSize()) {
        val screenState by remember { viewModel.configurationScreenState }.collectAsState()
        val configurationChangedByUser by remember { viewModel.changed }.collectAsState()
        when (val state = screenState) { // needed for immutability
            is ConfigurationScreenState.LIST -> {

                // Render list of options
                EbpfOptions(options = state.options, onCheckedChanged = { option, newValue ->
                    viewModel.optionChanged(option, newValue)
                })

                // Show the submit button if the user changed settings
                if (configurationChangedByUser) {
                    SubmitFab(
                        modifier = Modifier.align(
                            Alignment.BottomEnd,
                        ), onClick = {
                            viewModel.configurationSubmitted()
                        })
                }
            }

            is ConfigurationScreenState.ERROR -> {
                ErrorScreen(state.errorMessage, onBack)
            }

            is ConfigurationScreenState.LOADING -> {
                // Display loading anim while state is unknown
                CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
            }
        }
    }
}


