package de.amosproj3.ziofa.api.configuration

sealed class IndexingRequestState {
    data object NotStarted : IndexingRequestState()
    data object Started : IndexingRequestState()
    data object Done : IndexingRequestState()
    data class Error(val error: Throwable) : IndexingRequestState()
}