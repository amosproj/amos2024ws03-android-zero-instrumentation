// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.symbols

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationAction
import de.amosproj3.ziofa.api.configuration.GetSymbolsRequestState
import de.amosproj3.ziofa.api.configuration.SymbolsAccess
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry
import de.amosproj3.ziofa.ui.symbols.data.SymbolsScreenState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import timber.log.Timber

class SymbolsViewModel(
    private val symbolsAccess: SymbolsAccess,
    private val configurationAccess: ConfigurationAccess,
    val pids: List<UInt>,
) : ViewModel() {

    val screenState = MutableStateFlow<SymbolsScreenState>(SymbolsScreenState.WaitingForSearch)

    fun submit() {
        viewModelScope.launch {
            val currentState = screenState.value
            if (currentState is SymbolsScreenState.SearchResultReady) {
                val selectedSymbols =
                    currentState.symbols.entries.filter { it.value }.map { it.key }
                pids.forEach { pid ->
                    selectedSymbols.forEach {
                        configurationAccess.performAction(
                            // TODO how to we make sure, if there are multiple pids, that the we
                            // only
                            // set uprobes for the pids where each symbol is coming from??

                            // TODO replace SymbolsEntry with BackendFeatureOption for consistency
                            ConfigurationAction.ChangeFeature(
                                backendFeature =
                                    BackendFeatureOptions.UprobeOption(
                                        method = it.name,
                                        enabled = true,
                                        pids = pids.toSet(),
                                        offset = it.offset,
                                        odexFilePath = it.odexFile,
                                    ),
                                enable = true,
                                pids = pids.toSet(),
                            )
                        )
                    }
                }
            }
        }
    }

    fun symbolEntryChanged(symbolsEntry: SymbolsEntry, newState: Boolean) {
        screenState.update { prev ->
            if (prev is SymbolsScreenState.SearchResultReady) {
                prev.copy(
                    symbols =
                        prev.symbols.updateEntry(symbolsEntry = symbolsEntry, newState = newState)
                )
            } else {
                prev
            }
        }
    }

    fun startSearch(searchQuery: String) {
        viewModelScope.launch {
            symbolsAccess
                .searchSymbols(pids, searchQuery)
                .onStart { Timber.i("starting search") }
                .onEach { Timber.i("Search State: $it") }
                .onCompletion { Timber.i("search completed") }
                .collect { screenState.value = it.toUIState() }
        }
    }

    private fun SymbolsEntry.toUprobeConfigForPid(pid: UInt) =
        UprobeConfig(fnName = this.name, offset = this.offset, target = this.odexFile, pid = pid)

    private fun GetSymbolsRequestState.toUIState(): SymbolsScreenState {
        return when (this) {
            is GetSymbolsRequestState.Loading -> SymbolsScreenState.SymbolsLoading
            is GetSymbolsRequestState.Error -> SymbolsScreenState.Error(this.errorMessage)

            is GetSymbolsRequestState.Response ->
                SymbolsScreenState.SearchResultReady(symbols = this.symbols.associateWith { false })
        }
    }

    private fun Map<SymbolsEntry, Boolean>.updateEntry(
        symbolsEntry: SymbolsEntry,
        newState: Boolean,
    ) = this.minus(symbolsEntry).plus(symbolsEntry to newState)
}
