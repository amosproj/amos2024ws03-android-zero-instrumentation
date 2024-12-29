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
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import timber.log.Timber

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

    private val EMPTY_CONFIGURATION = Configuration(null, null, listOf(), null)

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
                on<ConfigurationAction.Change>(executionPolicy = ExecutionPolicy.ORDERED)
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

                on<ConfigurationAction.Change> { action, state ->
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
        change: ConfigurationAction.Change
    ): Configuration {
        Timber.e("changeFeatureConfigurationForPIDs.prev $this")
        Timber.e(
            "changeFeatureConfigurationForPIDs() $change"
        )
        return this.copy(
            vfsWrite =
            change.vfsWriteFeature?.let { requestedChanges ->
                this.vfsWrite.updatePIDs(
                    pidsToAdd =
                    if (change.enable) requestedChanges.entries.entries else setOf(),
                    pidsToRemove =
                    if (!change.enable) requestedChanges.entries.entries else setOf(),
                )
            } ?: this.vfsWrite,
            sysSendmsg =
            change.sendMessageFeature?.let { requestedChanges ->
                this.sysSendmsg.updatePIDs(
                    pidsToAdd =
                    if (change.enable) requestedChanges.entries.entries else setOf(),
                    pidsToRemove =
                    if (!change.enable) requestedChanges.entries.entries else setOf(),
                )
            } ?: this.sysSendmsg,
            uprobes =
            change.uprobesFeature.let { requestedChanges ->
                if (requestedChanges == null)
                    return@let this.uprobes
                this.uprobes.updateUProbes(
                    pidsToAdd = if (change.enable) requestedChanges else listOf(),
                    pidsToRemove = if (!change.enable) requestedChanges else listOf(),
                )
            },
            jniReferences =
            change.jniReferencesFeature?.let { requestedChanges ->
                this.jniReferences.updatePIDs(
                    pidsToAdd = if (change.enable) requestedChanges.pids else listOf(),
                    pidsToRemove = if (!change.enable) requestedChanges.pids else listOf(),
                )
            } ?: this.jniReferences,
        ).also { Timber.i("new local configuration = $it") }
    }
}
