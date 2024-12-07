package de.amosproj3.ziofa.bl.configuration

import de.amosproj3.ziofa.api.configuration.GetSymbolsRequestState
import de.amosproj3.ziofa.api.configuration.SymbolsAccess
import de.amosproj3.ziofa.ui.symbols.data.SymbolsEntry
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf

class UProbeManager : SymbolsAccess {

    // TODO REMOVE
    private suspend fun mockedGetOdexFilesFlow(pid: UInt) = flowOf<String>()

    private suspend fun mockedSymbolFlow(odexFile: String) = flowOf<String>()

    // TODO REMOVE

    override fun searchSymbols(
        pids: List<UInt>,
        searchQuery: String,
    ): Flow<GetSymbolsRequestState> = flow {
        emit(GetSymbolsRequestState.Loading)
        delay(1000)
        emit(
            GetSymbolsRequestState.Response(
                symbols =
                    listOf(
                        SymbolsEntry(
                            name =
                                "void kotlin.collections.ArraysKt___ArraysKt\\\$asSequence\\\$\\\$inlined\\\$Sequence\\\$2.<init>(byte[])",
                            odexFile = "",
                            1u,
                        ),
                        SymbolsEntry(
                            name =
                                "boolean androidx.compose.ui.platform.ViewLayer\\\$Companion.getHasRetrievedMethod()",
                            odexFile = "",
                            1u,
                        ),
                        SymbolsEntry(
                            name =
                                "byte androidx.emoji2.text.flatbuffer.FlexBuffers\\\$Blob.get(int)",
                            odexFile = "",
                            1u,
                        ),
                    )
            )
        )
    }
}
