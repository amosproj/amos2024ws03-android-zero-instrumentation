// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.utils

import de.amosproj3.ziofa.api.RunningComponent
import de.amosproj3.ziofa.ui.shared.toReadableString
import de.amosproj3.ziofa.ui.visualization.data.DropdownOption

fun DropdownOption.getPIDsOrNull(): List<UInt>? {
    return when (this) {
        is DropdownOption.Global -> null
        is DropdownOption.Process -> listOf(this.pid)
        is DropdownOption.AppOption -> this.pids
        else -> throw IllegalStateException("Invalid filter")
    }
}

fun List<RunningComponent>.toFilterOptions() =
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
