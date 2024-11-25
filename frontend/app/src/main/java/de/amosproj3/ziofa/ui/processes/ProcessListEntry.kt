// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.processes

import de.amosproj3.ziofa.api.InstalledPackageInfo
import uniffi.shared.Process

sealed class ProcessListEntry {

    data class ProcessEntry(val process: Process) : ProcessListEntry()

    data class ApplicationEntry(
        val packageInfo: InstalledPackageInfo,
        val processList: List<Process>,
    ) : ProcessListEntry()
}
