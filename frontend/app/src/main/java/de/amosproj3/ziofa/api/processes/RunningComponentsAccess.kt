// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.processes

import kotlinx.coroutines.flow.Flow

/** Provides access to the list of running components */
interface RunningComponentsAccess {

    /** The list of running components retrieved from the backend */
    val runningComponentsList: Flow<List<RunningComponent>>
}
