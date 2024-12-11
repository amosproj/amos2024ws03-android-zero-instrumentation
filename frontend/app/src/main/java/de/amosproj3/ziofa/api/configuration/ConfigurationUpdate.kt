// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.configuration

import de.amosproj3.ziofa.client.Configuration

sealed class ConfigurationUpdate {
    data class Valid(val configuration: Configuration) : ConfigurationUpdate()

    data class Invalid(val error: Throwable) : ConfigurationUpdate()

    data object Unknown : ConfigurationUpdate()
}
