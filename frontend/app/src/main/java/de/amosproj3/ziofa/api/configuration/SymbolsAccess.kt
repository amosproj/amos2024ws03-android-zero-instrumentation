// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.Flow

interface SymbolsAccess {

    /**
     * Search all symbols of the given [pids] for a string that **contains** the search query. The
     * search is case-insensitive.
     *
     * @param pids the PID whose binaries should be searched
     * @param searchQuery the string to search for using
     * @return a flow that describes the state of the request
     */
    fun searchSymbols(pids: List<UInt>, searchQuery: String): Flow<GetSymbolsRequestState>
}
