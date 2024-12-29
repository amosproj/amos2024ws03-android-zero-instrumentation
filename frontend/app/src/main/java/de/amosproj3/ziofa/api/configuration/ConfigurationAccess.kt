package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.StateFlow

interface ConfigurationAccess {
    val configurationState: StateFlow<ConfigurationState>
    suspend fun performAction(action: ConfigurationAction)
}