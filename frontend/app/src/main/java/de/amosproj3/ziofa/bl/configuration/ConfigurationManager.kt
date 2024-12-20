// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl.configuration

import de.amosproj3.ziofa.api.configuration.BackendConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationUpdate
import de.amosproj3.ziofa.api.configuration.LocalConfigurationAccess
import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import timber.log.Timber

class ConfigurationManager(val clientFactory: ClientFactory) :
    BackendConfigurationAccess, LocalConfigurationAccess {

    private val coroutineScope = CoroutineScope(Dispatchers.IO)
    private var client: Client? = null

    override val backendConfiguration: MutableStateFlow<ConfigurationUpdate> =
        MutableStateFlow(ConfigurationUpdate.Unknown)

    private val _localConfiguration = MutableStateFlow<ConfigurationUpdate?>(null)

    override val localConfiguration =
        _localConfiguration
            .onEach { Timber.i("local configuration updated $it") }
            .map { it ?: ConfigurationUpdate.Unknown }

    init {
        coroutineScope.launch {
            try {
                client = clientFactory.connect()
                initializeConfigurations()
            } catch (e: Exception) {
                backendConfiguration.update { ConfigurationUpdate.Invalid(e) }
            }
        }
    }

    override fun changeFeatureConfiguration(
        enable: Boolean,
        vfsWriteFeature: VfsWriteConfig?,
        sendMessageFeature: SysSendmsgConfig?,
        uprobesFeature: List<UprobeConfig>?,
        jniReferencesFeature: JniReferencesConfig?,
    ) {
        _localConfiguration.update { prev ->
            Timber.e("changeFeatureConfigurationForPIDs.prev $prev")
            Timber.e(
                "changeFeatureConfigurationForPIDs() $vfsWriteFeature, $sendMessageFeature, $uprobesFeature, $jniReferencesFeature"
            )
            // the configuration shall not be changed from the UI if there is none received from
            // backend
            if (prev != null && prev is ConfigurationUpdate.Valid) {
                val previousConfiguration = prev.configuration
                previousConfiguration
                    .copy(
                        vfsWrite =
                            vfsWriteFeature?.let { requestedChanges ->
                                previousConfiguration.vfsWrite.updatePIDs(
                                    pidsToAdd =
                                        if (enable) requestedChanges.entries.entries else setOf(),
                                    pidsToRemove =
                                        if (!enable) requestedChanges.entries.entries else setOf(),
                                )
                            } ?: previousConfiguration.vfsWrite,
                        sysSendmsg =
                            sendMessageFeature?.let { requestedChanges ->
                                previousConfiguration.sysSendmsg.updatePIDs(
                                    pidsToAdd =
                                        if (enable) requestedChanges.entries.entries else setOf(),
                                    pidsToRemove =
                                        if (!enable) requestedChanges.entries.entries else setOf(),
                                )
                            } ?: previousConfiguration.sysSendmsg,
                        uprobes =
                            uprobesFeature.let { requestedChanges ->
                                if (requestedChanges == null)
                                    return@let previousConfiguration.uprobes
                                previousConfiguration.uprobes.updateUProbes(
                                    pidsToAdd = if (enable) requestedChanges else listOf(),
                                    pidsToRemove = if (!enable) requestedChanges else listOf(),
                                )
                            },
                        jniReferences =
                            jniReferencesFeature?.let { requestedChanges ->
                                previousConfiguration.jniReferences.updatePIDs(
                                    pidsToAdd = if (enable) requestedChanges.pids else listOf(),
                                    pidsToRemove = if (!enable) requestedChanges.pids else listOf(),
                                )
                            } ?: previousConfiguration.jniReferences,
                    )
                    .also { Timber.i("new local configuration = $it") }
                    .let { ConfigurationUpdate.Valid(it) }
            } else return@update prev
        }
    }

    override fun submitConfiguration() {
        coroutineScope.launch {
            sendLocalToBackend()
            updateBothConfigurations(
                getFromBackend()
            ) // "emulates" callback of changed configuration until
        }
    }

    override fun reset() {
        runBlocking {
            client?.setConfiguration(Configuration(null, null, listOf(), null))
            updateBothConfigurations(getFromBackend())
        }
    }

    private suspend fun initializeConfigurations() {
        val initializedConfiguration =
            try {
                ConfigurationUpdate.Valid(client!!.getConfiguration())
            } catch (e: Exception) {
                getOrCreateInitialConfiguration()
            }
        updateBothConfigurations(initializedConfiguration)
    }

    // TODO this should be handled on the backend
    private suspend fun getOrCreateInitialConfiguration(): ConfigurationUpdate {
        return try {
            // the config may not be initialized, we should try initializing it
            client!!.setConfiguration(
                Configuration(
                    vfsWrite = null,
                    sysSendmsg = null,
                    uprobes = listOf(),
                    jniReferences = null,
                )
            )
            ConfigurationUpdate.Valid(client!!.getConfiguration())
        } catch (e: Exception) {
            return ConfigurationUpdate.Invalid(e)
        }
    }

    private suspend fun sendLocalToBackend() {
        _localConfiguration.value?.let {
            if (it is ConfigurationUpdate.Valid) client?.setConfiguration(it.configuration)
        } ?: Timber.e("unsubmittedConfiguration == null -> this should never happen")
    }

    private suspend fun getFromBackend(): ConfigurationUpdate {
        return try {
            (client?.getConfiguration()?.let { ConfigurationUpdate.Valid(it) }
                    ?: ConfigurationUpdate.Unknown)
                .also { Timber.i("Received config $it") }
        } catch (e: Exception) {
            ConfigurationUpdate.Invalid(e)
        }
    }

    private fun updateBothConfigurations(configurationUpdate: ConfigurationUpdate) {
        backendConfiguration.value = configurationUpdate
        _localConfiguration.value = configurationUpdate
    }
}
