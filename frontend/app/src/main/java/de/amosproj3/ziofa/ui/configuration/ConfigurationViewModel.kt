// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationAction
import de.amosproj3.ziofa.api.configuration.ConfigurationState
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions
import de.amosproj3.ziofa.ui.configuration.data.ConfigurationScreenState
import de.amosproj3.ziofa.ui.shared.toConfigurationChangeAction
import de.amosproj3.ziofa.ui.shared.toUIOptionsForPids
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import timber.log.Timber

class ConfigurationViewModel(
    private val configurationAccess: ConfigurationAccess,
    private val pids: List<UInt>,
) : ViewModel() {


    val configurationScreenState =
        configurationAccess.configurationState
            .onEach { Timber.i("Update from UI: $it") }
            .map { it.toConfigurationScreenState(pids) }
            .stateIn(viewModelScope, SharingStarted.Eagerly, ConfigurationScreenState.Loading)

    val changed = configurationAccess.configurationState
        .map { it is ConfigurationState.Different }
        .stateIn(viewModelScope, SharingStarted.Lazily, false)

    fun optionChanged(option: BackendFeatureOptions, active: Boolean) {
        if (configurationScreenState.value is ConfigurationScreenState.Valid) {

            val change = option.toConfigurationChangeAction(
                pids = pids.toSet(),
                active = active,
            )

            viewModelScope.launch { configurationAccess.performAction(change) }
        }
    }

    /** Submit the configuration changes to the backend. */
    fun configurationSubmitted() {
        viewModelScope.launch { configurationAccess.performAction(ConfigurationAction.Synchronize) }
    }

    private fun ConfigurationState.toConfigurationScreenState(
        relevantPids: List<UInt>
    ): ConfigurationScreenState =
        when (this) {
            is ConfigurationState.Synchronized ->
                ConfigurationScreenState.Valid(this.configuration.toUIOptionsForPids(relevantPids))

            is ConfigurationState.Different ->
                ConfigurationScreenState.Valid(this.localConfiguration.toUIOptionsForPids(relevantPids))

            is ConfigurationState.Error->
                ConfigurationScreenState.Invalid(this.error.stackTraceToString())

            is ConfigurationState.Uninitialized -> ConfigurationScreenState.Loading
        }
}
