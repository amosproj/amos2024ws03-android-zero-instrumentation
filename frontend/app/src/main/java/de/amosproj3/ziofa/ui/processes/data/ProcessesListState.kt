// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes.data

import de.amosproj3.ziofa.api.processes.RunningComponent
import kotlinx.collections.immutable.ImmutableList

sealed class ProcessesListState {
    data object Loading : ProcessesListState()

    data object NoResults : ProcessesListState()

    data class Valid(val list: ImmutableList<RunningComponent>) : ProcessesListState()
}
