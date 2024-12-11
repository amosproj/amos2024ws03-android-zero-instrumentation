// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry

sealed class GetSymbolsRequestState {
    data class Response(val symbols: List<SymbolsEntry>) : GetSymbolsRequestState()

    data class Error(val errorMessage: String) : GetSymbolsRequestState()

    data object Loading : GetSymbolsRequestState()
}
