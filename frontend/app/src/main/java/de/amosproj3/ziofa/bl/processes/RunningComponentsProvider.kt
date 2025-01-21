// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.bl.processes

import de.amosproj3.ziofa.api.processes.RunningComponent
import de.amosproj3.ziofa.api.processes.RunningComponentsAccess
import de.amosproj3.ziofa.client.Client
import de.amosproj3.ziofa.client.ClientFactory
import de.amosproj3.ziofa.client.Process
import de.amosproj3.ziofa.ui.shared.PROCESS_LIST_REFRESH_INTERVAL_MS
import de.amosproj3.ziofa.ui.shared.toReadableString
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import timber.log.Timber

/**
 * Provides an updating list of running components, with components being apps (groups of processes)
 * or standalone processes like native processes based on backend data and package info.
 */
class RunningComponentsProvider(
    private val clientFactory: ClientFactory,
    private val packageInformationProvider: PackageInformationProvider,
) : RunningComponentsAccess {
    private val processesList = MutableStateFlow<List<Process>>(listOf())
    private val coroutineScope = CoroutineScope(Dispatchers.IO)
    private var client: Client? = null

    override val runningComponentsList =
        processesList.groupByProcessName().splitIntoAppsAndStandaloneProcesses()

    init {
        coroutineScope.launch {
            try {
                client = clientFactory.connect()
                startPollingProcessList()
            } catch (e: Exception) {
                Timber.e(e, "Error connecting to backend")
            }
        }
    }

    /**
     * Start polling the backend process list and update the [processesList] every
     * [PROCESS_LIST_REFRESH_INTERVAL_MS] milliseconds.
     */
    private suspend fun startPollingProcessList() {
        while (true) {
            delay(PROCESS_LIST_REFRESH_INTERVAL_MS)
            client?.let { client -> processesList.update { client.listProcesses() } }
                ?: processesList.update { listOf() }.also { Timber.w("Client not ready!") }
        }
    }

    /** Group processes based on the [Process.cmd]. */
    private fun Flow<List<Process>>.groupByProcessName() =
        this.map { processList ->
            processList.groupBy { process -> process.cmd.toReadableString() }
        }

    /**
     * Separate grouped processes into apps and standalone processes like native processes. All
     * processes where [Process.cmd] is a package name will be treated as
     * [RunningComponent.Application].
     */
    private fun Flow<Map<String, List<Process>>>.splitIntoAppsAndStandaloneProcesses() =
        this.map { packageProcessMap ->
            packageProcessMap.entries.map { (packageOrProcessName, processList) ->
                packageInformationProvider.getPackageInfo(packageOrProcessName)?.let {
                    RunningComponent.Application(it, processList)
                } ?: RunningComponent.StandaloneProcess(processList[0])
            }
        }
}
