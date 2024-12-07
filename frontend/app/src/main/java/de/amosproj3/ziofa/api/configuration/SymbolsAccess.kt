package de.amosproj3.ziofa.api.configuration

import android.provider.Contacts.Intents.UI
import kotlinx.coroutines.flow.Flow

interface SymbolsAccess {
    fun getOdexFilesForPid(pid: UInt): Flow<GetOdexFilesRequestState>
    fun getSymbolsForFile(odexFile: String): Flow<GetSymbolsRequestState>


    fun searchSymbols(pid: UInt, searchQuery: String): Flow<GetSymbolsRequestState>
}