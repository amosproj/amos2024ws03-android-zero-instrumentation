// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.processes

import de.amosproj3.ziofa.client.Process

sealed class RunningComponent(val pids: List<UInt>) {
    data class StandaloneProcess(val process: Process) :
        RunningComponent(pids = listOf(process.pid.toUInt()))

    data class Application(val packageInfo: InstalledPackageInfo, val processList: List<Process>) :
        RunningComponent(pids = processList.map { it.pid.toUInt() })
}
