// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration.data

sealed class ConfigurationScreenState {
    data class LIST(val options: List<EBpfProgramOption>) : ConfigurationScreenState()

    data class ERROR(val errorMessage: String) : ConfigurationScreenState()

    data object LOADING : ConfigurationScreenState()
}
