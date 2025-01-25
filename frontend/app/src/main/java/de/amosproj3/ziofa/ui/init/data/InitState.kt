package de.amosproj3.ziofa.ui.init.data

sealed class InitState {
    data object Initializing : InitState()

    data object Initialized : InitState()

    data class Error(val error: String) : InitState()
}
