// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.api.processes

import de.amosproj3.ziofa.client.Process

/**
 * Wrapper class for running components, that can either be groups of processes or single processes
 */
sealed class RunningComponent(val pids: List<UInt>) {

    /** Standalone process that does not belong to an app */
    data class StandaloneProcess(val process: Process) :
        RunningComponent(pids = listOf(process.pid))

    /** Processes that belong to an app, along with the app's information like the icon */
    data class Application(val packageInfo: InstalledPackageInfo, val processList: List<Process>) :
        RunningComponent(pids = processList.map { it.pid })
}
