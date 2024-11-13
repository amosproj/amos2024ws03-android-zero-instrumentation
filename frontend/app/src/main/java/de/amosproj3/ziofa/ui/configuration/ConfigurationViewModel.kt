// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.configuration

import androidx.lifecycle.ViewModel
import de.amosproj3.ziofa.ui.configuration.data.EBpfProgramOption
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import timber.log.Timber

class ConfigurationViewModel : ViewModel() {

    val mockOptions =
        mutableListOf(
                EBpfProgramOption("uprobe 1", false),
                EBpfProgramOption("uprobe 2", true),
                EBpfProgramOption("uprobe 3", true),
                EBpfProgramOption("uprobe 4", true),
                EBpfProgramOption("uprobe 5", true),
                EBpfProgramOption("uprobe 6", true),
                EBpfProgramOption("uprobe 7", true),
                EBpfProgramOption("uprobe 8", true),
                EBpfProgramOption("uprobe 9", true),
                EBpfProgramOption("uprobe 10", true),
                EBpfProgramOption("uprobe 11", true),
                EBpfProgramOption("uprobe 12", true),
                EBpfProgramOption("uprobe 13", true),
                EBpfProgramOption("uprobe 14", true),
                EBpfProgramOption("uprobe 15", true),
                EBpfProgramOption("uprobe 16", true),
                EBpfProgramOption("uprobe 17", true),
            )
            .associateBy { it.name }
            .toMutableMap() // TODO delete

    private val _programList: MutableStateFlow<List<EBpfProgramOption>> =
        MutableStateFlow(mockOptions.values.toList())
    val programList: StateFlow<List<EBpfProgramOption>> = _programList.asStateFlow()

    fun optionChanged(eBpfProgramOption: EBpfProgramOption, newState: Boolean) {
        mockOptions[eBpfProgramOption.name] = EBpfProgramOption(eBpfProgramOption.name, newState)
        _programList.update { mockOptions.values.toList() }
    }

    fun configurationSubmitted() {
        Timber.i("configurationSubmitted")
        // TODO send to backend and display popup with loading anim
    }
}
