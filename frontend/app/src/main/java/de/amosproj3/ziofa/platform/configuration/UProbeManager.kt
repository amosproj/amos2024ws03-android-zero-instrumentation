// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.platform.configuration

import de.amosproj3.ziofa.api.configuration.GetSymbolsRequestState
import de.amosproj3.ziofa.api.configuration.SymbolsAccess
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flatMapMerge
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.toList
import timber.log.Timber

class UProbeManager(private val clientFactory: ClientFactory) : SymbolsAccess {

    /** We should do this on the backend in the future. */
    @OptIn(ExperimentalCoroutinesApi::class)
    override fun searchSymbols(
        pids: List<UInt>,
        searchQuery: String,
    ): Flow<GetSymbolsRequestState> =
        flow {
            emit(GetSymbolsRequestState.Loading)
            try {
                val client = clientFactory.connect()
                val symbols =
                    pids
                        .map { pid ->
                            client
                                .getOdexFiles(pid)
                                .onEach { Timber.i("Requesting symbols for odex file $it") }
                                .flatMapMerge { odexFile ->
                                    client
                                        .getSymbols(filePath = odexFile)
                                        .filter {
                                            it.method
                                                .lowercase()
                                                .contains(searchQuery.lowercase())
                                        }
                                        .map { symbol ->
                                            SymbolsEntry(symbol.method, odexFile, symbol.offset)
                                        }
                                }
                        }
                        .merge()
                        .toList()
                emit(GetSymbolsRequestState.Response(symbols))
            } catch (e: Exception) {
                emit(GetSymbolsRequestState.Error(e.stackTraceToString()))
            }
        }
            .onStart { Timber.i("searchSymbols pids=$pids searchQuery=$searchQuery") }
            .flowOn(Dispatchers.IO)
}
