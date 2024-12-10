// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.reset

import androidx.lifecycle.ViewModel
import de.amosproj3.ziofa.api.configuration.BackendConfigurationAccess

// For consistency ;)
class ResetViewModel(private val backendConfigurationAccess: BackendConfigurationAccess) :
    ViewModel() {
    fun reset() {
        backendConfigurationAccess.reset()
    }
}
