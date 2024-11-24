// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.ConfigurationAccess
import de.amosproj3.ziofa.api.ConfigurationUpdate
import de.amosproj3.ziofa.ui.configuration.data.ConfigurationScreenState
import de.amosproj3.ziofa.ui.configuration.data.EBpfProgramOption
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.shared.Configuration

class ConfigurationViewModel(val configurationAccess: ConfigurationAccess) : ViewModel() {

    private val _changed = MutableStateFlow(false)
    val changed = _changed.stateIn(viewModelScope, SharingStarted.Lazily, false)

    private val checkedOptions =
        MutableStateFlow<MutableMap<String, EBpfProgramOption>>(mutableMapOf())

    private val _configurationScreenState =
        MutableStateFlow<ConfigurationScreenState>(ConfigurationScreenState.Loading)
    val configurationScreenState: StateFlow<ConfigurationScreenState> =
        _configurationScreenState
            .onEach { Timber.i(it.toString()) }
            .stateIn(viewModelScope, SharingStarted.Eagerly, ConfigurationScreenState.Loading)

    init {
        viewModelScope.launch { updateUIFromBackend() }
    }

    private suspend fun updateUIFromBackend() {
        configurationAccess.configuration.collect { receivedConfiguration ->
            _configurationScreenState.update { receivedConfiguration.toUIUpdate() }
        }
    }

    fun optionChanged(eBpfProgramOption: EBpfProgramOption, newState: Boolean) {
        checkedOptions.update { currentMap ->
            currentMap.computeIfPresent(eBpfProgramOption.name) { _, oldValue ->
                oldValue.copy(active = newState)
            }
            currentMap
        }
        _configurationScreenState.update {
            ConfigurationScreenState.Valid(checkedOptions.value.values.toList())
        }
        _changed.update { true }
    }

    /**
     * Submit the configuration to the backend.
     *
     * @param pids the affected Process IDs or null if the configuration should be set globally
     */
    fun configurationSubmitted(pids: IntArray?) {
        viewModelScope.launch {
            configurationAccess.submitConfiguration(checkedOptions.value.toConfiguration())
        }
        _changed.update { false }
    }

    private fun ConfigurationUpdate.toUIUpdate(): ConfigurationScreenState {
        return when (this) {
            is ConfigurationUpdate.Valid -> {
                checkedOptions.update { this.toUIOptions().associateBy { it.name }.toMutableMap() }
                ConfigurationScreenState.Valid(checkedOptions.value.values.toList())
            }

            is ConfigurationUpdate.Invalid ->
                ConfigurationScreenState.Invalid(this.error.stackTraceToString())

            is ConfigurationUpdate.Unknown -> ConfigurationScreenState.Loading
        }
    }

    private fun ConfigurationUpdate.Valid.toUIOptions(): List<EBpfProgramOption> {
        return this.configuration.entries.map {
            EBpfProgramOption(it.hrName, active = it.attach, true, it)
        }
    }

    private fun MutableMap<String, EBpfProgramOption>.toConfiguration(): Configuration {
        return Configuration(this.values.map { it.ebpfEntry })
    }
}
