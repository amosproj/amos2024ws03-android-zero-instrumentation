// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.ConfigurationAccess
import de.amosproj3.ziofa.api.ConfigurationUpdate
import de.amosproj3.ziofa.ui.configuration.data.ConfigurationScreenState
import de.amosproj3.ziofa.ui.configuration.data.EbpfProgramOptions
import de.amosproj3.ziofa.ui.configuration.data.VfsWriteOption
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import timber.log.Timber
import uniffi.shared.Configuration
import uniffi.shared.VfsWriteConfig

class ConfigurationViewModel(val configurationAccess: ConfigurationAccess) : ViewModel() {

    private val _changed = MutableStateFlow(false)
    val changed = _changed.stateIn(viewModelScope, SharingStarted.Lazily, false)

    private val checkedOptions =
        MutableStateFlow(
            EbpfProgramOptions(vfsWriteOption = VfsWriteOption(enabled = false, pids = listOf()))
        )

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

    fun vfsWriteChanged(pids: IntArray?, newState: Boolean) {
        checkedOptions.update {
            it.copy(
                vfsWriteOption =
                    VfsWriteOption(
                        enabled = newState,
                        pids = pids?.let { it.map { it.toUInt() } } ?: listOf(),
                    )
            )
        }
        _configurationScreenState.update { ConfigurationScreenState.Valid(checkedOptions.value) }
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
                checkedOptions.update { this.toUIOptions() }
                ConfigurationScreenState.Valid(checkedOptions.value)
            }

            is ConfigurationUpdate.Invalid ->
                ConfigurationScreenState.Invalid(this.error.stackTraceToString())

            is ConfigurationUpdate.Unknown -> ConfigurationScreenState.Loading
        }
    }

    private fun ConfigurationUpdate.Valid.toUIOptions(): EbpfProgramOptions {
        val vfsOption =
            this.configuration.vfsWrite?.let { VfsWriteOption(enabled = true, pids = it.pids) }
                ?: VfsWriteOption(enabled = false, pids = listOf())

        return EbpfProgramOptions(vfsWriteOption = vfsOption)
    }

    private fun EbpfProgramOptions.toConfiguration(): Configuration {
        val vfsConfig =
            if (this.vfsWriteOption.enabled) {
                VfsWriteConfig(this.vfsWriteOption.pids)
            } else null

        return Configuration(vfsWrite = vfsConfig, uprobes = listOf())
    }
}
