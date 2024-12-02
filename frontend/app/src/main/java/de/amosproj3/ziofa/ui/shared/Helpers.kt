// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.client.Command
import de.amosproj3.ziofa.ui.processes.ProcessListEntry

fun Command?.toReadableString(): String {
    this?.let {
        return when (this) {
            is Command.Comm -> this.name
            is Command.Cmdline -> this.components.joinToString(" ")
        }
    } ?: return "null"
}

fun ProcessListEntry.getDisplayName(): String {
    return when (this) {
        is ProcessListEntry.ApplicationEntry -> this.packageInfo.displayName
        is ProcessListEntry.ProcessEntry -> this.process.cmd.toReadableString()
    }
}

fun ProcessListEntry.serializePIDs(): String {
    return when (this) {
        is ProcessListEntry.ApplicationEntry -> this.processList.map { it.pid }
        is ProcessListEntry.ProcessEntry -> listOf(this.process.pid)
    }.joinToString(",")
}

fun String.deserializePIDs(): IntArray {
    return this.split(",").map { it.toInt() }.toIntArray()
}

fun IntArray.validPIDsOrNull(): IntArray? {
    if (this.contains(-1)) {
        return null
    }
    return this
}
