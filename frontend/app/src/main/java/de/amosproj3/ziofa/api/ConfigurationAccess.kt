// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api

import kotlinx.coroutines.flow.StateFlow
import uniffi.shared.Configuration

interface ConfigurationAccess {
    val configuration: StateFlow<ConfigurationUpdate>

    fun submitConfiguration(configuration: Configuration)
}
