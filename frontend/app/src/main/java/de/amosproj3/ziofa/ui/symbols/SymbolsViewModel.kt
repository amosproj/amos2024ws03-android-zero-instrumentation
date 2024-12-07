package de.amosproj3.ziofa.ui.symbols

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.GetOdexFilesRequestState
import de.amosproj3.ziofa.api.configuration.GetSymbolsRequestState
import de.amosproj3.ziofa.api.configuration.LocalConfigurationAccess
import de.amosproj3.ziofa.api.configuration.SymbolsAccess
import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry
import de.amosproj3.ziofa.ui.symbols.data.SymbolsScreenState
import de.amosproj3.ziofa.ui.symbols.data.SymbolsSelectionState
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class SymbolsViewModel(
    val symbolsAccess: SymbolsAccess,
    val localConfigurationAccess: LocalConfigurationAccess,
    val pid: UInt,
) : ViewModel() {

    private val PLEASE_SELECT_ODEX = SymbolsSelectionState.IncompleteSelectionPrompt(
        message = "Please select a odex file!"
    )

    private val selectedOdexFile = MutableStateFlow<String?>(null)

    //TODO we need the offset and method name here -> data class
    private val selectedSymbol = MutableStateFlow<String?>(null)

    val odexSelectionState =
        symbolsAccess.getOdexFilesForPid(pid)
            .combine(selectedOdexFile) { odexFileRequest, selectedOdexFile ->
                when (odexFileRequest) {
                    is GetOdexFilesRequestState.Loading -> SymbolsSelectionState.Loading
                    is GetOdexFilesRequestState.Error -> SymbolsSelectionState.Error(odexFileRequest.errorMessage)
                    is GetOdexFilesRequestState.Response -> SymbolsSelectionState.Ready(
                        odexFileRequest.odexFiles,
                        selectedOdexFile
                    )
                }
            }.stateIn(
                viewModelScope,
                SharingStarted.Lazily,
                initialValue = SymbolsSelectionState.Loading
            )

    @OptIn(ExperimentalCoroutinesApi::class)
    val symbolSelectionState =
        selectedOdexFile.flatMapLatest { selectedOdexFile ->
            if (selectedOdexFile == null) {
                return@flatMapLatest flowOf()
            }
            symbolsAccess.getSymbolsForFile(selectedOdexFile)
        }.combine(selectedSymbol) { symbolFileRequest, selectedSymbol ->
            when (symbolFileRequest) {
                is GetSymbolsRequestState.Loading -> SymbolsSelectionState.Loading
                is GetSymbolsRequestState.Error -> SymbolsSelectionState.Error(symbolFileRequest.errorMessage)
                is GetSymbolsRequestState.Response -> SymbolsSelectionState.Ready(
                    symbolFileRequest.symbols,
                    selectedSymbol
                )
            }
        }.stateIn(viewModelScope, SharingStarted.Lazily, PLEASE_SELECT_ODEX)


    fun odexSelected(odexFile: String) {
        selectedOdexFile.value = odexFile
    }

    fun symbolSelected(symbol: String) {
        selectedSymbol.value = symbol
    }

    fun submit(odexFile: String, symbol: String) {
        //TODO submit along with pid, offset, odex, symbol to local configuration
    }

    val searchQuery = MutableStateFlow<String?>(null)
    val searchResult = flowOf<SymbolsScreenState>()

    fun symbolEntryChanged(symbolsEntry: SymbolsEntry, newState:Boolean){
        //TODo
    }

}