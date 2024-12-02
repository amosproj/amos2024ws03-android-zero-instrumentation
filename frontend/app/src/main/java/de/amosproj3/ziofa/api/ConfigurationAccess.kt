// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api

import de.amosproj3.ziofa.client.Configuration
import kotlinx.coroutines.flow.StateFlow

interface ConfigurationAccess {
    val configuration: StateFlow<ConfigurationUpdate>

    fun submitConfiguration(configuration: Configuration)
}
