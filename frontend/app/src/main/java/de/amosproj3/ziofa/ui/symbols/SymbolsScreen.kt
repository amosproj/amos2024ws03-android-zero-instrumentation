// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.symbols

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import de.amosproj3.ziofa.ui.configuration.composables.ErrorScreen
import de.amosproj3.ziofa.ui.configuration.composables.SubmitFab
import de.amosproj3.ziofa.ui.symbols.composables.SearchResultList
import de.amosproj3.ziofa.ui.symbols.composables.SymbolsSearchBar
import de.amosproj3.ziofa.ui.symbols.data.SymbolsScreenState
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
fun SymbolsScreen(pids: List<UInt>, modifier: Modifier = Modifier, onSymbolsSubmitted: () -> Unit) {
    val viewModel: SymbolsViewModel = koinViewModel(parameters = { parametersOf(pids) })

    var searchQuery by remember { mutableStateOf("") }
    val screenState = viewModel.screenState.collectAsState(SymbolsScreenState.WaitingForSearch)

    Box(modifier = modifier) {
        Column(modifier = Modifier.fillMaxWidth()) {
            SymbolsSearchBar(
                value = searchQuery,
                onValueChanged = { searchQuery = it },
                onStartSearch = { viewModel.startSearch(it) },
            )
            HorizontalDivider(thickness = 5.dp)
            Box(Modifier.fillMaxSize()) {
                when (val state = screenState.value) {
                    is SymbolsScreenState.SymbolsLoading ->
                        CircularProgressIndicator(Modifier.align(Alignment.Center))

                    is SymbolsScreenState.SearchResultReady ->
                        SearchResultList(
                            state.symbols,
                            onOptionChanged = { symbol, active ->
                                viewModel.symbolEntryChanged(symbol, active)
                            },
                        )

                    is SymbolsScreenState.WaitingForSearch -> Spacer(Modifier.fillMaxSize())
                    is SymbolsScreenState.Error -> ErrorScreen(error = state.errorMessage)
                }
            }
        }
        SubmitFab(
            modifier = Modifier.align(Alignment.BottomEnd),
            onClick = {
                viewModel.submit()
                onSymbolsSubmitted()
            },
        )
    }
}
