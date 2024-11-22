// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import de.amosproj3.ziofa.api.ProcessListAccess
import de.amosproj3.ziofa.bl.PackageInformationProvider
import de.amosproj3.ziofa.ui.shared.toReadableString
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class ProcessesViewModel(
    processListAccess: ProcessListAccess,
    packageInformationProvider: PackageInformationProvider,
) : ViewModel() {
    val processesList =
        processListAccess.processesList
            .map { processList -> processList.groupBy { it.cmd.toReadableString() } }
            .map { packageProcessMap ->
                packageProcessMap.entries.map {
                    val packageNameOrOther = it.key
                    val processList = it.value
                    // TODO We probably should not retrieve the info of all packages everytime
                    packageInformationProvider.getPackageInfo(packageNameOrOther)?.let {
                        ProcessListEntry.ApplicationEntry(it, processList)
                    } ?: ProcessListEntry.ProcessEntry(processList[0])
                }
            }
            .map { uiEntryList ->
                uiEntryList.sortedBy { if (it is ProcessListEntry.ApplicationEntry) -1 else 1 }
            }
            .stateIn(viewModelScope, started = SharingStarted.Lazily, listOf())
}
