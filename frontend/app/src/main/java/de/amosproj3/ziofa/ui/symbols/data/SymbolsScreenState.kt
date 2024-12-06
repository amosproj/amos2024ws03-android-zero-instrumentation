package de.amosproj3.ziofa.ui.symbols.data

data class SymbolsEntry(val name: String, val odexFile: String, val offset: ULong)
sealed class SymbolsScreenState {
    data object SymbolsLoading : SymbolsScreenState()
    data object WaitingForSearch : SymbolsScreenState()
    data class SearchResultReady(val symbols: Map<SymbolsEntry, Boolean>) : SymbolsScreenState()
    data class Error(val errorMessage: String) : SymbolsScreenState()
}
