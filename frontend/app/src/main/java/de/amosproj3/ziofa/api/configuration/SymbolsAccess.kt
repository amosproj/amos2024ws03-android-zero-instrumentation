package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.Flow

interface SymbolsAccess {
    fun searchSymbols(pids: List<UInt>, searchQuery: String): Flow<GetSymbolsRequestState>
}
