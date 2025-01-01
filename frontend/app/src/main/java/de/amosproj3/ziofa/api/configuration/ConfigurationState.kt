// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Configuration

/** State of the configuration */
sealed class ConfigurationState {
    /** Initial state */
    data class Uninitialized(val clientFactory: ClientFactory) : ConfigurationState()

    /** In-memory configuration and remote configuration are equivalent */
    data class Synchronized(val client: Client, val configuration: Configuration) :
        ConfigurationState()

    /**
     * In-memory configuration and remote configuration differ (i.e. there are unsynchronized local
     * changes)
     */
    data class Different(
        val client: Client,
        val localConfiguration: Configuration,
        val backendConfiguration: Configuration,
    ) : ConfigurationState()

    /** An error has occured during communication with the backend */
    data class Error(val error: Throwable) : ConfigurationState()
}
