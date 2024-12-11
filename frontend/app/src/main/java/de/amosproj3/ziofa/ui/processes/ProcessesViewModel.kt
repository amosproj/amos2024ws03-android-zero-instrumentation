// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.ui.shared.getDisplayName
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class ProcessesViewModel(runningComponentsProvider: RunningComponentsAccess) : ViewModel() {

    private val searchQuery = MutableStateFlow<String?>(null)

    val applicationsAndProcessesList =
        combine(
            runningComponentsProvider.runningComponentsList,
            searchQuery
        ) { runningComponents, query ->
            if (query == null) return@combine runningComponents
            runningComponents.filter { it.getDisplayName().lowercase().contains(query.lowercase()) }
        }
            .sortApplicationsFirst()
            .stateIn(viewModelScope, started = SharingStarted.Lazily, listOf())


    fun startSearch(query: String) {
        searchQuery.value = query
    }

    private fun Flow<List<RunningComponent>>.sortApplicationsFirst() =
        this.map { list -> list.sortedBy { if (it is RunningComponent.Application) -1 else 1 } }
}
