package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Configuration

sealed class ConfigurationState {
    data class Uninitialized(val clientFactory: ClientFactory) : ConfigurationState()
    data class Synchronized(
        val client: Client,
        val configuration: Configuration
    ) : ConfigurationState()

    data class Different(
        val client: Client,
        val localConfiguration: Configuration,
        val backendConfiguration: Configuration
    ) : ConfigurationState()

    data class Error(val error: Throwable) : ConfigurationState()
}