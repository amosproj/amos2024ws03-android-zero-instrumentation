package de.amosproj3.ziofa.bl.configuration

import de.amosproj3.ziofa.api.configuration.GetOdexFilesRequestState
import de.amosproj3.ziofa.api.configuration.GetSymbolsRequestState
import de.amosproj3.ziofa.api.configuration.SymbolsAccess
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.toList


class UProbeManager : SymbolsAccess {



    //TODO REMOVE
    private suspend fun mockedGetOdexFilesFlow(pid: UInt) = flowOf<String>()
    private suspend fun mockedSymbolFlow(odexFile: String) = flowOf<String>()
    //TODO REMOVE


    override fun getOdexFilesForPid(pid: UInt) =
        flow {
            emit(GetOdexFilesRequestState.Loading)
            try {
                emit(
                    GetOdexFilesRequestState.Response(
                        mockedGetOdexFilesFlow(pid)
                            .toList().sorted()
                    )
                )
            } catch (e: Exception) {
                emit(
                    GetOdexFilesRequestState.Error(e.stackTraceToString())
                )
            }
        }

    override fun getSymbolsForFile(odexFile: String) =
        flow {
            emit(GetSymbolsRequestState.Loading)
            try {
                emit(
                    GetSymbolsRequestState.Response(
                        mockedSymbolFlow(odexFile)
                            .toList()
                            .sorted()
                    )
                )
            } catch (e: Exception) {
                emit(GetSymbolsRequestState.Error(e.stackTraceToString()))
            }
        }



}

