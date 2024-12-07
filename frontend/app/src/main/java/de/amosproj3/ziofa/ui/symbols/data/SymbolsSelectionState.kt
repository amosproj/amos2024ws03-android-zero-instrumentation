package de.amosproj3.ziofa.ui.symbols.data

sealed class SymbolsSelectionState {

    data class Ready(val availableOptions: List<String>, val selectedOption: String?)

    data class IncompleteSelectionPrompt(val message: String)

    data class Error(val errorMessage: String)

    data object Loading
}
