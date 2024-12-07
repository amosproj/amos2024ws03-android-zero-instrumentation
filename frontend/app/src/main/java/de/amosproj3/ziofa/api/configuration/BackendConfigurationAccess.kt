// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import kotlinx.coroutines.flow.StateFlow

interface BackendConfigurationAccess {

    /** Only emits updates from the backend that are actually confirmed to be active */
    val backendConfiguration: StateFlow<ConfigurationUpdate>

    /** Clear the backend configuration. */
    fun reset()
}
