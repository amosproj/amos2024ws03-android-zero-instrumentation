// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

import kotlinx.collections.immutable.ImmutableList

sealed class ConfigurationScreenState {
    data class Valid(val options: ImmutableList<BackendFeatureOptions>) :
        ConfigurationScreenState()

    data class Invalid(val errorMessage: String) : ConfigurationScreenState()

    data object Loading : ConfigurationScreenState()
}
