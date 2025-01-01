// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.StateFlow

interface ConfigurationAccess {
    val configurationState: StateFlow<ConfigurationState>

    suspend fun performAction(action: ConfigurationAction)
}
