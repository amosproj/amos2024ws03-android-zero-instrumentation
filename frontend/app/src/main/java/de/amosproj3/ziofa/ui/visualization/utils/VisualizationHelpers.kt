// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.ui.shared.toReadableString
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import kotlin.contracts.ExperimentalContracts
import kotlin.contracts.contract

fun DropdownOption.getPIDsOrNull(): List<UInt>? {
    return when (this) {
        is DropdownOption.Global -> null
        is DropdownOption.Process -> listOf(this.pid)
        is DropdownOption.AppOption -> this.pids
        else -> error("Invalid filter")
    }
}

fun List<RunningComponent>.toUIOptions() =
    this.map {
        when (it) {
            is RunningComponent.Application ->
                DropdownOption.AppOption(
                    appName = it.packageInfo.displayName,
                    packageName = it.packageInfo.displayName,
                    icon = it.packageInfo.icon,
                    pids = it.processList.map { it.pid.toUInt() },
                )

            is RunningComponent.StandaloneProcess ->
                DropdownOption.Process(
                    it.process.cmd.toReadableString(),
                    pid = it.process.pid.toUInt(),
                )
        }
    }

@OptIn(ExperimentalContracts::class)
fun isValidSelection(selectedMetric: DropdownOption?, selectedTimeframe: DropdownOption?): Boolean {
    contract {
        returns(true) implies
            (selectedMetric is DropdownOption.MetricOption &&
                selectedTimeframe is DropdownOption.TimeframeOption)
    }

    return selectedMetric != null &&
        selectedMetric is DropdownOption.MetricOption &&
        selectedTimeframe != null &&
        selectedTimeframe is DropdownOption.TimeframeOption
}
