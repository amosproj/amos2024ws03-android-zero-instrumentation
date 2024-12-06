package de.amosproj3.ziofa.api.configuration


sealed class GetOdexFilesRequestState {
    data class Response(val odexFiles: List<String>) : GetOdexFilesRequestState()
    data class Error(val errorMessage: String) : GetOdexFilesRequestState()
    data object Loading : GetOdexFilesRequestState()
}

sealed class GetSymbolsRequestState {
    data class Response(val symbols: List<String>) : GetSymbolsRequestState()
    data class Error(val errorMessage: String) : GetSymbolsRequestState()
    data object Loading : GetSymbolsRequestState()
}