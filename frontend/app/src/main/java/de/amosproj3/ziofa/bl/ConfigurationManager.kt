// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl

import de.amosproj3.ziofa.api.ConfigurationAccess
import de.amosproj3.ziofa.api.ConfigurationUpdate
import de.amosproj3.ziofa.api.ProcessListAccess
import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.client.ClientException
import uniffi.shared.Configuration
import uniffi.shared.Process

class ConfigurationManager(val clientFactory: ClientFactory) :
    ProcessListAccess, ConfigurationAccess {

    private val coroutineScope = CoroutineScope(Dispatchers.IO)
    private var client: Client? = null

    override val processesList = MutableStateFlow<List<Process>>(listOf())
    override val configuration: MutableStateFlow<ConfigurationUpdate> =
        MutableStateFlow(ConfigurationUpdate.UNKNOWN)

    override fun submitConfiguration(configuration: Configuration) {
        coroutineScope.launch {
            client?.setConfiguration(configuration)
            getAndUpdateConfiguration() // "emulates" callback of changed configuration until
            // implemented
        }
    }

    init {
        coroutineScope.launch {
            try {
                client = clientFactory.connect()
                initializeConfigurationState()
                startProcessListUpdates()
            } catch (e: ClientException) {
                configuration.update { ConfigurationUpdate.NOK(e) }
            }
        }
    }

    private suspend fun initializeConfigurationState() {
        val initializedConfiguration =
            try {
                client!!.getConfiguration()
            } catch (e: ClientException) {
                // TODO this should be handled on the backend
                client!!.setConfiguration(Configuration(listOf()))
                client!!.getConfiguration()
            }
        configuration.update { ConfigurationUpdate.OK(initializedConfiguration) }
    }

    private suspend fun startProcessListUpdates() {
        while (true) {
            delay(1000)
            client?.let { client -> processesList.update { client.listProcesses() } }
                ?: processesList.update { listOf() }.also { Timber.w("Client not ready!") }
        }
    }

    private suspend fun getAndUpdateConfiguration() {
        configuration.update {
            try {
                (client?.getConfiguration()?.let { ConfigurationUpdate.OK(it) }
                        ?: ConfigurationUpdate.UNKNOWN)
                    .also { Timber.i("Received config $it") }
            } catch (e: Exception) {
                ConfigurationUpdate.NOK(e)
            }
        }
    }
}
