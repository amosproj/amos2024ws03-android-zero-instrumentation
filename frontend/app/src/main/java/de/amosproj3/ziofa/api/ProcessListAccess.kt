// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api

import de.amosproj3.ziofa.client.Process
import kotlinx.coroutines.flow.StateFlow

interface ProcessListAccess {
    val processesList: StateFlow<List<Process>>
}
