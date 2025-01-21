// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.client.Configuration
import de.amosproj3.ziofa.ui.shared.toReadableString
import de.amosproj3.ziofa.ui.shared.toUIOptionsForPids
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption
import kotlin.contracts.ExperimentalContracts
import kotlin.contracts.contract
import kotlinx.collections.immutable.toImmutableList

fun DropdownOption.getPIDsOrNull(): List<UInt>? {
    return when (this) {
        is DropdownOption.Global -> null
        is DropdownOption.Process -> listOf(this.pid)
        is DropdownOption.App -> this.pids
        else -> error("Invalid filter")
    }
}

fun List<RunningComponent>.toUIOptions() =
    this.map { component ->
            when (component) {
                is RunningComponent.Application ->
                    DropdownOption.App(
                        appName = component.packageInfo.displayName,
                        packageName = component.packageInfo.displayName,
                        icon = component.packageInfo.icon,
                        pids = component.processList.map { it.pid },
                    )

                is RunningComponent.StandaloneProcess ->
                    DropdownOption.Process(
                        component.process.cmd.toReadableString(),
                        pid = component.process.pid,
                    )
            }
        }
        .toImmutableList()

@OptIn(ExperimentalContracts::class)
fun isValidSelection(selectedMetric: DropdownOption?, selectedTimeframe: DropdownOption?): Boolean {
    contract {
        returns(true) implies
            (selectedMetric is DropdownOption.Metric &&
                selectedTimeframe is DropdownOption.Timeframe)
    }

    return selectedMetric != null &&
        selectedMetric is DropdownOption.Metric &&
        selectedTimeframe != null &&
        selectedTimeframe is DropdownOption.Timeframe
}

/**
 * Get a list of dropdown options from the [Configuration]. This list only contains metric that are
 * configured (== active) for the any of the given [pids].
 */
fun Configuration.getActiveMetricsForPids(pids: List<UInt>?) =
    this.toUIOptionsForPids(pids)
        .filter { it.active }
        .map { DropdownOption.Metric(it) }
        .toImmutableList()
