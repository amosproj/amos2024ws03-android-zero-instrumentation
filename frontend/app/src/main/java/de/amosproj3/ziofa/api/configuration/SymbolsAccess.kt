package de.amosproj3.ziofa.api.configuration

import android.provider.Contacts.Intents.UI
import kotlinx.coroutines.flow.Flow

interface SymbolsAccess {
    fun searchSymbols(pids: List<UInt>, searchQuery: String): Flow<GetSymbolsRequestState>
}