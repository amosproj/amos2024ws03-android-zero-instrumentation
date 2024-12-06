package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.Flow

interface SymbolsAccess {
    fun getOdexFilesForPid(pid: UInt): Flow<GetOdexFilesRequestState>
    fun getSymbolsForFile(odexFile: String): Flow<GetSymbolsRequestState>
}