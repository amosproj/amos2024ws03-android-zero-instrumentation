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

/**
 * Get the PIDs from the given [DropdownOption], or null if it is impossible to derive a pid from
 * the given option. (like Global)
 */
fun DropdownOption.getPIDsOrNull(): List<UInt>? {
    return when (this) {
        is DropdownOption.Global -> null
        is DropdownOption.Process -> listOf(this.pid)
        is DropdownOption.App -> this.pids
        else -> error("Invalid filter")
    }
}

/** Data conversion between [RunningComponent] platform wrapper and DropdownOption (UI wrapper) */
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

/**
 * Assert that the selection is valid (= can be used for selecting the correct chart data) and is
 * complete. The contract allows this function to be used where type safety is required
 */
@OptIn(ExperimentalContracts::class)
fun isValidSelection(
    selectedComponent: DropdownOption?,
    selectedMetric: DropdownOption?,
    selectedTimeframe: DropdownOption?,
): Boolean {
    contract {
        returns(true) implies
            ((selectedComponent is DropdownOption.Process ||
                selectedComponent is DropdownOption.App) &&
                selectedComponent != null &&
                selectedMetric is DropdownOption.Metric &&
                selectedTimeframe is DropdownOption.Timeframe)
    }

    return selectedComponent != null &&
        (selectedComponent is DropdownOption.App || selectedComponent is DropdownOption.Process) &&
        selectedMetric != null &&
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
