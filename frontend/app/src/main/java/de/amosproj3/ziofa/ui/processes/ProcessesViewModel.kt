// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class ProcessesViewModel(runningComponentsProvider: RunningComponentsAccess) : ViewModel() {

    val applicationsAndProcessesList =
        runningComponentsProvider.runningComponentsList
            .sortApplicationsFirst()
            .stateIn(viewModelScope, started = SharingStarted.Lazily, listOf())

    private fun Flow<List<RunningComponent>>.sortApplicationsFirst() =
        this.map { list -> list.sortedBy { if (it is RunningComponent.Application) -1 else 1 } }
}
