// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.ui.processes.data.ProcessesListState
import de.amosproj3.ziofa.ui.shared.getDisplayName
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

class ProcessesViewModel(runningComponentsProvider: RunningComponentsAccess) : ViewModel() {

    private val searchQuery = MutableStateFlow<String?>(null)

    val processListState =
        combine(runningComponentsProvider.runningComponentsList, searchQuery) {
                runningComponents,
                query ->
                if (query == null)
                    return@combine ProcessesListState.Valid(
                        runningComponents.sortApplicationsFirst().sortZIOFAFirst().toImmutableList()
                    )
                val filtered =
                    runningComponents.filter {
                        it.getDisplayName().lowercase().contains(query.lowercase())
                    }
                if (filtered.isEmpty()) ProcessesListState.NoResults
                else
                    ProcessesListState.Valid(
                        filtered.sortApplicationsFirst().sortZIOFAFirst().toImmutableList()
                    )
            }
            .stateIn(viewModelScope, started = SharingStarted.Lazily, ProcessesListState.Loading)

    fun startSearch(query: String) {
        searchQuery.value = query
    }

    private fun List<RunningComponent>.sortApplicationsFirst() =
        this.sortedBy { if (it is RunningComponent.Application) -1 else 1 }

    /** Most important app first. ;) */
    private fun List<RunningComponent>.sortZIOFAFirst() =
        this.sortedBy {
            if (it is RunningComponent.Application && it.packageInfo.displayName == "ZIOFA") -1
            else 1
        }
}
