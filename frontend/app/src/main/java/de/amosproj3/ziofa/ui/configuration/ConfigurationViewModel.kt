// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.BackendConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationUpdate
import de.amosproj3.ziofa.api.configuration.LocalConfigurationAccess
import de.amosproj3.ziofa.client.JniReferencesConfig
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.UprobeConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.configuration.data.ConfigurationScreenState
import de.amosproj3.ziofa.ui.shared.DURATION_THRESHOLD
import de.amosproj3.ziofa.ui.shared.toUIOptionsForPids
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import timber.log.Timber

class ConfigurationViewModel(
    private val localConfigurationAccess: LocalConfigurationAccess,
    private val backendConfigurationAccess: BackendConfigurationAccess,
    private val pids: List<UInt>,
) : ViewModel() {

    val configurationScreenState =
        localConfigurationAccess.localConfiguration
            .onEach { Timber.i("Update from UI: $it") }
            .map { it.toConfigurationScreenState(pids) }
            .stateIn(viewModelScope, SharingStarted.Eagerly, ConfigurationScreenState.Loading)

    val changed =
        combine(
                localConfigurationAccess.localConfiguration,
                backendConfigurationAccess.backendConfiguration,
            ) { local, backend ->
                local != backend
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, false)

    fun optionChanged(option: BackendFeatureOptions, active: Boolean) {
        if (configurationScreenState.value is ConfigurationScreenState.Valid) {
            when (option) {
                is BackendFeatureOptions.VfsWriteOption -> {
                    localConfigurationAccess.changeFeatureConfiguration(
                        enable = active,
                        vfsWriteFeature = VfsWriteConfig(pids.associateWith { DURATION_THRESHOLD }),
                    )
                }

                is BackendFeatureOptions.SendMessageOption -> {
                    localConfigurationAccess.changeFeatureConfiguration(
                        enable = active,
                        sendMessageFeature =
                            SysSendmsgConfig(
                                pids.associateWith { DURATION_THRESHOLD }
                                // TODO this is not a duration
                            ),
                    )
                }

                is BackendFeatureOptions.UprobeOption -> {
                    localConfigurationAccess.changeFeatureConfiguration(
                        enable = active,
                        uprobesFeature =
                            pids.map {
                                UprobeConfig(
                                    fnName = option.method,
                                    target = option.odexFilePath,
                                    offset = option.offset,
                                    pid = it,
                                )
                            },
                    )
                }

                is BackendFeatureOptions.JniReferencesOption->{
                    localConfigurationAccess.changeFeatureConfiguration(
                        enable = active,
                        jniReferencesFeature = JniReferencesConfig(pids)
                    )
                }

                else -> throw NotImplementedError("NO SUPPORT YET")
            }
        }
    }

    /** Submit the configuration changes to the backend. */
    fun configurationSubmitted() {
        viewModelScope.launch { localConfigurationAccess.submitConfiguration() }
    }

    private fun ConfigurationUpdate.toConfigurationScreenState(
        relevantPids: List<UInt>
    ): ConfigurationScreenState {
        return when (this) {
            is ConfigurationUpdate.Valid -> {
                ConfigurationScreenState.Valid(this.toUIOptionsForPids(relevantPids))
            }

            is ConfigurationUpdate.Invalid ->
                ConfigurationScreenState.Invalid(this.error.stackTraceToString())

            is ConfigurationUpdate.Unknown -> ConfigurationScreenState.Loading
        }
    }
}
