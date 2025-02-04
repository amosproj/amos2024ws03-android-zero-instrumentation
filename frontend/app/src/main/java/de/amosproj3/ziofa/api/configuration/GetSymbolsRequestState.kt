// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry

/** To be used in implementation of [SymbolsAccess.searchSymbols] */
sealed class GetSymbolsRequestState {

    /**
     * The request has finished successfully
     *
     * @param symbols the symbols that were found
     */
    data class Response(val symbols: List<SymbolsEntry>) : GetSymbolsRequestState()

    /**
     * The request has failed
     *
     * @param errorMessage the error message
     */
    data class Error(val errorMessage: String) : GetSymbolsRequestState()

    /** The request is currently in progress */
    data object Loading : GetSymbolsRequestState()
}
