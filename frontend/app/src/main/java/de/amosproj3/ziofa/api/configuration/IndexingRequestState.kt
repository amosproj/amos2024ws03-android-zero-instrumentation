// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

/** To be used in implementation of [SymbolsAccess.indexSymbols] */
sealed class IndexingRequestState {

    /** The indexing request is not started yet */
    data object NotStarted : IndexingRequestState()

    /** The indexing request is currently in progress */
    data object Started : IndexingRequestState()

    /** The indexing request has finished successfully */
    data object Done : IndexingRequestState()

    /** There was an error while indexing */
    data class Error(val error: Throwable) : IndexingRequestState()
}
