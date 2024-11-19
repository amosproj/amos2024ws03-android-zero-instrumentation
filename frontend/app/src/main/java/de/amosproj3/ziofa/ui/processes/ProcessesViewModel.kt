package de.amosproj3.ziofa.ui.processes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.ProcessListAccess
import de.amosproj3.ziofa.bl.ConfigurationManager
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class ProcessesViewModel(processListAccess: ProcessListAccess) : ViewModel() {
    val processesList =
        processListAccess.processesList.map { sortKey -> sortKey.sortedBy { it.pid } }.stateIn(
            viewModelScope,
            started = SharingStarted.Lazily,
            listOf()
        )
}