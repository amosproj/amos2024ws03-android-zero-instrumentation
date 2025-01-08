// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.reset

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.configuration.ConfigurationAccess
import de.amosproj3.ziofa.api.configuration.ConfigurationAction
import kotlinx.coroutines.launch

// For consistency ;)
class ResetViewModel(private val configurationAccess: ConfigurationAccess) : ViewModel() {
    fun reset() {
        viewModelScope.launch { configurationAccess.performAction(ConfigurationAction.Reset) }
    }
}
