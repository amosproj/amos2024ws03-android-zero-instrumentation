// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.init

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationState
import de.amosproj3.ziofa.ui.init.data.InitState
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class InitViewModel(configurationAccess: ConfigurationAccess) : ViewModel() {
    val initState: StateFlow<InitState> =
        configurationAccess.configurationState
            .map {
                when (it) {
                    is ConfigurationState.Uninitialized -> InitState.Initializing
                    is ConfigurationState.Error -> InitState.Error(it.error.stackTraceToString())
                    is ConfigurationState.Different,
                    is ConfigurationState.Synchronized -> InitState.Initialized
                }
            }
            .stateIn(viewModelScope, SharingStarted.Lazily, InitState.Initializing)
}
