// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

import kotlinx.collections.immutable.ImmutableList

/** Data class for the data to display on the ConfigurationScreen */
sealed class ConfigurationScreenState {

    /** The configuration screen data is valid and can be displayed */
    data class Valid(val options: ImmutableList<BackendFeatureOptions>) :
        ConfigurationScreenState()

    /**
     * Due to an error with the backend or a missing mapping, we cannot display the configuration
     * screen and a error message is displayed instead.
     */
    data class Invalid(val errorMessage: String) : ConfigurationScreenState()

    /** An operation for retrieving the configuration data is pending */
    data object Loading : ConfigurationScreenState()
}
