package de.amosproj3.ziofa.ui.symbols.data

data class SymbolsEntry(val name: String, val offset: ULong, val active: Boolean)
sealed class SymbolsScreenState {
    data object SymbolsLoading : SymbolsScreenState()
    data object WaitingForSearch: SymbolsScreenState()
    data class SearchResultReady(val symbols: List<SymbolsEntry>) : SymbolsScreenState()
    data class Error(val errorMessage: String) : SymbolsScreenState()
}
