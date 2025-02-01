// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2025 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.platform.configuration

import de.amosproj3.ziofa.api.configuration.GetSymbolsRequestState
import de.amosproj3.ziofa.api.configuration.IndexingRequestState
import de.amosproj3.ziofa.api.configuration.SymbolsAccess
import de.amosproj3.ziofa.client.ClientFactory
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

class UProbeManager(private val clientFactory: ClientFactory) : SymbolsAccess {

    /** We should do this on the backend in the future. */
    @OptIn(ExperimentalCoroutinesApi::class)
    override fun searchSymbols(
        pids: List<UInt>,
        searchQuery: String,
    ): Flow<GetSymbolsRequestState> = flow {}

    /**
     * Current problem: Symbols in the backend will be duplicated if they are already indexed and we
     * call this again. We don't know whether indexing was already done.
     */
    override fun indexSymbols(): Flow<IndexingRequestState> {
        TODO()
    }
}
