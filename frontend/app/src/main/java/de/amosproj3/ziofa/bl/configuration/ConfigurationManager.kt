// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl.configuration

import arrow.core.Either
import com.freeletics.flowredux.dsl.ExecutionPolicy
import com.freeletics.flowredux.dsl.FlowReduxStateMachine
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationAction
import de.amosproj3.ziofa.api.configuration.ConfigurationState
import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.configuration.utils.EMPTY_CONFIGURATION
import de.amosproj3.ziofa.ui.shared.DURATION_THRESHOLD
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
class ConfigurationManager(clientFactory: ClientFactory) :
    FlowReduxStateMachine<ConfigurationState, ConfigurationAction>(
        initialState = ConfigurationState.Uninitialized(
            clientFactory = clientFactory
        )
    ), ConfigurationAccess {

    init {
        initializeStateMachine()
    }


    private val _configurationState =
        MutableStateFlow<ConfigurationState>(ConfigurationState.Uninitialized(clientFactory))

    override val configurationState = _configurationState.asStateFlow()

    override suspend fun performAction(action: ConfigurationAction) {
        this.dispatch(action)
    }

    private fun initializeStateMachine() {
        spec {
            inState<ConfigurationState.Uninitialized> {
                onEnter { state ->
                    val client = state.snapshot.clientFactory.connect()
                    val configuration = initializeConfiguration(client)
                    state.override {
                        configuration.fold(
                            ifLeft = { ConfigurationState.Error(it) },
                            ifRight = { ConfigurationState.Synchronized(client, it) }
                        )
                    }
                }
            }

            inState<ConfigurationState.Synchronized> {
                on<ConfigurationAction.ChangeFeature>(executionPolicy = ExecutionPolicy.ORDERED)
                { action, state ->
                    state.override {
                        ConfigurationState.Different(
                            client = this.client,
                            localConfiguration = this.configuration.applyChange(action),
                            backendConfiguration = this.configuration
                        )
                    }
                }

                on<ConfigurationAction.Reset> { _, state ->
                    val updatedConfig =
                        state.snapshot.client.updateBackendConfiguration(EMPTY_CONFIGURATION)
                    state.override {
                        updatedConfig.fold(
                            ifLeft = { ConfigurationState.Error(it) },
                            ifRight = { ConfigurationState.Synchronized(client, it) }
                        )
                    }
                }
            }
            inState<ConfigurationState.Different> {
                on<ConfigurationAction.Synchronize> { _, state ->
                    val currentState = state.snapshot
                    val updatedConfig =
                        currentState.client.updateBackendConfiguration(currentState.localConfiguration)
                    state.override {
                        updatedConfig.fold(
                            ifLeft = { ConfigurationState.Error(it) },
                            ifRight = { ConfigurationState.Synchronized(client, it) }
                        )
                    }
                }

                on<ConfigurationAction.ChangeFeature> { action, state ->
                    state.override {
                        ConfigurationState.Different(
                            client = this.client,
                            localConfiguration = this.localConfiguration.applyChange(action),
                            backendConfiguration = this.backendConfiguration
                        )
                    }
                }

                on<ConfigurationAction.Reset> { _, state ->
                    val updatedConfig =
                        state.snapshot.client.updateBackendConfiguration(EMPTY_CONFIGURATION)
                    state.override {
                        updatedConfig.fold(
                            ifLeft = { ConfigurationState.Error(it) },
                            ifRight = { ConfigurationState.Synchronized(client, it) }
                        )
                    }
                }
            }
        }

        CoroutineScope(Dispatchers.IO).launch {
            this@ConfigurationManager.state.collect {
                _configurationState.value = it
            }
        }

    }

    private suspend fun Client.updateBackendConfiguration(configuration: Configuration) =
        Either.catch {
            this.setConfiguration(configuration)
            this.getConfiguration()
        }

    private suspend fun initializeConfiguration(client: Client) =
        try {
            Either.Right(client.getConfiguration())
        } catch (e: Exception) {
            client.updateBackendConfiguration(EMPTY_CONFIGURATION)
        }

    private fun Configuration.applyChange(
        action: ConfigurationAction.ChangeFeature,
    ): Configuration {

        val feature = action.backendFeature
        val enable = action.enable
        val pids = action.pids

        return when (feature) {
            is BackendFeatureOptions.VfsWriteOption ->
                this.copy(
                    vfsWrite = this.vfsWrite?.updatePIDs(
                        pidsToAdd = if (enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf(),
                        pidsToRemove = if (!enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf()
                    )
                )

            is BackendFeatureOptions.SendMessageOption ->
                this.copy(
                    sysSendmsg = this.sysSendmsg?.updatePIDs(
                        pidsToAdd = if (enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf(),
                        pidsToRemove = if (!enable) pids.associateWith { DURATION_THRESHOLD }.entries else setOf()
                    )
                )

            is BackendFeatureOptions.JniReferencesOption ->
                this.copy(
                    jniReferences = this.jniReferences?.updatePIDs(
                        pidsToAdd = if (enable) pids else setOf(),
                        pidsToRemove = if (!enable) pids else setOf()
                    )
                )

            is BackendFeatureOptions.UprobeOption -> {
                val uprobeUpdate = pids.map {
                    UprobeConfig(
                        fnName = feature.method,
                        target = feature.odexFilePath,
                        offset = feature.offset,
                        pid = it,
                    )
                }
                this.copy(
                    uprobes = this.uprobes.updateUProbes(
                        pidsToAdd = if (enable) uprobeUpdate else listOf(),
                        pidsToRemove = if (!enable) uprobeUpdate else listOf()
                    )
                )
            }

        }
    }
}
