// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.ui.processes.ProcessListEntry
import uniffi.shared.Cmd

fun Cmd?.toReadableString(): String {
    this?.let {
        return when (this) {
            is Cmd.Comm -> this.v1
            is Cmd.Cmdline -> this.v1.args.joinToString(" ")
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
