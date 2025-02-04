// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.Flow

interface SymbolsAccess {

    /**
     * Search all symbols of the given [pids] for a string that **contains** the search query. The
     * search is case-insensitive. Implementation should return a flow that completes once the
     * search is finished.
     *
     * @param pids the PID whose binaries should be searched
     * @param searchQuery the string to search for using
     * @return a flow that describes the state of the request
     */
    fun searchSymbols(pids: List<UInt>, searchQuery: String): Flow<GetSymbolsRequestState>

    /**
     * Start indexing all symbols of the system into the backend database This needs to be done once
     * before searching for symbols. Implementations should only index the backend symbols once to
     * avoid indexing the same symbols multiple times.
     *
     * @return a flow that describes the status of the indexing request
     */
    fun indexSymbols(): Flow<IndexingRequestState>
}
