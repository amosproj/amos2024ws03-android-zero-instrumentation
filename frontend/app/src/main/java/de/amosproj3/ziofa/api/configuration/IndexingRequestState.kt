// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

/** To be used in implementation of [SymbolsAccess.indexSymbols] */
sealed class IndexingRequestState {
    data object NotStarted : IndexingRequestState()

    data object Started : IndexingRequestState()

    data object Done : IndexingRequestState()

    data class Error(val error: Throwable) : IndexingRequestState()
}
