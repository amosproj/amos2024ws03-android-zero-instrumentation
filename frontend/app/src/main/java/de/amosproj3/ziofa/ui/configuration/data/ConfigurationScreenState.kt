// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

sealed class ConfigurationScreenState {
    data class Valid(val options: List<BackendFeatureOptions>) : ConfigurationScreenState()

    data class Invalid(val errorMessage: String) : ConfigurationScreenState()

    data object Loading : ConfigurationScreenState()
}
