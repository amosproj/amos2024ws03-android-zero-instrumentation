// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
// SPDX-FileCopyrightText: 2024 Robin Seidl <robin.seidl@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl.configuration

import arrow.core.Either
import com.freeletics.flowredux.dsl.ExecutionPolicy
import com.freeletics.flowredux.dsl.FlowReduxStateMachine
import com.freeletics.flowredux.dsl.State
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationAction
import de.amosproj3.ziofa.api.configuration.ConfigurationState
import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.ui.configuration.utils.EMPTY_CONFIGURATION
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * This class is organized as a state machine with 4 states in [ConfigurationState].
 * Starting in state [ConfigurationState.Uninitialized], once the configuration is initially
 * retrieved from the backend, we transition to [ConfigurationState.Synchronized]. If the user makes
 * a change, the state will change to [ConfigurationState.Different] until the user
 * submits his configuration, which will change it back to [ConfigurationState.Synchronized].
 * Any errors in communication with the backend will transition to [ConfigurationState.Error] from
 * any state.
 * @param clientFactory the client factory for backend communication
 * */
@OptIn(ExperimentalCoroutinesApi::class)
class ConfigurationManager(clientFactory: ClientFactory) :
    FlowReduxStateMachine<ConfigurationState, ConfigurationAction>(
        initialState = ConfigurationState.Uninitialized(
            clientFactory = clientFactory
        )
    ), ConfigurationAccess {

    private val _configurationState =
        MutableStateFlow<ConfigurationState>(ConfigurationState.Uninitialized(clientFactory))

    override val configurationState = _configurationState.asStateFlow()
    override suspend fun performAction(action: ConfigurationAction) {
        this.dispatch(action)
    }

    init {
        startStateMachine()
        startUpdatingConfigurationState()
    }

    private fun startStateMachine() {
        spec {
            inState<ConfigurationState.Uninitialized> {
                onEnter { state ->
                    val client = state.snapshot.clientFactory.connect()
                    val configuration = client.initializeConfiguration()
                    state.override { configuration.synchronizedOrErrorState(client) }
                }
            }

            inState<ConfigurationState.Synchronized> {
                on<ConfigurationAction.ChangeFeature>(executionPolicy = ExecutionPolicy.ORDERED)
                { action, state -> state.applyChangeAndTransitionToDifferent(action) }

                on<ConfigurationAction.Reset> { _, state ->
                    val updatedConfig =
                        state.snapshot.client.updateBackendConfiguration(EMPTY_CONFIGURATION)
                    state.override { updatedConfig.synchronizedOrErrorState(client) }
                }
            }
            inState<ConfigurationState.Different> {
                on<ConfigurationAction.Synchronize> { _, state ->
                    val currentState = state.snapshot
                    val updatedConfig =
                        currentState.client.updateBackendConfiguration(currentState.localConfiguration)
                    state.override { updatedConfig.synchronizedOrErrorState(client) }
                }

                on<ConfigurationAction.ChangeFeature> { action, state ->
                    state.applyChangeLocally(action)
                }

                on<ConfigurationAction.Reset> { _, state ->
                    val updatedConfig =
                        state.snapshot.client.updateBackendConfiguration(EMPTY_CONFIGURATION)
                    state.override { updatedConfig.synchronizedOrErrorState(client) }
                }
            }
        }
    }

    private fun startUpdatingConfigurationState() {
        CoroutineScope(Dispatchers.IO).launch {
            this@ConfigurationManager.state.collect {
                _configurationState.value = it
            }
        }
    }

    private fun State<ConfigurationState.Different>.applyChangeLocally(action: ConfigurationAction.ChangeFeature) =
        this.mutate { this.copy(localConfiguration = this.localConfiguration.applyChange(action)) }

    private fun State<ConfigurationState.Synchronized>.applyChangeAndTransitionToDifferent(action: ConfigurationAction.ChangeFeature) =
        this.override {
            ConfigurationState.Different(
                client = this.client,
                localConfiguration = this.configuration.applyChange(action),
                backendConfiguration = this.configuration
            )
        }

    private fun Either<Throwable, Configuration>.synchronizedOrErrorState(
        client: Client,
    ) = this.fold(
        ifLeft = { ConfigurationState.Error(it) },
        ifRight = { ConfigurationState.Synchronized(client, it) }
    )


    private suspend fun Client.updateBackendConfiguration(configuration: Configuration) =
        Either.catch {
            this.setConfiguration(configuration)
            this.getConfiguration()
        }

    private suspend fun Client.initializeConfiguration() =
        try {
            Either.Right(this.getConfiguration())
        } catch (e: Exception) {
            this.updateBackendConfiguration(EMPTY_CONFIGURATION)
        }
}
