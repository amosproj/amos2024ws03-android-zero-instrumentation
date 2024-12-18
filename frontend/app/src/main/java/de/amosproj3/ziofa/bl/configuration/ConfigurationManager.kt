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
import de.amosproj3.ziofa.ui.shared.merge
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
            .map {
                if (it is ConfigurationUpdate.Valid) {
                    it.copy(it.configuration.copy(jniReferences = null))
                } else it
            } // TODO remove this once the backend has integrated setting this feature
            .onEach { Timber.i("local configuration updated $it") }
            .map { it ?: ConfigurationUpdate.Unknown }

    init {
        @Suppress("TooGenericExceptionCaught") // we want to display all exceptions
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
                "changeFeatureConfigurationForPIDs() " +
                    "vfs=$vfsWriteFeature, sendMsg=$sendMessageFeature, " +
                    "uprobes=$uprobesFeature, jni=$jniReferencesFeature"
            )
            // the configuration shall not be changed from the UI if there is none received from
            // backend
            if (prev != null && prev is ConfigurationUpdate.Valid) {
                val previousConfiguration = prev.configuration
                previousConfiguration
                    .copy(
                        vfsWrite = previousConfiguration.merge(vfsWriteFeature, enable),
                        sysSendmsg = previousConfiguration.merge(sendMessageFeature, enable),
                        uprobes = previousConfiguration.merge(uprobesFeature, enable),
                        jniReferences = previousConfiguration.merge(jniReferencesFeature, enable),
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

    @Suppress("TooGenericExceptionCaught", "SwallowedException") // initialization mechanism
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
    @Suppress("TooGenericExceptionCaught") // we want to display all exceptions
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

    @Suppress("TooGenericExceptionCaught") // we want to display all exceptions
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
