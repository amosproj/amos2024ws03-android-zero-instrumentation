// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.symbols.data

sealed class SymbolsSelectionState {

    data class Ready(val availableOptions: List<String>, val selectedOption: String?)

    data class IncompleteSelectionPrompt(val message: String)

    data class Error(val errorMessage: String)

    data object Loading
}
