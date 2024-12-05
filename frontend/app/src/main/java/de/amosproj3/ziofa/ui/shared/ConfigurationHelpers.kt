// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.shared

import de.amosproj3.ziofa.api.ConfigurationUpdate
import de.amosproj3.ziofa.client.SysSendmsgConfig
import de.amosproj3.ziofa.client.VfsWriteConfig
import de.amosproj3.ziofa.ui.configuration.data.BackendFeatureOptions

fun VfsWriteConfig?.updatePIDs(
    pidsToAdd: Set<Map.Entry<UInt, ULong>> = setOf(),
    pidsToRemove: Set<Map.Entry<UInt, ULong>> = setOf(),
): VfsWriteConfig {
    val config = this ?: VfsWriteConfig(mapOf())
    return config.copy(
        entries =
            config.entries.entries.plus(pidsToAdd).minus(pidsToRemove).associate {
                it.key to it.value
            }
    )
}

fun SysSendmsgConfig?.updatePIDs(
    pidsToAdd: Set<Map.Entry<UInt, ULong>> = setOf(),
    pidsToRemove: Set<Map.Entry<UInt, ULong>> = setOf(),
): SysSendmsgConfig {
    val config = this ?: SysSendmsgConfig(mapOf())
    return config.copy(
        entries =
            config.entries.entries.plus(pidsToAdd).minus(pidsToRemove).associate {
                it.key to it.value
            }
    )
}

/** Show as enabled depending on the PIDs the screen is configuring. */
fun ConfigurationUpdate.Valid.toUIOptionsForPids(
    relevantPids: List<UInt>? = null
): List<BackendFeatureOptions> {
    val options = mutableListOf<BackendFeatureOptions>()
    if (relevantPids != null) {
        options.add(
            this.configuration.vfsWrite?.let {
                BackendFeatureOptions.VfsWriteOption(
                    enabled = it.entries.keys.anyPidsEnabled(relevantPids),
                    pids = it.entries.keys,
                )
            } ?: BackendFeatureOptions.VfsWriteOption(enabled = false, pids = setOf())
        )

        options.add(
            this.configuration.sysSendmsg?.let {
                BackendFeatureOptions.SendMessageOption(
                    enabled = it.entries.keys.anyPidsEnabled(relevantPids),
                    pids = it.entries.keys,
                )
            } ?: BackendFeatureOptions.SendMessageOption(enabled = false, pids = setOf())
        )
    }
    // TODO add uprobe
    // TODO what to do with global?

    return options.toList()
}
