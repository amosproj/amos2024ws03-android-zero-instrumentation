// SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.api.RunningComponent
import de.amosproj3.ziofa.client.Command

fun Command?.toReadableString(): String {
    this?.let {
        return when (this) {
            is Command.Comm -> this.name
            is Command.Cmdline -> this.components.joinToString(" ")
        }
    } ?: return "null"
}

fun RunningComponent.getDisplayName(): String {
    return when (this) {
        is RunningComponent.Application -> this.packageInfo.displayName
        is RunningComponent.StandaloneProcess -> this.process.cmd.toReadableString()
    }
}

fun RunningComponent.serializePIDs(): String {
    return when (this) {
        is RunningComponent.Application -> this.processList.map { it.pid }
        is RunningComponent.StandaloneProcess -> listOf(this.process.pid)
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

fun Iterable<UInt>.anyPidsEnabled(pids: List<UInt>?): Boolean =
    pids?.let { this.intersect(it.toSet()).isNotEmpty() } ?: true
